use clap::Parser;
use std::process;
use colored::*;

use crate::commands::command::Command;
use crate::error::jaws_error::JawsError;

mod commands;
mod aws_handler;
mod error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    /// Output wider, more detailed data.  May cause slower execution in some cases.
    #[clap(short, long, default_value_t = false, )]
    wide: bool,

    /// A subcommand to run
    #[command(subcommand)]
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

    /// Translate an instance ID to its public IP address (if available)
    TR,

    /// Gets the caller identity from the Security Token Service
    GCI,
}


// Main: starts here. We need tokio because the AWS libraries need it.
#[tokio::main]
async fn main() {
    // Parse options
    let options = Options::parse();

    // Switch based on the selected subcommand.

    let command: Option<&dyn Command> = match &options.subcommand {
        SubCommands::EC2 => Some(&commands::ec2::EC2Command {} as &dyn Command),
        SubCommands::GCI => Some(&commands::gci::GCICommand {} as &dyn Command),
        _ => None
    };

    match command {
        Some(c) => {
            match c.run(&options).await {
                Ok(_) => {} // Success - command ran to completion
                Err(e) => handle_and_abort(e)
            }
        }
        None => handle_and_abort(JawsError::new(format!("Command '{:?}' not found", options.subcommand))),
    }
}

fn handle_and_abort(error: JawsError) {
    println!("{}\n\n{}\n", "*** ABORT ***".red().bold().underline(),
        "Software aborted with the following error:".red());
    println!("{}", error.to_string().red());
    process::exit(1);
}

