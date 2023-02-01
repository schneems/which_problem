use clap::Parser;
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser, Debug)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub(crate) enum Cli {
    // Name of command i.e. 'whichp' is based on the name of this varient
    Whichp(WhichpArgs),
}

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct WhichpArgs {
    pub(crate) program: OsString,

    #[arg(short, long)]
    pub(crate) cwd: Option<PathBuf>,

    #[arg(short, long)]
    pub(crate) path: Option<OsString>,

    #[arg(short, long)]
    pub(crate) suggest: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        // Trigger Clap's internal assertions that validate the command configuration.
        Cli::command().debug_assert();
    }
}
