use clap::Args;
use std::path::Path;
use std::process::Command;

use crate::{config::Config, error::RyoikiError, jj, output::Printer, workspace::WorkspaceInfo};

#[derive(Debug, Args)]
pub struct ManifestArgs {
    /// 展開するワークスペース名 (省略時は全ワークスペース)
    pub names: Vec<String>,

    /// tmux セッション名 (デフォルト: リポジトリ名)
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
    args: &ManifestArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
) -> anyhow::Result<()> {
    // Verify tmux is installed
    if Command::new("tmux").arg("-V").output().is_err() {
        printer.error("tmux is not installed.");
        printer.hint("Install tmux to use the manifest command.");
        return Err(RyoikiError::ToolNotFound("tmux".into()).into());
    }

    let repo_name = jj::repo_name(repo_root);
    let session_name = args
        .session
        .clone()
        .unwrap_or_else(|| config.tmux.session_format.replace("{repo}", &repo_name));

    let layout = args
        .layout
        .as_deref()
        .unwrap_or(&config.tmux.default_layout)
        .to_owned();

    let all_workspaces = WorkspaceInfo::load_all(repo_root)?;
    let targets: Vec<&WorkspaceInfo> = if args.names.is_empty() {
        all_workspaces.iter().filter(|w| !w.is_stale).collect()
    } else {
        args.names
            .iter()
            .filter_map(|name| all_workspaces.iter().find(|w| &w.name == name))
            .collect()
    };

    if targets.is_empty() {
        anyhow::bail!("no workspaces to manifest");
    }

    // Check if session already exists
    let session_exists = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if session_exists && config.tmux.auto_attach {
        printer.println(&format!(
            "Session \"{}\" already exists, attaching…",
            session_name
        ));
        if !args.no_attach {
            Command::new("tmux")
                .args(["attach-session", "-t", &session_name])
                .status()?;
        }
        return Ok(());
    }

    // Create new session with first workspace
    let first = targets[0];
    Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &session_name,
            "-c",
            &first.path.to_string_lossy(),
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("tmux new-session failed: {e}"))?;

    if args.window_per_domain {
        // Rename first window to first workspace name
        Command::new("tmux")
            .args([
                "rename-window",
                "-t",
                &format!("{}:0", session_name),
                &first.name,
            ])
            .status()?;

        // Create a window per remaining workspace
        for ws in &targets[1..] {
            Command::new("tmux")
                .args([
                    "new-window",
                    "-t",
                    &session_name,
                    "-n",
                    &ws.name,
                    "-c",
                    &ws.path.to_string_lossy(),
                ])
                .status()?;
        }
    } else {
        // Split panes in the first window
        for ws in &targets[1..] {
            Command::new("tmux")
                .args([
                    "split-window",
                    "-t",
                    &session_name,
                    "-c",
                    &ws.path.to_string_lossy(),
                ])
                .status()?;
        }

        // Apply layout
        Command::new("tmux")
            .args(["select-layout", "-t", &session_name, &layout])
            .status()?;
    }

    printer.success(&format!(
        "Session \"{}\" created with {} domain(s).",
        session_name,
        targets.len()
    ));

    if !args.no_attach {
        Command::new("tmux")
            .args(["attach-session", "-t", &session_name])
            .status()?;
    }

    Ok(())
}
