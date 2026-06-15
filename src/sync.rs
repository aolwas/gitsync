use anyhow::Result;
use std::collections::HashMap;

use crate::git::{Remote, get_commit_sha, is_ancestor, is_identical, is_merged};
use crate::output::OutputManager;

pub fn process_branch(
    branch: &str,
    remote: &Remote,
    branch_to_remote: &HashMap<String, String>,
    current_branch: &str,
    default_branch: &str,
    local_default_branch: &str,
    dry_run: bool,
    output_manager: &OutputManager,
) -> Result<()> {
    let full_branch = format!("refs/heads/{}", branch);
    let mut remote_branch = format!("refs/remotes/{}/{}", remote.name, branch);
    let mut gone = false;

    // Check if branch has upstream configuration
    if let Some(branch_remote) = branch_to_remote.get(branch) {
        if *branch_remote == remote.name {
            let upstream_ref = format!("{}@{{upstream}}", branch);
            match get_commit_sha(&upstream_ref) {
                Ok(sha) if sha != "unknown" => {
                    // Get the actual upstream branch reference
                    let upstream_full = std::process::Command::new("git")
                        .args(["rev-parse", "--symbolic-full-name", &upstream_ref])
                        .output()?;

                    if upstream_full.status.success() {
                        let upstream_str = String::from_utf8(upstream_full.stdout)?;
                        remote_branch = upstream_str.trim().to_string();
                    }
                }
                _ => {
                    // Upstream is gone
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
        if is_identical(&full_branch, &remote_branch)? {
            // Branches are identical, do nothing
            return Ok(());
        } else if is_ancestor(&full_branch, &remote_branch)? {
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
                    let update_result = std::process::Command::new("git")
                        .args(["merge", "--ff-only", "--quiet", &remote_branch])
                        .output();

                    if let Ok(cmd_output) = update_result {
                        if !cmd_output.status.success() {
                            output_manager
                                .warning(&format!("warning: couldn't fast-forward '{}'", branch));
                        } else {
                            output_manager.success(&format!(
                                "Updated branch '{}' (was {}).",
                                branch, old_commit_short
                            ));
                        }
                    }
                } else {
                    // For other branches, use update-ref
                    let update_result = std::process::Command::new("git")
                        .args(["update-ref", &full_branch, &remote_branch])
                        .output();

                    if let Ok(cmd_output) = update_result {
                        if !cmd_output.status.success() {
                            output_manager
                                .warning(&format!("warning: couldn't fast-forward '{}'", branch));
                        } else {
                            output_manager.success(&format!(
                                "Updated branch '{}' (was {}).",
                                branch, old_commit_short
                            ));
                        }
                    }
                }
            }
        } else {
            // Local branch has unpushed commits
            output_manager.warning(&format!(
                "warning: '{}' seems to contain unpushed commits",
                branch
            ));
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
                    let _checkout_result = std::process::Command::new("git")
                        .args(["checkout", "--quiet", default_branch])
                        .output();
                }

                // Delete the branch
                let delete_result = std::process::Command::new("git")
                    .args(["branch", "-D", branch])
                    .output();

                if delete_result.is_ok() && delete_result.as_ref().unwrap().status.success() {
                    output_manager.success(&format!(
                        "Deleted branch '{}' (was {}).",
                        branch, old_commit_short
                    ));
                } else {
                    output_manager.warning(&format!("warning: couldn't delete '{}'", branch));
                }
            }
        } else {
            // Branch appears not merged
            output_manager.warning(&format!(
                "warning: '{}' was deleted on {}, but appears not merged into '{}'",
                branch, remote.name, default_branch
            ));
        }
    }

    Ok(())
}
