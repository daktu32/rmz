use clap::Parser;
use rmz::cli::Cli;
use rmz::commands::execute_command;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute_command(cli)
}