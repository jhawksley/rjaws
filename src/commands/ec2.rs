use std::borrow::Cow;
use crate::commands::command::Command;
use crate::error::jaws_error::JawsError;
use crate::{aws_handler, Options};
use async_trait::async_trait;
use tabled::{Tabled, Table, Style};
use aws_sdk_ec2::model::Instance;

pub struct EC2Command;

#[async_trait]
impl Command for EC2Command {
    async fn run(&self, options: &Options) -> Result<(), JawsError> {

        // Assert we can actually log in.
        aws_handler::sts_get_caller_identity().await?;

        match aws_handler::ec2_get_all().await {
            Ok(instances) => {
                if instances.len() == 0 {
                    println!("No instances found.");
                } else {
                    EC2Command::instance_tabulator(instances, options.wide);
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}


impl EC2Command {
    fn instance_tabulator(instances: Vec<Instance>, wide: bool) {
        // Take the passed instances and create Tabled Instances out of them
        let tabled_instances = to_tabled(instances, wide);
        // Print the table
        // println!("{:?}", tabled_instances);
        println!("{}", Table::new(tabled_instances).with(Style::rounded()).to_string());
        // panic!();
    }
}

/// Convert a vector of AWS SDK EC2 instances into a vector of
/// Tabled (printable) instances.  If the `wide` option is in force,
/// additional API calls are made to fill out the enhanced fields.
fn to_tabled(instances: Vec<Instance>, extended: bool) -> Vec<EC2TabledInstance> {
    let mut vec: Vec<EC2TabledInstance> = Vec::new();

    for instance in instances {
        vec.push(EC2TabledInstance {
            is_wide: extended,
            instance,
            ssm: None,
            az: None,
            instance_type: None,
            spec: None,
            private_dns: None,
        });
    }

    vec
}

#[derive(Debug)]
struct EC2TabledInstance {
    is_wide: bool,

    instance: aws_sdk_ec2::model::Instance,
    ssm: Option<bool>,
    az: Option<String>,
    instance_type: Option<String>,
    spec: Option<String>,
    private_dns: Option<String>,
}


// Non-wide mode:
//   Instance ID, Name, State, Public IP, Private IP
// Wide mode, additional:
//   SSM?, AZ, Type, Spec, Private DNS

impl Tabled for EC2TabledInstance {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        let mut vec: Vec<Cow<str>> = Vec::new();
        vec.push(Cow::from(self.instance.instance_id.as_ref().unwrap()) );
        vec.push(Cow::from( find_tag_value(self.instance.tags.as_ref().unwrap(), "Name" )));
        vec.push(Cow::from(self.instance.state().as_ref().unwrap().name().unwrap().as_str() ) );
        vec.push(Cow::from(self.instance.public_ip_address.as_ref().unwrap().as_str() ) );
        vec.push(Cow::from(self.instance.private_ip_address.as_ref().unwrap().as_str()) );


        vec
    }

    fn headers() -> Vec<Cow<'static, str>> {
        let mut vec: Vec<Cow<str>> = Vec::new();
        vec.push(Cow::from("Instance ID"));
        vec.push( Cow::from("Name"));
        vec.push( Cow::from("State"));
        vec.push( Cow::from("Public IP"));
        vec.push( Cow::from("Private IP"));
        vec
    }
}

fn find_tag_value(tags: &Vec<aws_sdk_ec2::model::Tag>, key: &str) -> String {

    for tag in tags {
        if tag.key().unwrap() == key {
            return tag.value().unwrap().to_string();
        }
    }

    return "Not Found".to_string();
}