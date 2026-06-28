use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct EnterArgs {
    /// 移動先ワークスペース名 (省略時は fzf 起動)
    pub name: Option<String>,

    /// fzf を使わず NAME 引数を必須とする
    #[arg(long)]
    pub no_fzf: bool,
}

pub fn run(
    _args: &EnterArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
    _shell_output: bool,
) -> anyhow::Result<()> {
    todo!("enter command not yet implemented")
}
