use std::io::{stdout, Write};
use std::process;
use std::string::ToString;

use clap::Parser;
use colored::*;
use termion::clear::CurrentLine;

use e_output_format::OutputFormat;

use crate::commands::Command;
use crate::errors::jaws_error::JawsError;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod aws_handler;
mod commands;
mod errors;
mod models;
mod tabulatable;
mod textutils;
mod e_output_format;

const LONG_ABOUT_TPL: &str = "JAWS - Nicer(ish) ways of interacting with AWS.\n\
                              John Hawksley <john@hawksley.net>\n\
                              \n\
                              JAWS (jaws) provides some nicer ways of interacting with AWS than the standard\n\
                              AWS CLI.  Since JAWS uses the Rust AWS Library, it requires your shell be \n\
                              correctly configured for AWS access.\n\
                              \n\
                              Use -h for terse and --help for verbose help.\n\
                              \n\
                              Project: https://github.com/jhawksley/rjaws";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = LONG_ABOUT_TPL)]
pub struct Options {
    /// Output wider, more detailed data.  May cause slower execution in some cases.
    /// Not all commands support this.
    #[clap(short, long, default_value_t = false, global = true)]
    wide: bool,

    /// A region to select (otherwise the default region is used)
    #[clap(short, long, global = true)]
    region: Option<String>,

    /// Output format for commands which support it
    // #[clap(short, long, global = true, default_value_t = Some(OutputFormat::Tabular))]
    #[clap(short, long, global = true, default_value = "tabular")]
    pub output_format: Option<OutputFormat>,

    /// A subcommand to run
    #[clap(subcommand)]
    subcommand: SubCommands,
}

// Subcommands and their options
#[derive(clap::Subcommand, Debug)]
enum SubCommands {
    /// List inventory of EC2 instances
    EC2,

    /// Start an SSM (login) session with an instance.
    SSM {
        /// The instance to which to connect
        instance_id: String,
    },

    /// Gets the caller identity from the Security Token Service
    GCI,

    /// Calculate reservation costs and fleet coverage
    RES {
        /// Output additional information about unused reservations
        #[clap(short, long, default_value_t = false)]
        show_unused: bool
    },
}

// Main: starts here. We need tokio because the AWS libraries need it.
#[tokio::main]
async fn main() {

    // Parse options
    let mut options = Options::parse();

    // Switch based on the selected subcommand
    let command: Option<Box<dyn Command>> = match &options.subcommand {
        SubCommands::EC2 => Some(Box::new(commands::ec2::EC2Command::new(&options).await)),
        SubCommands::GCI => Some(Box::new(commands::gci::GCICommand)),
        SubCommands::SSM { instance_id: _ } => {
            Some(Box::new(commands::ssm::SSMCommand::new()))
        }
        SubCommands::RES { show_unused: _ } => Some(Box::new(commands::res::ResCommand::new())),
    };

    match command {
        Some(mut c) => {
            match c.run(&mut options).await {
                Ok(_) => {} // Success - command ran to completion
                Err(e) => handle_and_abort(e),
            }
        }
        None => handle_and_abort(JawsError::new(format!(
            "Command '{:?}' not found",
            options.subcommand
        ))),
    }
}

fn handle_and_abort(error: JawsError) {
    // This was a call to txt_line_clear, but since the rejig of text output, and the fact that
    // this function should probably have as few dependencies as possible, I've inlined it.
    print!("\r{}", CurrentLine);
    _ = stdout().flush();

    // Output the abort
    println!(
        "{}\n\n{}\n",
        "*** ABORT ***".red().bold().underline(),
        "Software aborted with the following error:".red()
    );
    println!("{}", error.to_string().red());

    // ... and halt with error.
    process::exit(1);
}
