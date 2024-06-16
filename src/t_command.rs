use async_trait::async_trait;
use crate::errors::jaws_error::JawsError;
use crate::Options;
use crate::matrix_output::MatrixOutput;

/// This trait defines the interface which describes a CLI command.
#[async_trait]
pub trait Command {
    /// Run the command, returning either nothing, or a Matrix, which can be processed into
    /// different output formats.
    async fn run(&mut self, options: &mut Options) -> Result<(), JawsError>;

    /// If a command returns `Some` here, it is signalling that the output routine should
    /// process the matrix output into whatever `--output-format` specifies.  The command
    /// should not perform any output of its own (including user update data) in this case.
    fn get_matrix_output(&self) -> Option<MatrixOutput>;
}
