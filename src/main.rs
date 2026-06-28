mod cli;
mod commands;
mod config;
mod error;
mod hooks;
mod jj;
mod output;
mod workspace;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use cli::{Cli, Commands};
use config::Config;
use error::RyoikiError;
use output::{ColorMode, Printer};

fn main() {
    let cli = Cli::parse();

    let color_mode = if cli.no_color {
        ColorMode::Never
    } else {
        ColorMode::Auto
    };

    let printer = Printer::new(color_mode, cli.quiet, cli.verbose);

    match run(&cli, &printer) {
        Ok(()) => {}
        Err(e) => {
            if let Some(re) = e.downcast_ref::<RyoikiError>() {
                printer.error(&e.to_string());
                std::process::exit(re.exit_code());
            } else {
                printer.error(&e.to_string());
                std::process::exit(1);
            }
        }
    }
}

fn run(cli: &Cli, printer: &Printer) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = if let Some(repo) = &cli.repository {
        repo.clone()
    } else {
        match jj::find_repo_root(&cwd) {
            Ok(root) => root,
            Err(e) => {
                if matches!(&cli.command, Commands::Init(_)) {
                    PathBuf::new()
                } else {
                    return Err(e);
                }
            }
        }
    };

    let config = Config::load(
        cli.config.as_deref(),
        Some(&repo_root).filter(|p| !p.as_os_str().is_empty()).map(|p| p.as_path()),
    )?;

    match &cli.command {
        Commands::List(args) => commands::list::run(args, &config, printer, &repo_root),
        Commands::Status(args) => commands::status::run(args, &config, printer, &repo_root),
        Commands::Rename(args) => commands::rename::run(args, &config, printer, &repo_root),
        Commands::Expand(args) => {
            commands::expand::run(args, &config, printer, &repo_root, cli.shell_output)
        }
        Commands::Forge(args) => commands::forge::run(args, &config, printer, &repo_root),
        Commands::Enter(args) => {
            commands::enter::run(args, &config, printer, &repo_root, cli.shell_output)
        }
        Commands::Collapse(args) => commands::collapse::run(args, &config, printer, &repo_root),
        Commands::Sync(args) => commands::sync::run(args, &config, printer, &repo_root),
        Commands::Manifest(args) => commands::manifest::run(args, &config, printer, &repo_root),
        Commands::Init(args) => commands::init::run(args, &config, printer),
    }
}
