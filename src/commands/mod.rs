pub mod gci;
pub mod ec2;
pub mod ssm;
pub mod res;

use crate::errors::jaws_error::JawsError;
use crate::Options;
use async_trait::async_trait;

/// This trait defines the interface which describes a CLI command.
#[async_trait]
pub trait Command {
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError>;
}

