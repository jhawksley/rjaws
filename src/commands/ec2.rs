use async_trait::async_trait;
use aws_sdk_ec2::types::Instance;

use crate::aws_handler::AWSHandler;
use crate::errors::jaws_error::JawsError;
use crate::models::ec2_instance::EC2Instance;
use crate::Options;
use crate::t_command::Command;
use crate::matrix_output::MatrixOutput;
use crate::t_tabulatable::Tabulatable;
use crate::textutils::Textutil;

/// Run an EC2 command.  This type may also be called internally by other commands or
/// functionality.  This type creates its own `AWSHandler`, which itself caches various
/// large or slow datasets from AWS. For this reason, prefer to instantiate and reuse this
/// object, rather than creating new ones.
pub struct EC2Command {
    instances: Vec<EC2Instance>,
    instance_filter: Option<Vec<String>>,
    textutil: Textutil,
    handler: AWSHandler,
}

impl EC2Command {
    pub async fn new(options: &Options) -> Self {
        Self {
            instances: Vec::new(),
            instance_filter: None,
            textutil: Textutil::new(options),
            handler: AWSHandler::new(options).await
        }
    }

    pub(crate) async fn run_with_filter(&mut self, instances: Vec<String>, options: &mut Options) {
        self.instance_filter = Some(instances);
        _ = self.run(options).await;
    }
}

#[async_trait]
impl Command for EC2Command {
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError> {
        // Update the user we're talking to AWS
        self.textutil.notify_comms(Some("checking caller ID".to_string()));

        // Assert we can actually log in.
        self.handler.sts_get_caller_identity().await?;

        self.textutil.notify_comms(Some("getting instances".to_string()));
        // Get all EC2 instances and run them through Tabled for output
        match self.handler.ec2_get_all().await {
            Ok(instances) => {
                if instances.len() == 0 {
                    self.textutil.txt_line_output("No instances found.\n".to_string());
                } else {
                    // Convert the AWS instances to our own type
                    self.textutil.notify_working();
                    self.instances = to_ec2instances(instances, options.wide, &mut self.handler,
                                                     &self.instance_filter).await;
                    self.instances.sort_by_key(|i| i.get_name());
                    self.textutil.notify_clear();
                    (self as &dyn Tabulatable).tabulate(options.wide);
                }
                Ok(()) // TODO MATRIX
            }
            Err(e) => Err(e)
        }
    }

    fn get_matrix_output(&self) -> Option<MatrixOutput> {
        None //TODO Matrix
    }
}

/// Convert a vector of AWS SDK EC2 instances into a vector of
/// Tabled (printable) instances.  If the `wide` option is in force,
/// additional API calls are made to fill out the enhanced fields.
async fn to_ec2instances(instances: Vec<Instance>, extended: bool, handler: &mut AWSHandler, filter: &Option<Vec<String>>) -> Vec<EC2Instance> {
    let mut vec: Vec<EC2Instance> = Vec::new();

    for instance in instances {
        // Only gather Wide data if wide is enabled.  Otherwise it will waste time unnecessarily.

        let mut ssm = None;
        let mut az = None;
        let mut instance_type = None;
        let mut spec = None;

        if extended {
            ssm = if handler.instance_can_ssm(&instance).await {
                Some(true)
            } else {
                Some(false)
            };

            az = Some(instance.placement().unwrap().availability_zone().unwrap().to_string());
            instance_type = Some(instance.instance_type().unwrap().as_str().to_string());

            let k = instance.instance_type().unwrap().as_str();
            spec = handler.get_instance_spec(k).await;
        }

        if filter.is_none() ||
            (filter.is_some() && filter.as_ref().unwrap().contains(&instance.instance_id.as_ref().unwrap())) {
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
