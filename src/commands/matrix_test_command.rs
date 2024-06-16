use async_trait::async_trait;
use crate::t_command::Command;
use crate::errors::jaws_error::JawsError;
use crate::Options;
use crate::matrix_output::{Matrix, MatrixFooter, MatrixHeader, MatrixOutput};

pub struct MatrixTestCommand {}

#[async_trait]
impl Command for crate::commands::matrix_test_command::MatrixTestCommand
{
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError> {
        Ok(())
    }

    fn get_matrix_output(&self) -> Option<MatrixOutput> {
        None
    }
}
