use async_trait::async_trait;

use crate::commands::{Command, notify_comms};
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
        notify_comms(None);

        // Get all reservations

        // Get all unique instance types with a count of that type

        // Calculate the table structure for output

        // .. and output it

        Ok(())
    }
}
