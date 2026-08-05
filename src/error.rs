use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitSyncError {
    #[error("Git command failed: {command}")]
    GitCommandError {
        command: String,
        exit_code: i32,
        stderr: String,
        context: Option<String>,
    },

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("No git remotes found")]
    NoRemotesFound,

    #[error("Failed to determine default branch")]
    DefaultBranchError,

    #[error("Failed to get current branch")]
    CurrentBranchError,

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("Remote '{0}' not found")]
    RemoteNotFound(String),

    #[error("Failed to parse git output: {0}")]
    ParseError(String),

    #[error("Failed to parse git output for default branch")]
    DefaultBranchParseError,

    #[error("Failed to parse git output for branch mapping")]
    BranchMappingParseError,

    #[error("Failed to parse git output for commit SHA: {0}")]
    CommitShaParseError(String),

    #[error("Permission denied: {0}")]
    PermissionError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Branch '{0}' has unpushed commits")]
    UnpushedCommits(String),

    #[error("Branch '{0}' is not merged into '{1}")]
    BranchNotMerged(String, String),

    #[error("Failed to update branch '{0}': {1}")]
    BranchUpdateError(String, String),

    #[error("Failed to delete branch '{0}': {1}")]
    BranchDeleteError(String, String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

impl GitSyncError {
    pub fn with_context(self, context: &str) -> Self {
        match self {
            GitSyncError::GitCommandError {
                command,
                exit_code,
                stderr,
                context: existing_context,
            } => GitSyncError::GitCommandError {
                context: Some(format!(
                    "{}: {}",
                    context,
                    existing_context.unwrap_or_default()
                )),
                command,
                exit_code,
                stderr,
            },
            GitSyncError::ParseError(msg) => GitSyncError::ParseError(format!("{}: {}", context, msg)),
            GitSyncError::CommitShaParseError(ref_spec) => GitSyncError::CommitShaParseError(format!("{}: {}", context, ref_spec)),
            GitSyncError::NetworkError(msg) => GitSyncError::NetworkError(format!("{}: {}", context, msg)),
            GitSyncError::PermissionError(msg) => GitSyncError::PermissionError(format!("{}: {}", context, msg)),
            GitSyncError::BranchNotMerged(branch, into) => GitSyncError::BranchNotMerged(branch, format!("{}: {}", context, into)),
            GitSyncError::BranchUpdateError(branch, msg) => GitSyncError::BranchUpdateError(branch, format!("{}: {}", context, msg)),
            GitSyncError::BranchDeleteError(branch, msg) => GitSyncError::BranchDeleteError(branch, format!("{}: {}", context, msg)),
            _ => self,
        }
    }


}

impl From<std::str::Utf8Error> for GitSyncError {
    fn from(err: std::str::Utf8Error) -> Self {
        GitSyncError::Utf8Error(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for GitSyncError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        GitSyncError::Utf8Error(err.to_string())
    }
}

impl GitSyncError {
    pub fn commit_sha_parse_error(ref_spec: &str) -> Self {
        GitSyncError::CommitShaParseError(ref_spec.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitSyncError>;
