use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct StatusArgs {}

pub fn run(
    _args: &StatusArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("status command not yet implemented")
}
