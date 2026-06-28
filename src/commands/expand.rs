use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct ExpandArgs {
    /// ワークスペース名
    pub name: String,

    /// 作成先パス
    pub destination: Option<std::path::PathBuf>,

    /// 作業コピーコミットの親リビジョン
    #[arg(short, long, value_name = "REVSET")]
    pub revision: Option<String>,

    /// 初期コミットのメッセージ
    #[arg(short, long, value_name = "MSG")]
    pub message: Option<String>,

    /// スパースパターン: copy / full / empty
    #[arg(short, long, value_name = "MODE", default_value = "copy")]
    pub sparse: String,

    /// 移動せず作成のみ
    #[arg(long)]
    pub no_cd: bool,

    /// フックを実行しない
    #[arg(long)]
    pub no_hooks: bool,

    /// 作成先の基底ディレクトリを上書き
    #[arg(long, value_name = "PATH")]
    pub base_dir: Option<std::path::PathBuf>,
}

pub fn run(
    _args: &ExpandArgs,
    _config: &Config,
    _printer: &Printer,
    _repo_root: &std::path::Path,
    _shell_output: bool,
) -> anyhow::Result<()> {
    todo!("expand command not yet implemented")
}
