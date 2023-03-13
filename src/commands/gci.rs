use async_trait::async_trait;

use crate::aws_handler;
use crate::commands::command::Command;
use crate::error::jaws_error::JawsError;
use crate::Options;

pub struct GCICommand;

#[async_trait]
impl Command for GCICommand
{
    async fn run(&self, _options: &Options) -> Result<(), JawsError> {
        let id = aws_handler::sts_get_caller_identity();
        let identity_result = id.await;

        match identity_result {
            Ok(identity) => {
                println!("ARN:        {}", identity.arn().unwrap());
                println!("Account:    {}", identity.account().unwrap());
                println!("User:       {}", identity.user_id().unwrap());
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}