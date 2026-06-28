use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct ManifestArgs {
    /// 展開するワークスペース名 (省略時は全ワークスペース)
    pub names: Vec<String>,

    /// tmux セッション名
    #[arg(short, long, value_name = "NAME")]
    pub session: Option<String>,

    /// tmux レイアウト
    #[arg(short, long, value_name = "LAYOUT")]
    pub layout: Option<String>,

    /// ワークスペースごとに tmux window を作成
    #[arg(short = 'w', long)]
    pub window_per_domain: bool,

    /// アタッチせずにセッションだけ作成
    #[arg(long)]
    pub no_attach: bool,
}

pub fn run(
    _args: &ManifestArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("manifest command not yet implemented")
}
