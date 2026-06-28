use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct RenameArgs {
    /// 変更前のワークスペース名
    pub old_name: String,

    /// 変更後のワークスペース名
    pub new_name: String,
}

pub fn run(
    _args: &RenameArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("rename command not yet implemented")
}
