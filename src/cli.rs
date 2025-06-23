use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rmz")]
#[command(about = "The next gen rm command")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Example command")]
    Example {
        #[arg(short, long)]
        verbose: bool,
    },
}