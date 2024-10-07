use crate::matrix_handlers::tr_matrix_output_handler::TrMatrixOutputHandler;
use crate::matrix_handlers::t_matrix_output::MatrixOutput;

pub struct JsonOutput;

impl TrMatrixOutputHandler for JsonOutput {
    fn output(&mut self, matrix_output: &MatrixOutput) {
        println!("DUMMY OUTPUT HANDLER -> Json")
    }
}
