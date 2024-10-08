use crate::matrix_handlers::t_matrix_output::{Matrix, MatrixFooter, MatrixHeader, MatrixOutput};
use crate::matrix_handlers::tr_matrix_output_handler::TrMatrixOutputHandler;
use crate::textutils::get_terminal_size;
use crate::tui::{tui_center_text, tui_lcr_text, tui_separator_bar};
use chrono::Utc;
use tabled::builder::Builder;
use tabled::settings::object::Columns;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::themes::ColumnNames;
use tabled::settings::width::{MinWidth, Wrap};
use tabled::settings::{Alignment, Settings, Style, Width};
use whoami::{fallible, username};

pub struct TabularOutput {}

impl TrMatrixOutputHandler for TabularOutput {
    fn output(&mut self, matrix_output: &MatrixOutput) {
        self.output_header(&matrix_output.matrix_header);
        println!();
        
        for matrix in &matrix_output.matrices {
            self.output_matrix(&matrix);
        }
        println!();
        self.output_footer(&matrix_output.matrix_footer);
    }
}

impl TabularOutput {
    fn output_header(&self, matrix_header: &Option<MatrixHeader>) {
        // Program header
        if matrix_header.is_some() {
            println!("{}", termion::style::Bold);
            if matrix_header.as_ref().unwrap().output_program_header {
                println!("{}", tui_lcr_text(
                    Some("J A W S".to_string()),
                    Some("*".repeat(5)),
                    Some(format!("v{}", crate::VERSION).to_string()),
                ))
            }

            println!("{}", tui_separator_bar("-"));

            println!("{}", termion::style::Reset);

            // Matrix header if present
            if matrix_header.as_ref().unwrap().title.is_some() {
                println!("{}", tui_center_text(
                    matrix_header.as_ref().unwrap().title.as_ref().unwrap()));
            }
        }
    }

    pub(crate) fn output_footer(&self, matrix_footer: &Option<MatrixFooter>) {
        if matrix_footer.is_some() {
            // Matrix footer if present
            if matrix_footer.as_ref().unwrap().footer.is_some() {
                println!("{}", tui_center_text(
                    matrix_footer.as_ref().unwrap().footer.as_ref().unwrap()));
            }

            println!("{}", termion::style::Bold);
            println!("{}", tui_separator_bar("-"));

            if matrix_footer.as_ref().unwrap().output_program_footer {
                println!("{}", tui_lcr_text(
                    Some(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%SZ"))),
                    Some("*".repeat(5)),
                    Some(format!("{}@{}", username(), fallible::hostname().unwrap())),
                ))
            }
            println!("{}", termion::style::Reset);
        }
    }

    pub(crate) fn output_matrix(&self, matrix: &Matrix) {

        if matrix.header.is_some() {
            for header in matrix.header.as_ref().unwrap() {
                println!("{}\n", tui_center_text(header));
            }
        }

        let (width, _height) = get_terminal_size();
        let term_size_settings = Settings::default()
            .with(Width::wrap(width).priority(PriorityMax))
            .with(Width::increase(width));

        self.output_matrix_data_table(&matrix, &term_size_settings);

        if matrix.aggregate_rows.is_some() {
            self.output_matrix_aggregate_table(&matrix, &term_size_settings);
        }

        if matrix.notes.is_some() {
            self.output_notes(&matrix);
        }
    }

    fn output_matrix_data_table(&self, matrix: &Matrix, term_size_settings: &Settings<Settings<Settings, Wrap<usize, PriorityMax>>, MinWidth>) {
        // Output rows

        let mut builder = Builder::default();

        if matrix.rows.is_some()
        {

            for row in matrix.rows.as_ref().unwrap() {
                // Push all rows
                let mut cells: Vec<String> = Vec::new();
                for cell in row {
                    cells.push(match cell {
                        Some(t) => t.to_string(),
                        None => String::new(),
                    });
                }
                builder.push_record(cells);
            }

            let mut table = builder.build();
            let table = table
                .with(Style::rounded())
                .with(term_size_settings.clone());

            if matrix.first_rows_header {
                // table.modify(Rows::first(), Alignment::center());
                // Experimental - try it out and see if it works
                table.with(Style::rounded().remove_horizontals()).with(ColumnNames::default());
            } else {
                table.with(Style::rounded().remove_horizontals());
            }

            println!("{table}")
        }
    }

    fn output_matrix_aggregate_table(&self, matrix: &Matrix, _term_size_settings: &Settings<Settings<Settings, Wrap<usize, PriorityMax>>, MinWidth>) {
        let mut builder = Builder::default();

        for row in matrix.aggregate_rows.as_ref().unwrap() {
            builder.push_record([&row.name, &row.value.to_string()])
        }

        let mut table = builder.build();

        table.with(Style::rounded().remove_horizontals());
        table.modify(Columns::first(), Alignment::right());
        // table.modify(Rows::first()));

        println!("{table}")
    }

    fn output_notes(&self, matrix: &Matrix) {
        println!("\n{}Notes:{}", termion::style::Underline,
                 termion::style::Reset);

        for (i, note) in matrix.notes.as_ref().unwrap().iter().enumerate() {
            println!("{}{}:{} {note}"
                     , termion::style::Bold, i + 1, termion::style::Reset)
        }
    }
}
