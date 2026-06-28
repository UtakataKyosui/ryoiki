use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::jj;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub change_id: Option<String>,
    pub description: Option<String>,
    pub is_current: bool,
    pub is_stale: bool,
}

impl WorkspaceInfo {
    /// Load all workspaces from the repo.
    pub fn load_all(repo_root: &Path) -> anyhow::Result<Vec<Self>> {
        let raw = jj::run(repo_root, &["workspace", "list"])?;
        let parsed = jj::parse_workspace_list(&raw);

        let mut workspaces = Vec::new();
        for (name, path_opt, is_current) in parsed {
            let is_stale = path_opt.is_none();
            let path = path_opt.unwrap_or_else(|| repo_root.to_path_buf());

            let (change_id, description) = if !is_stale {
                match jj::workspace_change_info(repo_root, &name) {
                    Some((cid, desc)) => (Some(cid), Some(desc)),
                    None => (None, None),
                }
            } else {
                (None, None)
            };

            workspaces.push(WorkspaceInfo {
                name,
                path,
                change_id,
                description,
                is_current,
                is_stale,
            });
        }

        Ok(workspaces)
    }

    /// Find the current workspace by checking which workspace path is an ancestor
    /// of (or equal to) the cwd.
    pub fn detect_current(workspaces: &mut [WorkspaceInfo], cwd: &Path) {
        // Clear existing is_current (trust jj's output first)
        // If none is marked current, try path comparison.
        let any_current = workspaces.iter().any(|w| w.is_current);
        if any_current {
            return;
        }

        for ws in workspaces.iter_mut() {
            if cwd.starts_with(&ws.path) {
                ws.is_current = true;
                break;
            }
        }
    }

    pub fn display_path(&self) -> String {
        // Try to abbreviate home directory
        if let Some(home) = dirs::home_dir() {
            if let Ok(rel) = self.path.strip_prefix(&home) {
                return format!("~/{}", rel.display());
            }
        }
        self.path.display().to_string()
    }

    pub fn display_name(&self, show_current_marker: bool) -> String {
        if show_current_marker && self.is_current {
            format!("{} (current)", self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn status_str(&self) -> &'static str {
        if self.is_stale {
            "stale"
        } else {
            "active"
        }
    }
}
