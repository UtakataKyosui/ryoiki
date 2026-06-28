use thiserror::Error;

#[derive(Debug, Error)]
pub enum RyoikiError {
    #[error("jj command failed: {0}")]
    JjFailed(String),

    #[error("workspace not found: \"{0}\"")]
    WorkspaceNotFound(String),

    #[error("workspace already exists: \"{0}\"")]
    WorkspaceExists(String),

    #[error("external tool not found: {0}")]
    ToolNotFound(String),

    #[error("hook \"{0}\" failed")]
    HookFailed(String),

    #[error("user cancelled")]
    UserCancelled,

    #[error("cannot collapse current domain \"{0}\"")]
    CannotCollapseCurrentDomain(String),
}

impl RyoikiError {
    pub fn exit_code(&self) -> i32 {
        match self {
            RyoikiError::JjFailed(_) => 2,
            RyoikiError::HookFailed(_) => 3,
            RyoikiError::ToolNotFound(_) => 4,
            RyoikiError::UserCancelled => 130,
            _ => 1,
        }
    }
}
