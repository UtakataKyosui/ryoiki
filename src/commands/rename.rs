use clap::Args;

use crate::{config::Config, jj, output::Printer};

#[derive(Debug, Args)]
pub struct RenameArgs {
    /// 変更前のワークスペース名
    pub old_name: String,

    /// 変更後のワークスペース名
    pub new_name: String,
}

pub fn run(
    args: &RenameArgs,
    _config: &Config,
    printer: &Printer,
    repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    jj::workspace_rename(repo_root, &args.old_name, &args.new_name)?;

    printer.success(&format!(
        "Domain \"{}\" renamed to \"{}\".",
        args.old_name, args.new_name
    ));

    Ok(())
}
