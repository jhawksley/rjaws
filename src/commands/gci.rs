use async_trait::async_trait;

use crate::t_aws_handler::AWSHandler;
use crate::errors::jaws_error::JawsError;
use crate::Options;
use crate::t_command::Command;
use crate::matrix_handlers::t_matrix_output::MatrixOutput;
use crate::textutils::Textutil;

pub struct GCICommand;

#[async_trait]
impl Command for GCICommand
{
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError> {
        let textutil = Textutil::new(options);

        textutil.notify_comms(None);
        let handler: AWSHandler = AWSHandler::new(options).await;

        let id = handler.sts_get_caller_identity();
        let identity_result = id.await;

        textutil.notify_clear();

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

    fn get_matrix_output(&mut self) -> Option<MatrixOutput> {
        None
    }
}
