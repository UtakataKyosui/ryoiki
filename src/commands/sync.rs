use clap::Args;
use std::path::Path;

use crate::{
    config::Config,
    hooks::{HookContext, HookRunner},
    jj,
    output::Printer,
    workspace::WorkspaceInfo,
};

const ALL_HOOKS: &[&str] = &[
    "pre-sync",
    "pre-expand",
    "post-expand",
    "pre-enter",
    "post-enter",
    "pre-collapse",
    "post-collapse",
    "post-sync",
];

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
    args: &SyncArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
) -> anyhow::Result<()> {
    let all_workspaces = WorkspaceInfo::load_all(repo_root)?;
    let repo_name = jj::repo_name(repo_root);
    let current_ws = all_workspaces
        .iter()
        .find(|w| w.is_current)
        .map(|w| w.name.clone())
        .unwrap_or_else(|| "unknown".to_owned());

    let targets: Vec<&WorkspaceInfo> = if args.names.is_empty() {
        all_workspaces.iter().collect()
    } else {
        args.names
            .iter()
            .filter_map(|name| all_workspaces.iter().find(|w| &w.name == name))
            .collect()
    };

    let hooks_to_run: Vec<&str> = if let Some(h) = &args.hook {
        vec![h.as_str()]
    } else {
        ALL_HOOKS.to_vec()
    };

    let hook_runner = HookRunner::new(config, repo_root);

    // pre-sync
    if hooks_to_run.contains(&"pre-sync") {
        for ws in &targets {
            if args.dry_run {
                printer.println(&format!("[dry-run] pre-sync for \"{}\"", ws.name));
                continue;
            }
            hook_runner.run(&HookContext {
                hook_name: "pre-sync",
                workspace_name: &ws.name,
                workspace_path: &ws.path,
                repo_root,
                repo_name: &repo_name,
                current_workspace: &current_ws,
                change_id: ws.change_id.as_deref(),
            })?;
        }
    }

    // Per-workspace hooks (excluding pre/post-sync)
    let per_ws_hooks: Vec<&str> = hooks_to_run
        .iter()
        .copied()
        .filter(|h| *h != "pre-sync" && *h != "post-sync")
        .collect();

    for ws in &targets {
        for hook in &per_ws_hooks {
            if args.dry_run {
                printer.println(&format!("[dry-run] {} for \"{}\"", hook, ws.name));
                continue;
            }
            printer.verbose(&format!("Running {} for \"{}\"", hook, ws.name));
            hook_runner.run(&HookContext {
                hook_name: hook,
                workspace_name: &ws.name,
                workspace_path: &ws.path,
                repo_root,
                repo_name: &repo_name,
                current_workspace: &current_ws,
                change_id: ws.change_id.as_deref(),
            })?;
        }
    }

    // post-sync
    if hooks_to_run.contains(&"post-sync") {
        for ws in &targets {
            if args.dry_run {
                printer.println(&format!("[dry-run] post-sync for \"{}\"", ws.name));
                continue;
            }
            hook_runner.run(&HookContext {
                hook_name: "post-sync",
                workspace_name: &ws.name,
                workspace_path: &ws.path,
                repo_root,
                repo_name: &repo_name,
                current_workspace: &current_ws,
                change_id: ws.change_id.as_deref(),
            })?;
        }
    }

    if !args.dry_run {
        printer.success(&format!("Sync complete for {} domain(s).", targets.len()));
    }

    Ok(())
}
