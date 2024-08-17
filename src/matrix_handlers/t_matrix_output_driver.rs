use crate::e_output_format::OutputFormat;
use crate::matrix_handlers::t_json_output::JsonOutput;
use crate::matrix_handlers::t_matrix_output::MatrixOutput;
use crate::matrix_handlers::t_tabular_output::TabularOutput;
use crate::matrix_handlers::tr_matrix_output_handler::TrMatrixOutputHandler;

pub struct MatrixOutputDriver {
    pub(crate) output_format: OutputFormat,
    pub(crate) matrix_output: MatrixOutput,
}


impl MatrixOutputDriver {
    pub fn output(&mut self) {
        let mut formatter: Box<dyn TrMatrixOutputHandler> =
            match self.output_format {
                OutputFormat::Tabular => Box::new(TabularOutput {}) as Box<dyn TrMatrixOutputHandler>,
                OutputFormat::Json => Box::new(JsonOutput {}) as Box<dyn TrMatrixOutputHandler>
            };

        formatter.output(&self.matrix_output);
    }
}
