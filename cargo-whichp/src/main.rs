#![warn(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

mod cli;

use crate::cli::{Cli, WhichpArgs};
use clap::Parser;
use which_problem::Which;

const COMMAND_SUCCESS: i32 = 0;
const COMMAND_ERRORED: i32 = -1;

fn main() {
    match Cli::parse() {
        Cli::Whichp(args) => handle_whichp(args),
    }
}

fn handle_whichp(args: WhichpArgs) {
    let path_env = match args.path {
        Some(p) => Some(p),
        None => Which::default().path_env,
    };

    let which = Which {
        program: args.program,
        cwd: args.cwd,
        path_env,
        guess_limit: args.suggest.unwrap_or(Which::default().guess_limit),
    };
    match which.diagnose() {
        Ok(program) => {
            println!("{program}");
            std::process::exit(COMMAND_SUCCESS);
        }
        Err(error) => {
            eprintln!("Error, cannot continue");
            eprintln!("Details: {error}");

            std::process::exit(COMMAND_ERRORED);
        }
    };
}
