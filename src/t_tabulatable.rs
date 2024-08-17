use tabled::builder::Builder;
use tabled::settings::object::Rows;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Alignment, Settings, Style, Width};
use tabled::Table;

use crate::textutils::get_terminal_size;

// TODO after conversion to matrix - delete this type.

pub trait Tabulatable {
    fn get_table_headers(&self, extended: bool) -> Vec<String>;
    fn get_table_rows(&self, extended: bool) -> Vec<Vec<String>>;

    fn modify(&self, _table: &mut Table) {
        // Default impl does nothing
    }

    fn tabulate(&self, extended: bool) {
        let mut builder = Builder::default();

        // Get the tabled headers
        let headers = self.get_table_headers(extended);
        builder.push_record(headers.into_iter());

        for value in self.get_table_rows(extended) {
            builder.push_record(value.into_iter());
        }

        // Enlarge to term width
        let (width, _height) = get_terminal_size();

        //        Width::wrap(width).priority(PriorityMax),

        let term_size_settings = Settings::default()
            .with(Width::wrap(width).priority(PriorityMax))
            .with(Width::increase(width));

        let mut builder = builder.build();

        let table = builder
            .with(Style::rounded())
            .with(term_size_settings);
        // .modify(Columns::new(1..), Alignment::right())

        // Center the first row
        table.modify(Rows::first(), Alignment::center());

        // Allow the subtype to modify the table prior to printing, to apply any formatting etc.
        self.modify(table);

        // Print the table
        println!("{table}");
    }
}


