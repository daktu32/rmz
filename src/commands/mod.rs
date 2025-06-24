pub mod delete;
pub mod list;
pub mod purge;
pub mod restore;
pub mod status;

use crate::cli::{Cli, Commands};

pub fn execute_command(cli: Cli) -> anyhow::Result<()> {
    // Initialize colored output based on CLI flags
    #[cfg(feature = "colors")]
    {
        if cli.no_color {
            colored::control::set_override(false);
        }
    }

    match cli.command {
        Commands::Delete {
            paths,
            force,
            dry_run,
            tag,
            interactive,
        } => delete::execute(paths, force, dry_run, tag, interactive, cli.verbose),
        Commands::Restore {
            file,
            id,
            interactive,
            all,
            to,
        } => restore::execute(file, id, interactive, all, to, cli.verbose),
        Commands::List {
            json,
            filter,
            since,
            group_by,
            limit,
        } => list::execute(json, filter, since, group_by, limit, cli.verbose),
        Commands::Status { detailed } => status::execute(detailed, cli.verbose),
        Commands::Purge {
            all,
            days,
            size,
            id,
            interactive,
        } => purge::execute(all, days, size, id, interactive, cli.verbose),
        _ => {
            println!("Command not yet implemented");
            Ok(())
        }
    }
}
