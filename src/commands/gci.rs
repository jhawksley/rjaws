use async_trait::async_trait;

use crate::aws_handler;
use crate::aws_handler::AWSHandler;
use crate::commands::{Command, notify_clear, notify_comms};
use crate::errors::jaws_error::JawsError;
use crate::Options;

pub struct GCICommand;

#[async_trait]
impl Command for GCICommand
{
    async fn run(&mut self, _options: &Options) -> Result<(), JawsError> {

        notify_comms(None);
        let handler: AWSHandler = AWSHandler::default();

        let id = handler.sts_get_caller_identity();
        let identity_result = id.await;

        notify_clear();

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