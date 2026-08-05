use std::collections::HashMap;

use crate::error::{GitSyncError, Result};
use crate::git::{Remote, get_commit_sha, is_ancestor, is_identical, is_merged};
use crate::output::OutputManager;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum BranchAction {
    Updated,
    Deleted,
    Warning,
    NoChange,

}

fn perform_branch_update(
    args: &[&str],
    branch: &str,
    old_commit_short: &str,
    output_manager: &OutputManager,
    error_context: &str,
) -> Result<bool> {
    let cmd_output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git {}", args.join(" ")),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some(error_context.to_string()),
        })?;

    if cmd_output.status.success() {
        output_manager.success(&format!(
            "Updated branch '{}' (was {}).",
            branch, old_commit_short
        ));
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn process_branch(
    branch: &str,
    remote: &Remote,
    branch_to_remote: &HashMap<String, String>,
    current_branch: &str,
    default_branch: &str,
    local_default_branch: &str,
    dry_run: bool,
    output_manager: &OutputManager,
) -> Result<BranchAction> {
    let full_branch = format!("refs/heads/{}", branch);
    let mut remote_branch = format!("refs/remotes/{}/{}", remote.name, branch);
    let mut gone = false;

    // Check if branch has upstream configuration
    if let Some(branch_remote) = branch_to_remote.get(branch) {
        if *branch_remote == remote.name {
            let upstream_ref = format!("{}@{{upstream}}", branch);
            match get_commit_sha(&upstream_ref) {
                Ok(_sha) => {
                    // Get the actual upstream branch reference
                    let upstream_full = std::process::Command::new("git")
                        .args(["rev-parse", "--symbolic-full-name", &upstream_ref])
                        .output()
                        .map_err(|e| GitSyncError::GitCommandError {
                            command: format!("git rev-parse --symbolic-full-name {}", upstream_ref),
                            exit_code: -1,
                            stderr: e.to_string(),
                            context: Some("Failed to get upstream branch reference".to_string()),
                        })?;

                    if upstream_full.status.success() {
                        let upstream_str = String::from_utf8(upstream_full.stdout)
                            .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
                        remote_branch = upstream_str.trim().to_string();
                    }
                }
                Err(_) => {
                    // Upstream is gone or error getting SHA
                    remote_branch = String::new();
                    gone = true;
                }
            }
        }
    } else if !crate::git::has_remote_branch(&remote_branch)? {
        remote_branch = String::new();
    }

    if !remote_branch.is_empty() {
        // Branch has corresponding remote branch
        // First check if branches are identical
        output_manager.verbose(&format!("Comparing {} with {}", full_branch, remote_branch));
        if is_identical(&full_branch, &remote_branch)? {
            // Branches are identical, do nothing
            output_manager.verbose(&format!("Branch '{}' is identical to remote", branch));
            return Ok(BranchAction::NoChange);
        }
        
        // Check if local is ancestor of remote (meaning local is behind)
        let local_behind = is_ancestor(&full_branch, &remote_branch)?;
        let remote_behind = is_ancestor(&remote_branch, &full_branch)?;
        
        output_manager.verbose(&format!("Branch '{}': local_behind={}, remote_behind={}", branch, local_behind, remote_behind));
        
        // If both are true, branches point to the same commit (up-to-date)
        if local_behind && remote_behind {
            output_manager.verbose(&format!("Branch '{}' is up-to-date (same commit)", branch));
            return Ok(BranchAction::NoChange);
        } else if local_behind && !remote_behind {
            output_manager.verbose(&format!("Branch '{}' is behind remote", branch));
            // Local branch is ancestor of remote (behind), can fast-forward
            let old_commit = get_commit_sha(&full_branch)?;
            let old_commit_short = if old_commit.len() > 7 {
                &old_commit[..7]
            } else {
                &old_commit
            };

            if dry_run {
                output_manager.info(&format!(
                    "[DRY RUN] Would update branch '{}' (was {}).",
                    branch, old_commit_short
                ));
            } else {
                // Perform the update
                if branch == current_branch {
                    // For current branch, use merge --ff-only
                    if perform_branch_update(
                        &["merge", "--ff-only", "--quiet", &remote_branch],
                        branch,
                        old_commit_short,
                        &output_manager,
                        "Failed to fast-forward current branch"
                    )? {
                        return Ok(BranchAction::Updated);
                    } else {
                        output_manager
                            .warning(&format!("warning: couldn't fast-forward '{}'", branch));
                        return Ok(BranchAction::Warning);
                    }
                } else {
                    // For other branches, use update-ref
                    if perform_branch_update(
                        &["update-ref", &full_branch, &remote_branch],
                        branch,
                        old_commit_short,
                        &output_manager,
                        "Failed to update branch reference"
                    )? {
                        return Ok(BranchAction::Updated);
                    } else {
                        output_manager
                            .warning(&format!("warning: couldn't fast-forward '{}'", branch));
                        return Ok(BranchAction::Warning);
                    }
                }
            }
        } else {
            // Local branch has unpushed commits
            output_manager.warning(&format!(
                "warning: '{}' seems to contain unpushed commits",
                branch
            ));
            return Ok(BranchAction::Warning);
        }
    } else if gone {
        // Remote branch was deleted
        if is_merged(&full_branch, local_default_branch)? {
            // Branch is ancestor of default branch, safe to delete
            let old_commit = get_commit_sha(&full_branch)?;
            let old_commit_short = if old_commit.len() > 7 {
                &old_commit[..7]
            } else {
                &old_commit
            };

            if dry_run {
                output_manager.info(&format!(
                    "[DRY RUN] Would delete branch '{}' (was {}).",
                    branch, old_commit_short
                ));
            } else {
                // Need to checkout default branch if deleting current branch
                if branch == current_branch {
                    let cmd_output = std::process::Command::new("git")
                        .args(["checkout", "--quiet", default_branch])
                        .output()
                        .map_err(|e| GitSyncError::GitCommandError {
                            command: format!("git checkout --quiet {}", default_branch),
                            exit_code: e.raw_os_error().unwrap_or(-1),
                            stderr: e.to_string(),
                            context: Some("Failed to checkout default branch before deletion".to_string()),
                        })?;

                    if !cmd_output.status.success() {
                        output_manager.warning(&format!(
                            "warning: couldn't checkout '{}' before deleting '{}'",
                            default_branch, branch
                        ));
                        return Ok(BranchAction::Warning);
                    }
                }

                // Delete the branch
                let cmd_output = std::process::Command::new("git")
                    .args(["branch", "-D", branch])
                    .output()
                    .map_err(|e| GitSyncError::GitCommandError {
                        command: format!("git branch -D {}", branch),
                        exit_code: e.raw_os_error().unwrap_or(-1),
                        stderr: e.to_string(),
                        context: Some("Failed to delete branch".to_string()),
                    })?;

                if cmd_output.status.success() {
                    output_manager.success(&format!(
                        "Deleted branch '{}' (was {}).",
                        branch, old_commit_short
                    ));
                    return Ok(BranchAction::Deleted);
                } else {
                    output_manager.warning(&format!("warning: couldn't delete '{}'", branch));
                    return Ok(BranchAction::Warning);
                }
            }
        } else {
            // Branch appears not merged
            output_manager.warning(&format!(
                "warning: '{}' was deleted on {}, but appears not merged into '{}'",
                branch, remote.name, default_branch
            ));
            return Ok(BranchAction::Warning);
        }
    }

    Ok(BranchAction::NoChange)
}
