use async_trait::async_trait;
use aws_sdk_ec2::types::Instance;
use std::fmt::Display;

use crate::errors::jaws_error::JawsError;
use crate::matrix_handlers::t_matrix_output::{Matrix, MatrixAggregateValue, MatrixFooter, MatrixHeader, MatrixOutput, MatrixRowsT};
use crate::t_aws_handler::AWSHandler;
use crate::t_command::Command;
use crate::t_ec2_instance::EC2Instance;
use crate::textutils::Textutil;
use crate::Options;

/// Run an EC2 command.  This type may also be called internally by other commands or
/// functionality.  This type creates its own `AWSHandler`, which itself caches various
/// large or slow datasets from AWS. For this reason, prefer to instantiate and reuse this
/// object, rather than creating new ones.
pub struct EC2Command {
    instances: Vec<EC2Instance>,
    instance_filter: Option<Vec<String>>,
    textutil: Textutil,
    handler: AWSHandler,
    extended_output: bool,
}

impl EC2Command {
    pub async fn new(options: &Options) -> Self {
        Self {
            instances: Vec::new(),
            instance_filter: None,
            textutil: Textutil::new(options),
            handler: AWSHandler::new(options).await,
            extended_output: options.wide,
        }
    }

    pub(crate) async fn run_with_filter(&mut self, instances: Vec<String>, options: &mut Options) {
        self.instance_filter = Some(instances);
        _ = self.run(options).await;
    }

    fn generate_matrix(&self) -> Matrix {
        // Header
        let mut header: Vec<Option<Box<dyn Display>>> = Vec::new();
        header.push(Some(Box::new("Instance ID".to_string())));
        header.push(Some(Box::new("Name".to_string())));
        header.push(Some(Box::new("Status".to_string())));
        header.push(Some(Box::new("Public IP".to_string())));
        header.push(Some(Box::new("Private IP".to_string())));
        header.push(Some(Box::new("Spot".to_string())));

        if self.extended_output {
            header.push(Some(Box::new("SSM".to_string())));
            header.push(Some(Box::new("AZ".to_string())));
            header.push(Some(Box::new("Type".to_string())));
            header.push(Some(Box::new("Spec".to_string())));
        }

        // Generate row data
        let mut main_rows: MatrixRowsT = Vec::new();
        main_rows.push(header);

        // Aggregate
        let mut cpu_tot = 0;
        let mut mem_tot = 0;

        for instance in &self.instances {
            let mut row: Vec<Option<Box<dyn Display>>> = Vec::new();

            row.push(Some(Box::new(instance.instance.instance_id.clone().unwrap().to_string())));
            row.push(Some(Box::new(instance.get_name())));
            row.push(Some(Box::new(instance.instance.state.clone().unwrap().name.unwrap().to_string())));
            row.push(Some(Box::new(instance.instance.public_ip_address.clone().unwrap_or("None".to_string()).to_string())));
            row.push(Some(Box::new(instance.instance.private_ip_address.clone().unwrap().to_string())));
            let spot = instance.instance.spot_instance_request_id().is_some();
            row.push(Some(Box::new(if spot { "Yes".to_string() } else { "No".to_string() })));

            if self.extended_output {
                row.push(Some(Box::new(match instance.ssm {
                    None => { "-".to_string() }
                    Some(ssm) => {
                        match ssm {
                            true => { "Yes".to_string() }
                            false => { "No".to_string() }
                        }
                    }
                })));
                row.push(Some(Box::new(instance.az.clone().unwrap_or("Unknown".to_string()))));
                row.push(Some(Box::new(instance.instance_type.clone().unwrap_or("Unknown".to_string()))));

                let spec = instance.spec.clone();

                if spec.is_some() {
                    let mut parts = spec.as_ref().unwrap().split("/");
                    cpu_tot = cpu_tot + parts.next().unwrap().parse::<i32>().unwrap();
                    mem_tot = mem_tot + parts.next().unwrap().parse::<i32>().unwrap();
                }

                row.push(Some(Box::new(spec.unwrap_or("Unknown".to_string()))));
            }

            main_rows.push(row);
        }

        // Aggregate rows

        
        let mut aggregate_rows: Vec<MatrixAggregateValue> = Vec::new();
        aggregate_rows.push(MatrixAggregateValue {
            name: "Fleet CPU Total".to_string(),
            value: Box::new(cpu_tot.to_string()),
        });
        aggregate_rows.push(MatrixAggregateValue {
            name: "Fleet Memory Total".to_string(),
            value: Box::new(mem_tot.to_string()),
        });

        // Return the completed matrix.

        Matrix {
            header: Some(vec!["Instance Inventory".to_string()]),
            rows: Some(main_rows),
            aggregate_rows: Some(aggregate_rows),
            notes: None,
            first_rows_header: true,
        }
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
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    fn get_matrix_output(&mut self) -> Option<MatrixOutput> {
        Some(
            MatrixOutput {
                matrix_header: Some(MatrixHeader { title: Some("EC2".to_string()), output_program_header: true }),
                matrix_footer: Some(MatrixFooter { footer: None, output_program_footer: true }),
                matrices: vec![self.generate_matrix()],
            }
        )
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
