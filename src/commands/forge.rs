use clap::Args;
use std::path::Path;

use crate::{
    commands::expand::{ExpandArgs, run as expand_run},
    config::Config,
    output::Printer,
};

#[derive(Debug, Args)]
pub struct ForgeArgs {
    /// ワークスペース名
    pub name: String,

    /// 作成先パス (省略時: <base_dir>/<dir_format>)
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

    /// フックを実行しない
    #[arg(long)]
    pub no_hooks: bool,

    /// 作成先の基底ディレクトリを上書き
    #[arg(long, value_name = "PATH")]
    pub base_dir: Option<std::path::PathBuf>,
}

pub fn run(
    args: &ForgeArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
) -> anyhow::Result<()> {
    // forge = expand --no-cd (no shell output)
    let expand_args = ExpandArgs {
        name: args.name.clone(),
        destination: args.destination.clone(),
        revision: args.revision.clone(),
        message: args.message.clone(),
        sparse: args.sparse.clone(),
        no_cd: true,
        no_hooks: args.no_hooks,
        base_dir: args.base_dir.clone(),
    };

    expand_run(&expand_args, config, printer, repo_root, false)
}
