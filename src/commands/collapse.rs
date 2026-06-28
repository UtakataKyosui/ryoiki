use clap::Args;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::{
    config::Config,
    error::RyoikiError,
    hooks::{HookContext, HookRunner},
    jj,
    output::Printer,
    workspace::WorkspaceInfo,
};

#[derive(Debug, Args)]
pub struct CollapseArgs {
    /// 削除するワークスペース名 (省略時は fzf で複数選択)
    pub names: Vec<String>,

    /// 確認プロンプトをスキップ
    #[arg(short, long)]
    pub force: bool,

    /// ディレクトリをディスクに残す (jj workspace forget のみ)
    #[arg(long)]
    pub keep_dir: bool,

    /// fzf を使わない
    #[arg(long)]
    pub no_fzf: bool,
}

pub fn run(
    args: &CollapseArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
) -> anyhow::Result<()> {
    let workspaces = WorkspaceInfo::load_all(repo_root)?;
    let repo_name = jj::repo_name(repo_root);
    let current_ws = workspaces
        .iter()
        .find(|w| w.is_current)
        .map(|w| w.name.clone())
        .unwrap_or_else(|| "unknown".to_owned());

    let targets: Vec<&WorkspaceInfo> = if args.names.is_empty() {
        // fzf multi-select
        if args.no_fzf || !config.fzf.enabled {
            anyhow::bail!("workspace name required (fzf is disabled)");
        }
        fzf_multi_select(&workspaces, config)?
    } else {
        args.names
            .iter()
            .map(|name| {
                workspaces
                    .iter()
                    .find(|w| &w.name == name)
                    .ok_or_else(|| RyoikiError::WorkspaceNotFound(name.clone()).into())
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    };

    if targets.is_empty() {
        return Ok(());
    }

    let hook_runner = HookRunner::new(config, repo_root);

    for ws in targets {
        // Cannot collapse current workspace
        if ws.is_current {
            printer.error(&format!("Cannot collapse current domain \"{}\".", ws.name));
            printer.hint("Enter another domain first with `ryoiki enter <name>`.");
            return Err(RyoikiError::CannotCollapseCurrentDomain(ws.name.clone()).into());
        }

        // Confirm
        if !args.force {
            eprint!("Collapse domain \"{}\"? [y/N] ", ws.name);
            std::io::stderr().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                printer.println(&format!("Skipping \"{}\".", ws.name));
                continue;
            }
        }

        // 1. pre-collapse hook
        hook_runner.run(&HookContext {
            hook_name: "pre-collapse",
            workspace_name: &ws.name,
            workspace_path: &ws.path,
            repo_root,
            repo_name: &repo_name,
            current_workspace: &current_ws,
            change_id: ws.change_id.as_deref(),
        })?;

        // 2. jj workspace forget
        jj::workspace_forget(repo_root, &ws.name)?;

        // 3. Remove directory unless --keep-dir
        if !args.keep_dir && ws.path.exists() {
            std::fs::remove_dir_all(&ws.path)
                .map_err(|e| anyhow::anyhow!("failed to remove {}: {}", ws.path.display(), e))?;
        }

        // 4. zoxide remove (failure is just a warning)
        if config.zoxide.enabled {
            let result = Command::new("zoxide")
                .args(["remove", &ws.path.to_string_lossy()])
                .status();
            if let Ok(s) = result
                && !s.success()
            {
                printer.warning(&format!("zoxide remove failed for {}", ws.path.display()));
            }
        }

        // 5. post-collapse hook
        hook_runner.run(&HookContext {
            hook_name: "post-collapse",
            workspace_name: &ws.name,
            workspace_path: &ws.path,
            repo_root,
            repo_name: &repo_name,
            current_workspace: &current_ws,
            change_id: ws.change_id.as_deref(),
        })?;

        printer.success(&format!("Domain \"{}\" collapsed.", ws.name));
    }

    Ok(())
}

fn fzf_multi_select<'a>(
    workspaces: &'a [WorkspaceInfo],
    config: &Config,
) -> anyhow::Result<Vec<&'a WorkspaceInfo>> {
    let input = workspaces
        .iter()
        .filter(|w| !w.is_current) // don't offer current workspace for deletion
        .map(|w| format!("{}\t{}", w.name, w.path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    let mut fzf_args = vec!["--multi", "--with-nth=1", "--delimiter=\t"];
    let extra: Vec<&str> = config.fzf.opts.split_whitespace().collect();
    fzf_args.extend(extra.iter().copied());

    let mut child = Command::new("fzf")
        .args(&fzf_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| RyoikiError::ToolNotFound("fzf".into()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(RyoikiError::UserCancelled.into());
    }

    let selected = String::from_utf8_lossy(&output.stdout);
    let names: Vec<&str> = selected
        .lines()
        .filter_map(|line| line.split('\t').next())
        .collect();

    Ok(workspaces
        .iter()
        .filter(|w| names.contains(&w.name.as_str()))
        .collect())
}
