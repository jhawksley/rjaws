use async_trait::async_trait;

use crate::commands::Command;
use crate::errors::jaws_error::JawsError;
use crate::Options;

pub struct SSMCommand {
    instance_id: String
}

impl SSMCommand {
    pub fn new(instance_id: &String) -> Self {
        Self {
            instance_id: instance_id.to_string()
        }
    }
}

#[async_trait]
impl Command for SSMCommand {
    async fn run(&mut self, options: &Options) -> Result<(), JawsError> {
        println!("{}", self.instance_id);

        // The implementation of this should really be to get the WSS urls
        // and attach them to stdin/stdout.
        // To get something running, we use the old Jaws 2 way of spawning SSM.

        // let child = std::process::Command::new("aws").arg("ssm").arg("start-session").arg(format!("--target {}", self.instance_id)).spawn().expect("SSM spawn command failed.");

        // subprocess::Exec::cmd("")

        Ok(())
    }
}