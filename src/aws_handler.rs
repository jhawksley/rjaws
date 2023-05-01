// https://awslabs.github.io/aws-sdk-rust/

use std::collections::HashMap;
use aws_sdk_ec2;
use aws_sdk_ec2::error::ProvideErrorMetadata;
use aws_sdk_ec2::types::{Instance};
use aws_sdk_iam::types::InstanceProfile;
use aws_sdk_sts;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput;
use crate::commands::notify_comms;

use crate::errors::jaws_error::JawsError;

// ***********************************************************************************************
// STS
// ***********************************************************************************************

/// Return the current context's STS caller identity.
pub async fn sts_get_caller_identity() -> Result<GetCallerIdentityOutput, JawsError> {
    let res = aws_sdk_sts::Client::new(&aws_config::load_from_env().await)
        .get_caller_identity().send().await;


    match res {
        Ok(output) => Ok(output),
        Err(e) => Err(JawsError::new(format!("Ensure your AWS credentials are set correctly in the environment.\n\nThe underlying error is:\n\t{:?}",
                                             e.into_service_error().message().unwrap())))
    }
}

// ***********************************************************************************************
// EC2
// ***********************************************************************************************


pub async fn ec2_get_all() -> Result<Vec<Instance>, JawsError>
{
    let mut instances: Vec<Instance> = Vec::new();

    let client: aws_sdk_ec2::Client = aws_sdk_ec2::Client::new(&aws_config::load_from_env().await);

    let resp_result = client
        .describe_instances()
        .send()
        .await;

    match resp_result {
        Ok(resp) => {
            for reservation in resp.reservations().unwrap_or_default() {
                let list: &[Instance] = reservation.instances().unwrap_or_default();
                for instance in list {
                    // println!("{:?}", instance);
                    instances.push(instance.to_owned());
                }
            }

            Ok(instances)
        }
        Err(error) => Err(JawsError::new(format!("{}", error)))
    }
}

pub async fn instance_can_ssm(instance: &Instance,
                              instance_profile_cache: &mut HashMap<String, InstanceProfile>,
                              instance_profile_ssm_mapping_cache: &mut HashMap<String, bool>) -> bool
{


    // Load cache if necessary
    let client: aws_sdk_iam::Client = aws_sdk_iam::Client::new(&aws_config::load_from_env().await);

    if instance_profile_cache.len() == 0 {
        notify_comms(Some("filling Instance Profile cache".to_string()));
        for ip in client.list_instance_profiles().send()
            .await.unwrap().instance_profiles().unwrap() {
            instance_profile_cache.insert(ip.arn().unwrap().to_string(), ip.clone());
        }
    }

    // Instance may not have a profile
    if instance.iam_instance_profile().is_none() {
        return false;
    }

    // Check whether the answer is cached already
    let ip_arn = instance.iam_instance_profile().unwrap().arn().unwrap();
    let answer: Option<&bool> = instance_profile_ssm_mapping_cache.get( ip_arn );

    // ... yes.
    if answer.is_some() {
        return *answer.unwrap();
    }

    // no...

    // Check whether the instance has policy AmazonSSMManagedInstanceCore in its role
    notify_comms(Some(format!("getting IAM role information {:?}", instance.iam_instance_profile().unwrap().id().unwrap())));

    let instance_profile_arn = instance.iam_instance_profile().unwrap().arn().unwrap();
    let ip = instance_profile_cache.get(instance_profile_arn).unwrap();

    // Get the role in this instance profile -- there can be only one
    let role = &ip.roles().unwrap()[0];

    // Load the policies and check whether the SSM policy is in there.
    let policies = client.list_attached_role_policies().role_name(role.role_name().unwrap()).send().await.unwrap();

    for policy in policies.attached_policies().unwrap() {
        if policy.policy_name().unwrap() == "AmazonSSMManagedInstanceCore" {
            instance_profile_ssm_mapping_cache.insert(ip_arn.to_string(), true );
            return true;
        }
    }

    instance_profile_ssm_mapping_cache.insert(ip_arn.to_string(), false );
    false
}

const TYPE_BATCH_SIZE: i32 = 100;

pub async fn generate_spec_map() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    let client: aws_sdk_ec2::Client = aws_sdk_ec2::Client::new(&aws_config::load_from_env().await);

    let mut response = client.describe_instance_types().
        set_max_results(Some(TYPE_BATCH_SIZE)).send().await.unwrap();

    let mut loaded: usize = 0;

    loop {
        notify_comms(Some(format!("getting instance types [{}]", loaded).to_string()));

        for t in response.instance_types.as_ref().unwrap() {
            let key = t.instance_type().unwrap().as_str().to_string();
            map.insert(key,
                       format!("{}/{}", t.v_cpu_info().unwrap().default_v_cpus().unwrap(),
                               t.memory_info().unwrap().size_in_mi_b().unwrap() / 1024));
        }

        loaded = map.len();

        response = match response.next_token() {
            Some(token) => {
                client.describe_instance_types().set_next_token(Some(token.to_string())).
                    set_max_results(Some(TYPE_BATCH_SIZE)).send().await.unwrap()
                // And loop again
            }
            None => {
                break;
                // And stop looping
            }
        }
    }

    map
}
