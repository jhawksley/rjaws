// https://awslabs.github.io/aws-sdk-rust/

use std::collections::HashMap;

use aws_config::{BehaviorVersion, Region};
use aws_sdk_ec2;
use aws_sdk_ec2::error::ProvideErrorMetadata;
use aws_sdk_ec2::types::{Filter, Instance, InstanceType, ReservedInstances};
use aws_sdk_iam::types::InstanceProfile;
use aws_sdk_pricing::types;
use aws_sdk_pricing::types::FilterType;
use aws_sdk_sts;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput;
use serde_json::Value;

use crate::errors::jaws_error::JawsError;
use crate::Options;
use crate::textutils::Textutil;

const TYPE_BATCH_SIZE: i32 = 100;

pub struct AWSHandler {
    instance_profile_cache: HashMap<String, InstanceProfile>,
    instance_profile_ssm_mapping_cache: HashMap<String, bool>,
    specmap: HashMap<String, String>,
    odm_rate_cache: HashMap<InstanceType, f32>,
    region: Option<String>,
    textutil: Textutil,
}


impl AWSHandler {
    /// Get a new handler, primed with any optional elements.
    pub async fn new(options: &Options) -> Self {
        println!("!! NEW HANDLER");
        let mut handler = AWSHandler {
            instance_profile_cache: HashMap::new(),
            instance_profile_ssm_mapping_cache: HashMap::new(),
            specmap: HashMap::new(),
            odm_rate_cache: HashMap::new(),
            region: None,
            textutil: Textutil::new(options),
        };
        // Load the region from options, if Some.  If None, load using AWS defaulting.
        handler.region = match &options.region {
            None => Some(aws_config::load_defaults(BehaviorVersion::latest()).await.region().unwrap().to_string()),
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
            self.textutil.notify_comms(Some("filling Instance Profile cache".to_string()));
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
        self.textutil.notify_comms(Some(format!(
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

    pub(crate) async fn get_odm_rate(&mut self, instance_type: &InstanceType) -> f32 {
        // Get the on-demand rate for a given instance type.

        // Check if it's already in the cache
        if ! self.odm_rate_cache.contains_key(instance_type) {

            // Get the on-demand rate and cache it, then return it
            // AWS Pricing is not available everywhere - we use eu-central-1 to access it.

            let mut config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            config = config.to_builder().region(Region::from_static("eu-central-1")).build();

            let client = aws_sdk_pricing::Client::new(&config);

            let instance_type_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("instanceType")
                .value(instance_type.as_str())
                .build().unwrap();
            let region_code_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("regionCode")
                .value(self.region.as_ref().unwrap())
                .build().unwrap();
            let software_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("preInstalledSw")
                .value("NA")
                .build().unwrap();
            let tenancy_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("tenancy")
                .value("Shared")
                .build().unwrap();
            let product_family_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("productfamily")
                .value("compute instance")
                .build().unwrap();
            let os_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("operatingSystem")
                .value("Linux")
                .build().unwrap();
            let capacity_filter = types::Filter::builder()
                .r#type(FilterType::TermMatch)
                .field("capacityStatus")
                .value("Used")
                .build().unwrap();

            let result = client
                .get_products()
                .filters(instance_type_filter)
                .filters(region_code_filter)
                .filters(software_filter)
                .filters(product_family_filter)
                .filters(tenancy_filter)
                .filters(os_filter)
                .filters(capacity_filter)
                .service_code("AmazonEC2")
                .send().await.unwrap();

            // for price in result.price_list() {
            //     println!("{}", price);
            // }

            assert_eq!(result.price_list().len(), 1, "ODM pricing data search for '{}' returned non-unique result ({}).", instance_type.to_string(), result.price_list().len());

            let price_data: Value = serde_json::from_str(result.price_list()[0].as_str()).unwrap();

            // The next stage is a bit fiddly, because the name of the key after the tree node "OnDemand"
            // is dynamic.`as_object()` converts it to a map, so we can get the first value.
            let mut odm: &Value = &price_data["terms"]["OnDemand"].as_object().unwrap()
                .values().next().unwrap();

            // ... And again.
            odm = odm["priceDimensions"].as_object().unwrap()
                .values().next().unwrap();

            odm = &odm["pricePerUnit"]["USD"];

            // Get the final price out of the string.
            let price: f32 = odm.as_str().unwrap().parse::<f32>().unwrap();

            self.odm_rate_cache.insert(instance_type.clone(), price);
        }

        return *self.odm_rate_cache.get(instance_type).unwrap();
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
            self.textutil.notify_comms(Some(
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
