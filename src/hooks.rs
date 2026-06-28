use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;
use crate::error::RyoikiError;

pub struct HookContext<'a> {
    pub hook_name: &'a str,
    pub workspace_name: &'a str,
    pub workspace_path: &'a Path,
    pub repo_root: &'a Path,
    pub repo_name: &'a str,
    pub current_workspace: &'a str,
    pub change_id: Option<&'a str>,
}

pub struct HookRunner<'a> {
    config: &'a Config,
    repo_root: &'a Path,
}

impl<'a> HookRunner<'a> {
    pub fn new(config: &'a Config, repo_root: &'a Path) -> Self {
        Self { config, repo_root }
    }

    pub fn run(&self, ctx: &HookContext<'_>) -> Result<()> {
        if !self.config.hooks.enabled {
            return Ok(());
        }

        let scripts = self.discover(ctx.hook_name);
        for script in scripts {
            self.exec_script(&script, ctx)?;
        }

        Ok(())
    }

    fn discover(&self, hook_name: &str) -> Vec<PathBuf> {
        let mut scripts = Vec::new();

        // 1. Repo-local hooks first
        let local_hook = self.repo_root.join(".ryoiki").join("hooks").join(hook_name);
        if local_hook.exists() {
            scripts.push(local_hook);
        }

        // 2. Global hooks second
        let global_hook = self.config.global_hook_dir().join(hook_name);
        if global_hook.exists() {
            scripts.push(global_hook);
        }

        scripts
    }

    fn exec_script(&self, script: &Path, ctx: &HookContext<'_>) -> Result<()> {
        let mut cmd = Command::new(script);

        cmd.current_dir(ctx.workspace_path)
            .env("RYOIKI_HOOK_NAME", ctx.hook_name)
            .env("RYOIKI_WORKSPACE_NAME", ctx.workspace_name)
            .env("RYOIKI_WORKSPACE_PATH", ctx.workspace_path)
            .env("RYOIKI_REPO_ROOT", ctx.repo_root)
            .env("RYOIKI_REPO_NAME", ctx.repo_name)
            .env("RYOIKI_CURRENT_WORKSPACE", ctx.current_workspace);

        if let Some(cid) = ctx.change_id {
            cmd.env("RYOIKI_JJ_CHANGE_ID", cid);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("failed to execute hook {}: {}", script.display(), e))?;

        if !status.success() {
            let err = RyoikiError::HookFailed(ctx.hook_name.to_owned());
            match self.config.hooks.on_failure.as_str() {
                "abort" => return Err(err.into()),
                "warn" => {
                    eprintln!(
                        "warning: hook \"{}\" exited with non-zero status",
                        ctx.hook_name
                    );
                }
                _ => {} // ignore
            }
        }

        Ok(())
    }
}
