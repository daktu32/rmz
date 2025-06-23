pub mod example;

use crate::cli::{Cli, Commands};

pub fn execute_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Example { verbose } => {
            example::execute(verbose)
        }
    }
}