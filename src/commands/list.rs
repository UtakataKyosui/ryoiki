use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct ListArgs {
    /// 出力形式: table / simple / json / porcelain
    #[arg(short, long, value_name = "FMT")]
    pub format: Option<String>,

    /// stale なワークスペースも表示
    #[arg(short, long)]
    pub all: bool,

    /// 現在のワークスペース情報のみ表示
    #[arg(short, long)]
    pub current: bool,
}

pub fn run(
    _args: &ListArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    todo!("list command not yet implemented")
}
