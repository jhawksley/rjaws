pub mod gci;
pub mod ec2;

use crate::errors::jaws_error::JawsError;
use crate::Options;
use async_trait::async_trait;
use crate::textutils::{txt_line_clear, txt_line_output};

#[async_trait]
pub trait Command {
    async fn run(&mut self, options: &Options) -> Result<(), JawsError>;
}

pub fn notify_comms(action: Option<String>) {
    match action {
        Some(action) => txt_line_output(format!("Talking to AWS ({})...", action)),
        None => txt_line_output("Talking to AWS...".to_string())
    }
}

pub fn notify_working() {
    txt_line_output("Marshalling data...".to_string());
}

pub fn notify_clear() {
    txt_line_clear();
}