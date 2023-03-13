use std::fmt::{Display, Formatter};

/// Wraps any error that can be thrown during command execution.
#[derive(Debug)]
pub struct JawsError {
    message: String,
}

impl JawsError {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl Display for JawsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JawsError {
}