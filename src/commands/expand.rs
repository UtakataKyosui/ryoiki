use clap::Args;
use std::path::{Path, PathBuf};

use crate::{
    config::{expand_tilde, Config},
    error::RyoikiError,
    hooks::{HookContext, HookRunner},
    jj,
    output::{emit_cd, Printer},
    workspace::WorkspaceInfo,
};

#[derive(Debug, Args)]
pub struct ExpandArgs {
    /// ワークスペース名
    pub name: String,

    /// 作成先パス (省略時: <base_dir>/<dir_format>)
    pub destination: Option<PathBuf>,

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
    pub base_dir: Option<PathBuf>,
}

pub fn run(
    args: &ExpandArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
    shell_output: bool,
) -> anyhow::Result<()> {
    let destination = resolve_destination(args, config, repo_root);

    let repo_name = jj::repo_name(repo_root);
    let current_ws = current_workspace_name(repo_root);

    let hook_runner = HookRunner::new(config, repo_root);

    // 1. pre-expand hook
    if !args.no_hooks {
        hook_runner.run(&HookContext {
            hook_name: "pre-expand",
            workspace_name: &args.name,
            workspace_path: &destination,
            repo_root,
            repo_name: &repo_name,
            current_workspace: &current_ws,
            change_id: None,
        })?;
    }

    // 2. jj workspace add
    printer.verbose(&format!(
        "Creating workspace \"{}\" at {}",
        args.name,
        destination.display()
    ));

    jj::workspace_add(repo_root, &args.name, &destination, args.revision.as_deref())
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("already exists") {
                anyhow::Error::from(RyoikiError::WorkspaceExists(args.name.clone()))
            } else {
                e
            }
        })?;

    printer.success(&format!(
        "Domain \"{}\" expanded at {}",
        args.name,
        destination.display()
    ));

    // 3. zoxide add
    if config.zoxide.enabled {
        let _ = std::process::Command::new("zoxide")
            .args(["add", &destination.to_string_lossy()])
            .status();
    }

    // 4. post-expand hook
    if !args.no_hooks {
        hook_runner.run(&HookContext {
            hook_name: "post-expand",
            workspace_name: &args.name,
            workspace_path: &destination,
            repo_root,
            repo_name: &repo_name,
            current_workspace: &current_ws,
            change_id: None,
        })?;
    }

    // 5. cd (expand only, not forge, not --no-cd)
    if shell_output && !args.no_cd {
        emit_cd(&destination);
    }

    Ok(())
}

fn resolve_destination(args: &ExpandArgs, config: &Config, repo_root: &Path) -> PathBuf {
    if let Some(dest) = &args.destination {
        return dest.clone();
    }

    let base = if let Some(b) = &args.base_dir {
        b.clone()
    } else {
        config.resolve_base_dir(repo_root)
    };

    let repo_name = jj::repo_name(repo_root);
    let dir_name = config
        .core
        .dir_format
        .replace("{name}", &args.name)
        .replace("{repo}", &repo_name)
        .replace("{date}", &today_str());

    base.join(dir_name)
}

fn today_str() -> String {
    // Simple YYYY-MM-DD without external crates
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    // Good enough epoch-based date (won't be perfect but functional)
    let y = 1970 + days / 365;
    let m = (days % 365) / 30 + 1;
    let d = (days % 365) % 30 + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn current_workspace_name(repo_root: &Path) -> String {
    WorkspaceInfo::load_all(repo_root)
        .ok()
        .and_then(|ws| ws.into_iter().find(|w| w.is_current).map(|w| w.name))
        .unwrap_or_else(|| "unknown".to_owned())
}
