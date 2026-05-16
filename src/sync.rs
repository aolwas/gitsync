use crate::git::Repo;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Change {
    pub branch: String,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub enum Action {
    FastForward { upstream: String },
    DeleteMerged { into: String },
    DeleteUpstreamGone,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::FastForward { upstream } => write!(f, "Fast-forward from {}", upstream),
            Action::DeleteMerged { into } => write!(f, "Delete (merged into {})", into),
            Action::DeleteUpstreamGone => write!(f, "Delete (upstream gone)"),
        }
    }
}

pub struct Syncer {
    repo: Repo,
    current_branch: String,
    default_branch: String,
}

impl Syncer {
    pub fn new(repo: Repo, default_branch: &str) -> Result<Self> {
        let current_branch = repo.current_branch().unwrap_or_else(|_| "main".to_string());
        Ok(Syncer {
            repo,
            current_branch,
            default_branch: default_branch.to_string(),
        })
    }

    pub fn find_changes(&self) -> Result<Vec<Change>> {
        let branches = self.repo.local_branches()?;
        let mut changes = Vec::new();

        for branch in branches {
            if branch.name == self.current_branch {
                continue;
            }

            if let Some(upstream) = &branch.tracking {
                // Check for fast-forward
                if self.repo.can_fastforward(&branch.name, upstream)? {
                    changes.push(Change {
                        branch: branch.name.clone(),
                        action: Action::FastForward {
                            upstream: upstream.clone(),
                        },
                    });
                    continue;
                }

                // Check if upstream is gone
                if !self.repo.upstream_exists(upstream)? {
                    changes.push(Change {
                        branch: branch.name.clone(),
                        action: Action::DeleteUpstreamGone,
                    });
                    continue;
                }

                // Check if merged into default branch
                if self.repo.is_merged(&branch.name, &self.default_branch)? {
                    changes.push(Change {
                        branch: branch.name.clone(),
                        action: Action::DeleteMerged {
                            into: self.default_branch.clone(),
                        },
                    });
                }
            }
        }

        Ok(changes)
    }

    pub fn apply_changes(&self, changes: &[Change]) -> Result<()> {
        for change in changes {
            match &change.action {
                Action::FastForward { upstream } => {
                    self.repo.fastforward(&change.branch, upstream)?;
                    println!("  ✓ Fast-forwarded {}", change.branch);
                }
                Action::DeleteMerged { .. } | Action::DeleteUpstreamGone => {
                    self.repo.delete_branch(&change.branch)?;
                    println!("  ✓ Deleted {}", change.branch);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display_fast_forward() {
        let action = Action::FastForward {
            upstream: "origin/main".to_string(),
        };
        assert_eq!(action.to_string(), "Fast-forward from origin/main");
    }

    #[test]
    fn test_action_display_delete_merged() {
        let action = Action::DeleteMerged {
            into: "main".to_string(),
        };
        assert_eq!(action.to_string(), "Delete (merged into main)");
    }

    #[test]
    fn test_action_display_delete_upstream_gone() {
        let action = Action::DeleteUpstreamGone;
        assert_eq!(action.to_string(), "Delete (upstream gone)");
    }

    #[test]
    fn test_change_structure() {
        let change = Change {
            branch: "feature".to_string(),
            action: Action::FastForward {
                upstream: "origin/feature".to_string(),
            },
        };
        assert_eq!(change.branch, "feature");
        assert_eq!(
            change.action.to_string(),
            "Fast-forward from origin/feature"
        );
    }
}
