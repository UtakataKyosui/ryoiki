use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ryoiki",
    version,
    about = "Jujutsu ワークスペースを管理する CLI ツール (領域展開)",
    long_about = None,
)]
pub struct Cli {
    /// カラー出力を無効化
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 主要出力以外を抑制
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// 詳細ログを出力
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// 設定ファイルのパスを明示指定
    #[arg(short = 'C', long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// jj リポジトリのパスを指定 (デフォルト: 自動検出)
    #[arg(short = 'R', long, global = true, value_name = "PATH")]
    pub repository: Option<PathBuf>,

    /// シェル統合から使用される内部フラグ
    #[arg(long, global = true, hide = true)]
    pub shell_output: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// ワークスペースを新規作成して移動する (シェル統合必要)
    #[command(alias = "add")]
    Expand(crate::commands::expand::ExpandArgs),

    /// ワークスペースを作成するが移動しない
    #[command(alias = "create")]
    Forge(crate::commands::forge::ForgeArgs),

    /// すべてのワークスペースを一覧表示する
    #[command(alias = "ls")]
    List(crate::commands::list::ListArgs),

    /// ワークスペースに移動する (シェル統合必要)
    #[command(alias = "cd")]
    Enter(crate::commands::enter::EnterArgs),

    /// ワークスペースを削除する
    #[command(alias = "rm")]
    Collapse(crate::commands::collapse::CollapseArgs),

    /// 全ワークスペースにフックを再適用する
    Sync(crate::commands::sync::SyncArgs),

    /// 複数のワークスペースを tmux セッションとして展開する
    #[command(alias = "open")]
    Manifest(crate::commands::manifest::ManifestArgs),

    /// ワークスペース名を変更する
    Rename(crate::commands::rename::RenameArgs),

    /// 現在のワークスペース情報とサマリーを表示する
    #[command(alias = "st")]
    Status(crate::commands::status::StatusArgs),

    /// シェル統合スクリプトを標準出力に出力する
    Init(crate::commands::init::InitArgs),
}
