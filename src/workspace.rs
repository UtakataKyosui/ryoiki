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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make(name: &str, path: &str, is_current: bool, is_stale: bool) -> WorkspaceInfo {
        WorkspaceInfo {
            name: name.to_owned(),
            path: PathBuf::from(path),
            change_id: None,
            description: None,
            is_current,
            is_stale,
        }
    }

    #[test]
    fn status_str_active() {
        assert_eq!(make("main", "/repo", false, false).status_str(), "active");
    }

    #[test]
    fn status_str_stale() {
        assert_eq!(make("old", "/repo", false, true).status_str(), "stale");
    }

    #[test]
    fn display_name_current_with_marker() {
        let ws = make("main", "/repo", true, false);
        assert_eq!(ws.display_name(true), "main (current)");
    }

    #[test]
    fn display_name_current_without_marker() {
        let ws = make("main", "/repo", true, false);
        assert_eq!(ws.display_name(false), "main");
    }

    #[test]
    fn display_name_non_current() {
        let ws = make("feature", "/repo", false, false);
        // Even with show_current_marker=true, non-current should not get the suffix
        assert_eq!(ws.display_name(true), "feature");
    }

    #[test]
    fn detect_current_falls_back_to_path() {
        let mut workspaces = vec![
            make("main", "/repo", false, false),
            make("feature", "/repo-feature", false, false),
        ];
        WorkspaceInfo::detect_current(&mut workspaces, Path::new("/repo-feature/src"));
        assert!(workspaces[1].is_current);
        assert!(!workspaces[0].is_current);
    }

    #[test]
    fn detect_current_skips_when_already_marked() {
        let mut workspaces = vec![
            make("main", "/repo", true, false),
            make("feature", "/repo-feature", false, false),
        ];
        // Should not change anything since one is already marked current
        WorkspaceInfo::detect_current(&mut workspaces, Path::new("/repo-feature"));
        assert!(workspaces[0].is_current);
        assert!(!workspaces[1].is_current);
    }
}
