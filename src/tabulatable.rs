use tabled::builder::Builder;
use tabled::settings::Style;

pub trait Tabulatable {
    fn get_table_headers(&self, extended: bool) -> Vec<String>;
    fn get_table_rows(&self, extended: bool) -> Vec<Vec<String>>;

    fn tabulate(&self, extended: bool) {
        let mut builder = Builder::default();

        // Get the tabled headers
        let headers = self.get_table_headers(extended);
        builder.push_record(headers.into_iter());

        for value in self.get_table_rows(extended) {
            builder.push_record(value.into_iter());
        }

        // Print the table
        println!("{}", builder.build().with(Style::rounded()).to_string());
    }
}


