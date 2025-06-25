pub mod completions;
pub mod config;
pub mod delete;
pub mod doctor;
pub mod list;
pub mod log;
pub mod protect;
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
            recursive,
        } => delete::execute(paths, force, dry_run, tag, interactive, recursive, cli.verbose),
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
        Commands::Log {
            detailed,
            operation,
            since,
        } => log::execute(detailed, operation, since, cli.verbose),
        Commands::Protect { action } => protect::execute(action, cli.verbose),
        Commands::Config { action } => config::execute(action, cli.verbose),
        Commands::Doctor { check, fix, verbose, force } => {
            // Convert CLI DiagnosticCheck to doctor module's DiagnosticCheck
            let doctor_check = check.map(|c| match c {
                crate::cli::DiagnosticCheck::TrashZone => crate::commands::doctor::DiagnosticCheck::TrashZone,
                crate::cli::DiagnosticCheck::Metadata => crate::commands::doctor::DiagnosticCheck::Metadata, 
                crate::cli::DiagnosticCheck::Permissions => crate::commands::doctor::DiagnosticCheck::Permissions,
                crate::cli::DiagnosticCheck::Config => crate::commands::doctor::DiagnosticCheck::Config,
                crate::cli::DiagnosticCheck::Dependencies => crate::commands::doctor::DiagnosticCheck::Dependencies,
                crate::cli::DiagnosticCheck::All => crate::commands::doctor::DiagnosticCheck::All,
            });
            crate::commands::doctor::execute(doctor_check, fix, verbose, force)
        },
        Commands::Completions { shell } => completions::execute(shell, cli.verbose),
    }
}
