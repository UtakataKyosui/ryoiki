use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct CollapseArgs {
    /// 削除するワークスペース名 (省略時は fzf で複数選択)
    pub names: Vec<String>,

    /// 確認プロンプトをスキップ
    #[arg(short, long)]
    pub force: bool,

    /// ディレクトリをディスクに残す
    #[arg(long)]
    pub keep_dir: bool,

    /// fzf を使わない
    #[arg(long)]
    pub no_fzf: bool,
}

pub fn run(
    _args: &CollapseArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("collapse command not yet implemented")
}
