// This file defines the output formats allowed for Jaws.

// CLAP requires the implementation of various traits for custom enums, and this can get
// rather nontrivial, so I've done it all here in its own file.

use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[derive(clap::ValueEnum)]
pub enum OutputFormat {
    /// Output in tabular format (default)
    Tabular,

    /// Output in Json format
    Json,
}

impl OutputFormat {
    pub fn supports_free_text_output(&self) -> bool {
        match *self {
            OutputFormat::Tabular => true,
            OutputFormat::Json => false
        }
    }
}

// A block here from the example which set up the ValueEnum in code has been reduced (as
// is optionally allowed) to the `#[derive(clap::ValueEnum)]` above.  I prefer to use the
// derivation annotation where possible since it doesn't introduce code that could be
// obviated by generation or macro processing.

// Most of this is based off the CLAP example for custom enums
// https://github.com/clap-rs/clap/blob/master/examples/tutorial_builder/04_01_enum.rs

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}
