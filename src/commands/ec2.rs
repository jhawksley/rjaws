use std::borrow::Cow;
use std::ptr::null;
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

        // Get all EC2 instances and run them through Tabled for output
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
        let mut tabled_instances = to_tabled(instances, wide);

        // Sort instances by name (case insensitive)
        tabled_instances.sort_unstable_by_key(|i| i.get_name().to_lowercase());

        // Print the table
        println!("{}", Table::new(tabled_instances).with(Style::rounded()).to_string());
        // panic!(); // Mr Mainwaring!
    }
}

/// Convert a vector of AWS SDK EC2 instances into a vector of
/// Tabled (printable) instances.  If the `wide` option is in force,
/// additional API calls are made to fill out the enhanced fields.
fn to_tabled(instances: Vec<Instance>, extended: bool) -> Vec<EC2TabledInstance> {
    let mut vec: Vec<EC2TabledInstance> = Vec::new();

    for instance in instances {
        if extended {
            // Only gather Wide data if wide is enabled.  Otherwise it will waste time unnecessarily.

            let can_ssm: bool =
                if aws_handler::instance_can_ssm(&instance.instance_id.to_owned().unwrap())
                { true } else { false };

            vec.push(EC2TabledInstance {
                is_wide: extended,
                instance,
                ssm: Some(can_ssm),
                az: None,
                instance_type: None,
                spec: None,
                private_dns: None,
            });
        } else {
            // Non-wide (simple) version
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

impl EC2TabledInstance {
    /// Gets a name for this instance. The `name` tag is used first.  If that is not present,
    /// the tag `aws:eks:cluster-name` is used.  If that is also missing, the `None` variant
    /// is returned.
    pub fn get_name(&self) -> String {
        match find_tag_value(self.instance.tags.as_ref().unwrap(), "Name") {
            Some(string) => string,
            None => match find_tag_value(self.instance.tags.as_ref().unwrap(), "aws:eks:cluster-name") {
                Some(string) => format!("[EKS] {}", string),
                None => "Untitled".to_string()
            }
        }
    }
}

pub fn find_tag_value(tags: &Vec<aws_sdk_ec2::model::Tag>, key: &str) -> Option<String> {
    for tag in tags {
        if tag.key().unwrap() == key {
            return Some(tag.value().unwrap().to_string());
        }
    }

    None
}

// Non-wide mode:
//   Instance ID, Name, State, Public IP, Private IP
// Wide mode, additional:
//   SSM?, AZ, Type, Spec, Private DNS

impl Tabled for EC2TabledInstance {
    const LENGTH: usize = 6;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        let mut vec: Vec<Cow<str>> = Vec::new();
        vec.push(Cow::from(self.instance.instance_id.as_ref().unwrap()));

        let name = self.get_name();
        vec.push(Cow::from(name));
        vec.push(Cow::from(self.instance.state().as_ref().unwrap().name().unwrap().as_str()));

        // IP addresses may not be assigned
        vec.push(Cow::from(
            match self.instance.public_ip_address.as_ref() {
                Some(address) => address,
                None => "None"
            }
        ));

        vec.push(Cow::from(
            match self.instance.private_ip_address.as_ref() {
                Some(address) => address,
                None => "None"
            }
        ));

        // If this is an extended/wide display, also push the extended fields.
        if (self.is_wide) {
            vec.push(Cow::from(match self.ssm {
                Some(ssm) => if (ssm) { "Yes" } else { "No" },
                None => "-"
            }));
        }

        vec
    }

    fn headers() -> Vec<Cow<'static, str>> {
        let mut vec: Vec<Cow<str>> = Vec::new();
        vec.push(Cow::from("Instance ID"));
        vec.push(Cow::from("Name"));
        vec.push(Cow::from("State"));
        vec.push(Cow::from("Public IP"));
        vec.push(Cow::from("Private IP"));
        vec.push(Cow::from("SSM"));

        vec
    }
}

