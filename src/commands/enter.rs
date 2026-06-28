use clap::Args;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::{
    config::Config,
    error::RyoikiError,
    hooks::{HookContext, HookRunner},
    jj,
    output::{Printer, emit_cd},
    workspace::WorkspaceInfo,
};

#[derive(Debug, Args)]
pub struct EnterArgs {
    /// 移動先ワークスペース名 (省略時は fzf 起動)
    pub name: Option<String>,

    /// fzf を使わず NAME 引数を必須とする
    #[arg(long)]
    pub no_fzf: bool,
}

pub fn run(
    args: &EnterArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &Path,
    shell_output: bool,
) -> anyhow::Result<()> {
    let workspaces = WorkspaceInfo::load_all(repo_root)?;
    let repo_name = jj::repo_name(repo_root);
    let current_ws = workspaces
        .iter()
        .find(|w| w.is_current)
        .map(|w| w.name.clone())
        .unwrap_or_else(|| "unknown".to_owned());

    let target = resolve_target(args, &workspaces, config)?;

    let hook_runner = HookRunner::new(config, repo_root);

    // 1. pre-enter hook
    hook_runner.run(&HookContext {
        hook_name: "pre-enter",
        workspace_name: &target.name,
        workspace_path: &target.path,
        repo_root,
        repo_name: &repo_name,
        current_workspace: &current_ws,
        change_id: target.change_id.as_deref(),
    })?;

    // 2. zoxide add
    if config.zoxide.enabled {
        let _ = Command::new("zoxide")
            .args(["add", &target.path.to_string_lossy()])
            .status();
    }

    // 3. cd via shell integration
    if shell_output {
        emit_cd(&target.path);
    } else {
        printer.println(&target.path.to_string_lossy());
    }

    // 4. post-enter hook
    hook_runner.run(&HookContext {
        hook_name: "post-enter",
        workspace_name: &target.name,
        workspace_path: &target.path,
        repo_root,
        repo_name: &repo_name,
        current_workspace: &current_ws,
        change_id: target.change_id.as_deref(),
    })?;

    Ok(())
}

fn resolve_target<'a>(
    args: &EnterArgs,
    workspaces: &'a [WorkspaceInfo],
    config: &Config,
) -> anyhow::Result<&'a WorkspaceInfo> {
    if let Some(name) = &args.name {
        // Exact match
        if let Some(ws) = workspaces.iter().find(|w| &w.name == name) {
            return Ok(ws);
        }

        // Prefix match
        let matches: Vec<_> = workspaces
            .iter()
            .filter(|w| w.name.starts_with(name.as_str()))
            .collect();

        match matches.len() {
            0 => {
                return Err(RyoikiError::WorkspaceNotFound(name.clone()).into());
            }
            1 => return Ok(matches[0]),
            _ => {
                // Ambiguous — fall through to fzf if enabled
                if args.no_fzf || !config.fzf.enabled {
                    anyhow::bail!(
                        "ambiguous name \"{name}\": matches {:?}",
                        matches.iter().map(|w| &w.name).collect::<Vec<_>>()
                    );
                }
                return fzf_select(workspaces, config);
            }
        }
    }

    // No name given
    if args.no_fzf || !config.fzf.enabled {
        anyhow::bail!(
            "workspace name required (fzf is disabled, use --no-fzf=false or provide NAME)"
        );
    }

    fzf_select(workspaces, config)
}

fn fzf_select<'a>(
    workspaces: &'a [WorkspaceInfo],
    config: &Config,
) -> anyhow::Result<&'a WorkspaceInfo> {
    // Build fzf input: "name\tpath"
    let input = workspaces
        .iter()
        .map(|w| format!("{}\t{}", w.name, w.path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    let mut fzf_args = vec!["--with-nth=1", "--delimiter=\t"];
    // Add user-configured opts (split on spaces, simple)
    let extra: Vec<&str> = config.fzf.opts.split_whitespace().collect();
    fzf_args.extend(extra.iter().copied());

    if config.fzf.preview {
        let preview_cmd = "echo 'Change: '$(jj log --no-graph -r \"@{1}\" -T 'change_id.short(8)' 2>/dev/null || echo '?')";
        fzf_args.push("--preview");
        fzf_args.push(preview_cmd);
        fzf_args.push("--preview-window");
        fzf_args.push(&config.fzf.preview_window);
    }

    let mut child = Command::new("fzf")
        .args(&fzf_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| RyoikiError::ToolNotFound("fzf".into()))?;

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(RyoikiError::UserCancelled.into());
    }

    let selected = String::from_utf8_lossy(&output.stdout);
    let selected_name = selected.split('\t').next().unwrap_or("").trim().to_owned();

    workspaces
        .iter()
        .find(|w| w.name == selected_name)
        .ok_or_else(|| RyoikiError::WorkspaceNotFound(selected_name).into())
}
