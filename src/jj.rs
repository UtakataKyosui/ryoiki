use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::RyoikiError;

/// Run a jj subcommand from the given directory, returning stdout.
pub fn run(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("jj")
        .args(args)
        .current_dir(cwd)
        .env("JJ_EDITOR", "true") // prevent interactive editor from opening
        .output()
        .context("failed to execute jj — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(RyoikiError::JjFailed(stderr).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run a jj subcommand, returning (stdout, success).  Stderr is swallowed.
pub fn run_silent(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("jj")
        .args(args)
        .current_dir(cwd)
        .env("JJ_EDITOR", "true")
        .output()
        .context("failed to execute jj")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Detect the jj repository root starting from `start`.
pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let output = Command::new("jj")
        .args(["root"])
        .current_dir(start)
        .output()
        .context("failed to execute jj — is it installed?")?;

    if output.status.success() {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok(PathBuf::from(root))
    } else {
        anyhow::bail!("not inside a jj repository (run from inside one, or use --repository)")
    }
}

/// Parse `jj workspace list` output into (name, path_or_stale, is_current) tuples.
/// jj output format:
///   name: /path/to/workspace
///   name: /path/to/workspace (current)
pub fn parse_workspace_list(raw: &str) -> Vec<(String, Option<PathBuf>, bool)> {
    let mut workspaces = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split on first ':'
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };

        let name = name.trim().to_owned();
        let rest = rest.trim();

        let (path_str, is_current) = if let Some(p) = rest.strip_suffix(" (current)") {
            (p.trim(), true)
        } else {
            (rest, false)
        };

        let path = if path_str.is_empty() || path_str == "(stale)" {
            None
        } else {
            Some(PathBuf::from(path_str))
        };

        workspaces.push((name, path, is_current));
    }

    workspaces
}

/// Get the short change ID and description for a workspace revision.
/// Returns None if the workspace is stale or the query fails.
pub fn workspace_change_info(
    repo_root: &Path,
    workspace_name: &str,
) -> Option<(String, String)> {
    let revset = format!("@{}", workspace_name);
    let template = "change_id.short(8) ++ \"\\t\" ++ description.first_line() ++ \"\\n\"";

    let output = Command::new("jj")
        .args(["log", "--no-graph", "-r", &revset, "-T", template])
        .current_dir(repo_root)
        .env("JJ_EDITOR", "true")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let line = stdout.lines().next()?;
    let (change_id, desc) = line.split_once('\t')?;

    Some((change_id.trim().to_owned(), desc.trim().to_owned()))
}

/// Return the repo name derived from the root directory name.
pub fn repo_name(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo")
        .to_owned()
}

pub fn workspace_add(
    repo_root: &Path,
    name: &str,
    destination: &Path,
    revision: Option<&str>,
) -> Result<()> {
    let mut args = vec!["workspace", "add", "--name", name];
    let rev_owned;
    if let Some(rev) = revision {
        rev_owned = rev.to_owned();
        args.push("--revision");
        args.push(&rev_owned);
    }
    let dest = destination.to_string_lossy().into_owned();
    args.push(&dest);

    run(repo_root, &args)?;
    Ok(())
}

pub fn workspace_forget(repo_root: &Path, name: &str) -> Result<()> {
    run(repo_root, &["workspace", "forget", name])?;
    Ok(())
}

pub fn workspace_rename(repo_root: &Path, old: &str, new: &str) -> Result<()> {
    run(repo_root, &["workspace", "rename", old, new])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let raw = "main: /home/user/repo\nfeature: /home/user/repo-feature (current)\n";
        let ws = parse_workspace_list(raw);
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[0].0, "main");
        assert_eq!(ws[0].1, Some(PathBuf::from("/home/user/repo")));
        assert!(!ws[0].2, "main should not be current");
        assert_eq!(ws[1].0, "feature");
        assert_eq!(ws[1].1, Some(PathBuf::from("/home/user/repo-feature")));
        assert!(ws[1].2, "feature should be current");
    }

    #[test]
    fn parse_stale() {
        let raw = "old: (stale)\n";
        let ws = parse_workspace_list(raw);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].0, "old");
        assert_eq!(ws[0].1, None, "stale workspace should have no path");
        assert!(!ws[0].2);
    }

    #[test]
    fn parse_empty() {
        assert!(parse_workspace_list("").is_empty());
        assert!(parse_workspace_list("  \n\n  ").is_empty());
    }

    #[test]
    fn parse_single_current() {
        let raw = "default: /repo (current)";
        let ws = parse_workspace_list(raw);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].0, "default");
        assert_eq!(ws[0].1, Some(PathBuf::from("/repo")));
        assert!(ws[0].2);
    }

    #[test]
    fn parse_ignores_lines_without_colon() {
        let raw = "no-colon-here\nmain: /repo\n";
        let ws = parse_workspace_list(raw);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].0, "main");
    }

    #[test]
    fn parse_path_with_colon() {
        // Paths like C:\Users\... on Windows — split only on first ':'
        let raw = "default: C:\\Users\\user\\repo (current)";
        let ws = parse_workspace_list(raw);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].1, Some(PathBuf::from("C:\\Users\\user\\repo")));
        assert!(ws[0].2);
    }
}
