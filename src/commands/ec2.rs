use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_ec2::types::{Instance};
use aws_sdk_iam::types::InstanceProfile;

use crate::{aws_handler, Options};
use crate::aws_handler::{generate_spec_map};
use crate::commands::{Command, notify_clear, notify_comms, notify_working};
use crate::errors::jaws_error::JawsError;
use crate::models::ec2_instance::EC2Instance;
use crate::tabulatable::Tabulatable;
use crate::textutils::txt_line_output;

pub struct EC2Command {
    instances: Vec<EC2Instance>,
}

impl EC2Command {
    pub fn new() -> Self {
        Self {
            instances: Vec::new()
        }
    }
}

#[async_trait]
impl Command for EC2Command {
    async fn run(&mut self, options: &Options) -> Result<(), JawsError> {
        // Update the user we're talking to AWS
        notify_comms(Some("checking caller ID".to_string()));

        // Assert we can actually log in.
        aws_handler::sts_get_caller_identity().await?;

        notify_comms(Some("getting instances".to_string()));
        // Get all EC2 instances and run them through Tabled for output
        match aws_handler::ec2_get_all().await {
            Ok(instances) => {
                if instances.len() == 0 {
                    txt_line_output("No instances found.".to_string());
                } else {
                    // Convert the AWS instances to our own type
                    notify_working();
                    self.instances = to_ec2instances(instances, options.wide).await;
                    self.instances.sort_by_key(|i| i.get_name());
                    notify_clear();
                    (self as &dyn Tabulatable).tabulate(options.wide);
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}

/// Convert a vector of AWS SDK EC2 instances into a vector of
/// Tabled (printable) instances.  If the `wide` option is in force,
/// additional API calls are made to fill out the enhanced fields.
async fn to_ec2instances(instances: Vec<Instance>, extended: bool) -> Vec<EC2Instance> {
    let mut vec: Vec<EC2Instance> = Vec::new();

    let mut specmap: Option<HashMap<String, String>> = None;

    if extended {
        specmap = Some(generate_spec_map().await);
    }

    let mut instance_profile_cache: HashMap<String, InstanceProfile> = HashMap::new();
    let mut instance_ssm_cache: HashMap<String, bool> = HashMap::new();

    for instance in instances {
        // Only gather Wide data if wide is enabled.  Otherwise it will waste time unnecessarily.

        let mut ssm = None;
        let mut az = None;
        let mut instance_type = None;
        let mut spec = None;

        if extended {
            ssm = if aws_handler::instance_can_ssm(&instance, &mut instance_profile_cache,
                                                   &mut instance_ssm_cache).await {
                Some(true)
            } else {
                Some(false)
            };

            az = Some(instance.placement().unwrap().availability_zone().unwrap().to_string());
            instance_type = Some(instance.instance_type().unwrap().as_str().to_string());
            let k = instance.instance_type().unwrap().as_str();
            spec = Some(specmap.as_ref().unwrap().get(k).unwrap().to_string());
        }

        vec.push(EC2Instance {
            is_extended: extended,
            instance,
            // Extended types
            ssm,
            az,
            instance_type,
            spec,
        });
    }

    vec
}

impl Tabulatable for EC2Command {
    fn get_table_headers(&self, extended: bool) -> Vec<String> {
        let mut headers: Vec<String> = Vec::new();

        headers.push("Instance ID".to_string());
        headers.push("Name".to_string());
        headers.push("State".to_string());
        headers.push("Public IP".to_string());
        headers.push("Private IP".to_string());

        if extended {
            // Add the wide fields
            headers.push("SSM".to_string());
            headers.push("AZ".to_string());
            headers.push("Type".to_string());
            headers.push("Spec".to_string());
        }

        headers
    }

    fn get_table_rows(&self, extended: bool) -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = Vec::new();

        for instance in self.instances.iter() {
            rows.push(instance.values(extended));
        }

        rows
    }
}
