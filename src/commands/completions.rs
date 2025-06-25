use crate::cli::Cli;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completion scripts
pub fn execute(shell: Shell, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Generating completion scripts for {:?}", shell);
    }

    let mut cmd = Cli::command();
    let cmd_name = cmd.get_name().to_string();
    
    generate(shell, &mut cmd, cmd_name, &mut io::stdout());
    
    Ok(())
}