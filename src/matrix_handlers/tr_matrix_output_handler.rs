use crate::matrix_handlers::t_matrix_output::MatrixOutput;

pub trait TrMatrixOutputHandler {
   fn output(&mut self, matrix_output: &MatrixOutput);
}
