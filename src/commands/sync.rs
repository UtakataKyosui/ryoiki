use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// 対象ワークスペース名 (省略時は全ワークスペース)
    pub names: Vec<String>,

    /// 実行するフック名を指定
    #[arg(long, value_name = "HOOK")]
    pub hook: Option<String>,

    /// 実際には実行せず、何を行うかのみ表示
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// ワークスペースを並行処理
    #[arg(short, long)]
    pub parallel: bool,
}

pub fn run(
    _args: &SyncArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("sync command not yet implemented")
}
