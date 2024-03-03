use async_trait::async_trait;
use aws_sdk_ec2::types::Reservation;
use tracing::debug;
use crate::aws_handler::AWSHandler;

use crate::commands::{Command, notify_clear, notify_comms};
use crate::errors::jaws_error::JawsError;
use crate::Options;

pub struct ResCommand;

impl ResCommand
{
    pub fn new() -> Self {
        Self {
            // Nothing to set yet.
        }
    }
}
#[async_trait]
impl Command for ResCommand
{

    async fn run(&mut self, options: &Options) -> Result<(), JawsError> {

        let handler = AWSHandler::new(options);

        notify_comms(Some("Getting reservation data".to_string()));

        // Get all reservations

        match handler.reservations_get_live().await {
            Ok(reservations) => {
                notify_clear();
                println!("\nres count: {}", reservations.len());
                for res in reservations {
                    println!("{}", res.instance_type.unwrap().as_str());
                    println!("{}", res.instance_count.unwrap());
                }

                Ok(())
            }
            Err(e) => Err(e)
        }

        // Get all unique instance types with a count of that type

        // Calculate the table structure for output

        // .. and output it

        // Ok(())
    }
}
