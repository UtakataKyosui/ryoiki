use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub core: CoreConfig,
    pub list: ListConfig,
    pub fzf: FzfConfig,
    pub tmux: TmuxConfig,
    pub zoxide: ZoxideConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CoreConfig {
    pub base_dir: Option<String>,
    pub dir_format: String,
    pub show_status_on_enter: bool,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ListConfig {
    pub format: String,
    pub show_stale: bool,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FzfConfig {
    pub enabled: bool,
    pub opts: String,
    pub preview: bool,
    pub preview_window: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TmuxConfig {
    pub default_layout: String,
    pub session_format: String,
    pub auto_attach: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ZoxideConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HooksConfig {
    pub enabled: bool,
    pub hook_dir: Option<String>,
    pub timeout_seconds: u64,
    pub on_failure: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            list: ListConfig::default(),
            fzf: FzfConfig::default(),
            tmux: TmuxConfig::default(),
            zoxide: ZoxideConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            base_dir: None,
            dir_format: "{repo}-{name}".into(),
            show_status_on_enter: true,
            color: "auto".into(),
        }
    }
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            format: "table".into(),
            show_stale: false,
            columns: vec![
                "name".into(),
                "path".into(),
                "change_id".into(),
                "description".into(),
                "status".into(),
            ],
        }
    }
}

impl Default for FzfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            opts: "--height=40% --border --reverse".into(),
            preview: true,
            preview_window: "right:50%:wrap".into(),
        }
    }
}

impl Default for TmuxConfig {
    fn default() -> Self {
        Self {
            default_layout: "even-horizontal".into(),
            session_format: "{repo}".into(),
            auto_attach: true,
        }
    }
}

impl Default for ZoxideConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hook_dir: None,
            timeout_seconds: 30,
            on_failure: "warn".into(),
        }
    }
}

impl Config {
    pub fn load(explicit_path: Option<&Path>, repo_root: Option<&Path>) -> Result<Self> {
        let global = Self::load_global()?;
        let local = if let Some(root) = repo_root {
            Self::load_local(root)?
        } else {
            None
        };

        if let Some(path) = explicit_path {
            let explicit: Config = toml::from_str(
                &std::fs::read_to_string(path)
                    .with_context(|| format!("failed to read config: {}", path.display()))?,
            )
            .context("failed to parse config file")?;
            return Ok(explicit);
        }

        match local {
            Some(l) => Ok(Self::merge(global, l)),
            None => Ok(global),
        }
    }

    fn load_global() -> Result<Self> {
        // Priority: $RYOIKI_CONFIG > $XDG_CONFIG_HOME/ryoiki/config.toml
        let path = if let Ok(p) = std::env::var("RYOIKI_CONFIG") {
            PathBuf::from(p)
        } else {
            let config_home = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            config_home.join("ryoiki").join("config.toml")
        };

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            toml::from_str(&content).context("failed to parse global config")
        } else {
            Ok(Self::default())
        }
    }

    fn load_local(repo_root: &Path) -> Result<Option<Self>> {
        let path = repo_root.join(".ryoiki.toml");
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(Some(
                toml::from_str(&content).context("failed to parse .ryoiki.toml")?,
            ))
        } else {
            Ok(None)
        }
    }

    fn merge(global: Self, local: Self) -> Self {
        let local_default = Self::default();

        Self {
            core: CoreConfig {
                base_dir: local.core.base_dir.or(global.core.base_dir),
                dir_format: if local.core.dir_format != local_default.core.dir_format {
                    local.core.dir_format
                } else {
                    global.core.dir_format
                },
                show_status_on_enter: if local.core.show_status_on_enter
                    != local_default.core.show_status_on_enter
                {
                    local.core.show_status_on_enter
                } else {
                    global.core.show_status_on_enter
                },
                color: if local.core.color != local_default.core.color {
                    local.core.color
                } else {
                    global.core.color
                },
            },
            list: if local.list.format != local_default.list.format
                || local.list.show_stale != local_default.list.show_stale
            {
                local.list
            } else {
                global.list
            },
            fzf: if local.fzf.opts != local_default.fzf.opts {
                local.fzf
            } else {
                global.fzf
            },
            tmux: if local.tmux.default_layout != local_default.tmux.default_layout {
                local.tmux
            } else {
                global.tmux
            },
            zoxide: if local.zoxide.enabled != local_default.zoxide.enabled {
                local.zoxide
            } else {
                global.zoxide
            },
            hooks: HooksConfig {
                enabled: if local.hooks.enabled != local_default.hooks.enabled {
                    local.hooks.enabled
                } else {
                    global.hooks.enabled
                },
                hook_dir: local.hooks.hook_dir.or(global.hooks.hook_dir),
                timeout_seconds: if local.hooks.timeout_seconds
                    != local_default.hooks.timeout_seconds
                {
                    local.hooks.timeout_seconds
                } else {
                    global.hooks.timeout_seconds
                },
                on_failure: if local.hooks.on_failure != local_default.hooks.on_failure {
                    local.hooks.on_failure
                } else {
                    global.hooks.on_failure
                },
            },
        }
    }

    pub fn global_hook_dir(&self) -> PathBuf {
        if let Some(dir) = &self.hooks.hook_dir {
            expand_tilde(dir)
        } else {
            let config_home = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            config_home.join("ryoiki").join("hooks")
        }
    }

    pub fn resolve_base_dir(&self, repo_root: &Path) -> PathBuf {
        match &self.core.base_dir {
            Some(dir) => {
                let p = expand_tilde(dir);
                if p.is_absolute() {
                    p
                } else {
                    repo_root.join(p)
                }
            }
            None => repo_root.parent().unwrap_or(repo_root).to_path_buf(),
        }
    }
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(rest)
    } else if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
    } else {
        PathBuf::from(path)
    }
}
