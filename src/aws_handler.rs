// https://awslabs.github.io/aws-sdk-rust/

use aws_config::BehaviorVersion;
use std::collections::HashMap;
use std::ops::Deref;

use aws_sdk_ec2;
use aws_sdk_ec2::error::ProvideErrorMetadata;
use aws_sdk_ec2::types::builders::FilterBuilder;
use aws_sdk_ec2::types::{Filter, Instance, Reservation, ReservedInstances};
use aws_sdk_iam::types::InstanceProfile;
use aws_sdk_sts;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput;
use tracing::debug;
use tracing_subscriber::filter::Filtered;

use crate::commands::notify_comms;
use crate::errors::jaws_error::JawsError;
use crate::Options;

const TYPE_BATCH_SIZE: i32 = 100;

pub struct AWSHandler {
    instance_profile_cache: HashMap<String, InstanceProfile>,
    instance_profile_ssm_mapping_cache: HashMap<String, bool>,
    specmap: HashMap<String, String>,
    region: Option<String>,
}

impl Default for AWSHandler {
    fn default() -> Self {
        Self {
            instance_profile_cache: HashMap::new(),
            instance_profile_ssm_mapping_cache: HashMap::new(),
            specmap: HashMap::new(),
            region: None,
        }
    }
}

impl AWSHandler {
    /// Get a new handler, primed with any optional elements.
    pub fn new(options: &Options) -> Self {
        let mut handler = AWSHandler::default();
        handler.region = match &options.region {
            None => None,
            Some(region) => Some(region.to_string()),
        };

        // Set this in the environment if it's set.  The AWS library will then pick it up
        // during the various client creation statements.
        if handler.region.is_some() {
            std::env::set_var("AWS_DEFAULT_REGION", handler.region.as_ref().unwrap());
        }

        handler
    }
    /// Return the current context's STS caller identity.
    pub async fn sts_get_caller_identity(&self) -> Result<GetCallerIdentityOutput, JawsError> {
        let res =
            aws_sdk_sts::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await)
                .get_caller_identity()
                .send()
                .await;

        match res {
            Ok(output) => Ok(output),
            Err(e) => Err(JawsError::new(format!("Ensure your AWS credentials are set correctly in the environment.\n\nThe underlying error is:\n\t{:?}",
                                                 e.into_service_error().message().unwrap())))
        }
    }

    pub async fn ec2_get_all(&self) -> Result<Vec<Instance>, JawsError> {
        let mut instances: Vec<Instance> = Vec::new();

        let client: aws_sdk_ec2::Client =
            aws_sdk_ec2::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

        let resp_result = client.describe_instances().send().await;

        match resp_result {
            Ok(resp) => {
                for reservation in resp.reservations() {
                    let list: &[Instance] = reservation.instances();
                    for instance in list {
                        // println!("{:?}", instance);
                        instances.push(instance.to_owned());
                    }
                }

                Ok(instances)
            }
            Err(error) => Err(JawsError::new(format!("{}", error))),
        }
    }

    pub async fn instance_can_ssm(&mut self, instance: &Instance) -> bool {
        // Load cache if necessary
        let client: aws_sdk_iam::Client =
            aws_sdk_iam::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

        if self.instance_profile_cache.len() == 0 {
            notify_comms(Some("filling Instance Profile cache".to_string()));
            for ip in client
                .list_instance_profiles()
                .send()
                .await
                .unwrap()
                .instance_profiles()
            {
                self.instance_profile_cache
                    .insert(ip.arn().to_string(), ip.clone());
            }
        }

        // Instance may not have a profile
        if instance.iam_instance_profile().is_none() {
            return false;
        }

        // Check whether the answer is cached already
        let ip_arn = instance.iam_instance_profile().unwrap().arn().unwrap();
        let answer: Option<&bool> = self.instance_profile_ssm_mapping_cache.get(ip_arn);

        // ... yes.
        if answer.is_some() {
            return *answer.unwrap();
        }

        // no...

        // Check whether the instance has policy AmazonSSMManagedInstanceCore in its role
        notify_comms(Some(format!(
            "getting IAM role information {:?}",
            instance.iam_instance_profile().unwrap().id().unwrap()
        )));

        let instance_profile_arn = instance.iam_instance_profile().unwrap().arn().unwrap();
        let ip = self
            .instance_profile_cache
            .get(instance_profile_arn)
            .unwrap();

        // Get the role in this instance profile -- there can be only one
        let role = &ip.roles()[0];

        // Load the policies and check whether the SSM policy is in there.
        let policies = client
            .list_attached_role_policies()
            .role_name(role.role_name())
            .send()
            .await
            .unwrap();

        for policy in policies.attached_policies() {
            if policy.policy_name().unwrap() == "AmazonSSMManagedInstanceCore" {
                self.instance_profile_ssm_mapping_cache
                    .insert(ip_arn.to_string(), true);
                return true;
            }
        }

        self.instance_profile_ssm_mapping_cache
            .insert(ip_arn.to_string(), false);
        false
    }

    pub async fn get_instance_spec(&mut self, instance_type_key: &str) -> Option<String> {
        if self.specmap.len() == 0 {
            self.populate_spec_map().await;
        }

        let value = self.specmap.get(instance_type_key);

        // This is rather ugly but avoids returning an Option<&String>.
        return match value {
            Some(k) => Some(k.to_string()),
            None => None,
        };
    }

    pub async fn reservations_get_live(&self) -> Result<Vec<ReservedInstances>, JawsError> {
        let client: aws_sdk_ec2::Client =
            aws_sdk_ec2::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

        let result = client
            .describe_reserved_instances()
            .filters(Filter::builder().name("state").values("active").build())
            .send()
            .await;

        match result {
            Ok(resp) => Ok(resp.reserved_instances.unwrap()),
            Err(error) => Err(JawsError::new(format!("{}", error))),
        }
    }

    // -------------------------------------------------------------------------------
    // Private
    // -------------------------------------------------------------------------------

    async fn populate_spec_map(&mut self) {
        let client: aws_sdk_ec2::Client =
            aws_sdk_ec2::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

        let mut response = client
            .describe_instance_types()
            .set_max_results(Some(TYPE_BATCH_SIZE))
            .send()
            .await
            .unwrap();

        let mut loaded: usize = 0;

        loop {
            notify_comms(Some(
                format!("getting instance types [{}]", loaded).to_string(),
            ));

            for t in response.instance_types.as_ref().unwrap() {
                let key = t.instance_type().unwrap().as_str().to_string();
                self.specmap.insert(
                    key,
                    format!(
                        "{}/{}",
                        t.v_cpu_info().unwrap().default_v_cpus().unwrap(),
                        t.memory_info().unwrap().size_in_mib().unwrap() / 1024
                    ),
                );
            }

            loaded = self.specmap.len();

            response = match response.next_token() {
                Some(token) => {
                    client
                        .describe_instance_types()
                        .set_next_token(Some(token.to_string()))
                        .set_max_results(Some(TYPE_BATCH_SIZE))
                        .send()
                        .await
                        .unwrap()
                    // And loop again
                }
                None => {
                    break;
                    // And stop looping
                }
            }
        }
    }
}
