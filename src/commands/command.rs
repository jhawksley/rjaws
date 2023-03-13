use crate::error::jaws_error::JawsError;
use crate::Options;
use async_trait::async_trait;

#[async_trait]
pub trait Command {
    async fn run(&self, options: &Options) -> Result<(), JawsError>;
}