use std::fmt::Display;
use crate::errors::jaws_error::JawsError;
use crate::matrix_handlers::t_matrix_output::{Matrix, MatrixAggregateValue, MatrixFooter, MatrixHeader, MatrixOutput};
use crate::t_command::Command;
use crate::Options;
use async_trait::async_trait;
pub struct MatrixTestCommand {}

#[async_trait]
impl Command for crate::commands::matrix_test_command::MatrixTestCommand
{
    async fn run(&mut self, _: &mut Options) -> Result<(), JawsError> {
        Ok(())
    }

    fn get_matrix_output(&mut self) -> Option<MatrixOutput> {
        Some(self.generate_matrix_test_set())
    }
}

impl MatrixTestCommand {
    fn generate_matrix_test_set(&self) -> MatrixOutput {
        let matrix = Matrix {
            header: Some(vec![String::from("The Matrix Title")]),

            // pub rows: Vec<Vec<Option<Box<dyn Display>>>>,

            rows: Some::<Vec<Vec<Option<Box<dyn Display>>>>> (vec![
                vec![
                    Some(Box::new(String::from("Column A"))),
                    Some(Box::new(String::from("Column B"))),
                    Some(Box::new(String::from("Column C"))),
                ],
                vec![
                    Some(Box::new(String::from("Foo"))),
                    None,
                    Some(Box::new(String::from("32"))),
                ],
                vec![
                    Some(Box::new(String::from("Bar"))),
                    Some(Box::new(String::from("Quux"))),
                    Some(Box::new(String::from("Slartibartfast"))),
                ],
                vec![
                    None,
                    Some(Box::new(String::from("Arthur"))),
                    Some(Box::new(String::from("Trillian"))),
                ],
            ]),
            //    pub aggregate_rows: Vec<Vec<MatrixAggregateValue>>,
            aggregate_rows: Some(
                vec![
                    MatrixAggregateValue {
                        name: String::from("Total"),
                        value: Box::new(2)
                    },
                    MatrixAggregateValue {
                        name: String::from("Average"),
                        value: Box::new(std::f64::consts::PI)
                    },
                    MatrixAggregateValue {
                        name: String::from("95th Percentile"),
                        value: Box::new(78)
                    },
                    MatrixAggregateValue {
                        name: String::from("Fred"),
                        value: Box::new("Jim, Sheila")
                    },
                ]
            ),
            notes: Some(vec![
                String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin ut lobortis ipsum, et efficitur enim. Sed imperdiet risus ut dapibus tristique. Sed convallis lectus nulla, non cursus lorem mollis et. "),
                String::from("Sed at viverra erat. Pellentesque non risus molestie, aliquam eros a, consequat neque. Morbi volutpat rhoncus sem, in posuere nunc pharetra quis. Etiam eget neque eu odio dignissim aliquam ut ut velit.")
            ]),
            first_rows_header: true,
        };

        MatrixOutput {
            matrix_header: Some(MatrixHeader {
                title: Some(String::from("Matrix Header")),
                output_program_header: true,
            }),
            matrix_footer: Some(MatrixFooter {
                footer: Some(String::from("Matrix Footer")),
                output_program_footer: true,
            }),
            matrices: vec![matrix],
        }
    }
}
