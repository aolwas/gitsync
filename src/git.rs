use regex::Regex;
use std::process::Command;
use std::str;

use crate::error::{GitSyncError, Result};

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
}

pub fn is_git_repo() -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: "git rev-parse --git-dir".to_string(),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to check if directory is a git repository".to_string()),
        })?;

    Ok(output.status.success())
}

pub fn get_main_remote() -> Result<Remote> {
    let remotes = get_remotes()?;

    if remotes.is_empty() {
        return Err(GitSyncError::NoRemotesFound);
    }

    // Priority order: upstream, origin, others
    let priority_order = ["upstream", "origin"];

    for priority in priority_order {
        if let Some(remote) = remotes.iter().find(|r| r.name == priority) {
            return Ok(remote.clone());
        }
    }

    // Return first remote if no priority match
    Ok(remotes[0].clone())
}

pub fn get_remote_by_name(name: &str) -> Result<Remote> {
    let remotes = get_remotes()?;
    
    if let Some(remote) = remotes.iter().find(|r| r.name == name) {
        return Ok(remote.clone());
    }
    
    Err(GitSyncError::RemoteNotFound(name.to_string()))
}

pub fn get_remotes() -> Result<Vec<Remote>> {
    let output = Command::new("git")
        .args(["remote", "-v"])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: "git remote -v".to_string(),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to get remotes".to_string()),
        })?;

    if !output.status.success() {
        return Err(GitSyncError::GitCommandError {
            command: "git remote -v".to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8(output.stderr)
                .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?,
            context: Some("Failed to get remotes".to_string()),
        });
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    let mut remotes = Vec::new();

    for line in output_str.lines() {
        if line.ends_with("(fetch)") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                remotes.push(Remote { name });
            }
        }
    }

    Ok(remotes)
}

pub fn get_default_branch(remote: &Remote) -> Result<String> {
    // Try to get symbolic ref for remote HEAD first
    let output = Command::new("git")
        .args([
            "symbolic-ref",
            &format!("refs/remotes/{}/HEAD", remote.name),
        ])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git symbolic-ref refs/remotes/{}/HEAD", remote.name),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to get remote HEAD symbolic ref".to_string()),
        })?;

    if output.status.success() {
        let ref_str = str::from_utf8(&output.stdout)
            .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
        let ref_str = ref_str.trim();
        let prefix = format!("refs/remotes/{}/", remote.name);
        if ref_str.starts_with(&prefix) {
            return Ok(ref_str[prefix.len()..].to_string());
        }
    }

    // Check if main branch exists on remote
    if has_remote_branch(&format!("refs/remotes/{}/main", remote.name))? {
        return Ok("main".to_string());
    }

    // Check if master branch exists on remote
    if has_remote_branch(&format!("refs/remotes/{}/master", remote.name))? {
        return Ok("master".to_string());
    }

    // Default to main (modern default)
    Ok("main".to_string())
}

pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: "git symbolic-ref --short HEAD".to_string(),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to get current branch".to_string()),
        })?;

    if !output.status.success() {
        return Err(GitSyncError::CurrentBranchError);
    }

    let branch = str::from_utf8(&output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    Ok(branch.trim().to_string())
}

pub fn has_remote_branch(remote_branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", remote_branch])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git show-ref --verify --quiet {}", remote_branch),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to check if remote branch exists".to_string()),
        })?;

    Ok(output.status.success())
}

pub fn get_branch_to_remote_mapping() -> Result<std::collections::HashMap<String, String>> {
    let output = Command::new("git")
        .args(["config", "--get-regexp", r"^branch\..*\.remote$"])
        .output();

    let mut mapping = std::collections::HashMap::new();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = str::from_utf8(&output.stdout)
                .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
            let re = Regex::new(r"^branch\.(.+?)\.remote (.+)")
                .map_err(|e| GitSyncError::RegexError(e))?;

            for line in output_str.lines() {
                if let Some(captures) = re.captures(line) {
                    if captures.len() >= 3 {
                        let branch = captures[1].to_string();
                        let remote = captures[2].to_string();
                        mapping.insert(branch, remote);
                    }
                }
            }
        }
    }

    Ok(mapping)
}

pub fn get_local_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: "git branch --format=%(refname:short)".to_string(),
            exit_code: -1,
            stderr: e.to_string(),
            context: Some("Failed to get local branches".to_string()),
        })?;

    if !output.status.success() {
        return Err(GitSyncError::ParseError("Failed to get local branches".to_string()));
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    let mut branches = Vec::new();

    for line in output_str.lines() {
        let branch = line.trim();
        if !branch.is_empty() {
            branches.push(branch.to_string());
        }
    }

    Ok(branches)
}

pub fn fetch_from_remote(remote: &Remote, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY RUN] Would fetch from remote: {}", remote.name);
        return Ok(());
    }

    let output = Command::new("git")
        .args(["fetch", "--prune", "--quiet", "--progress", &remote.name])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git fetch --prune --quiet --progress {}", remote.name),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to fetch from remote".to_string()),
        })?;

    if !output.status.success() {
        return Err(GitSyncError::NetworkError(format!(
            "Failed to fetch from {}: {}",
            remote.name,
            String::from_utf8(output.stderr)
                .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?
        )));
    }

    Ok(())
}

pub fn is_ancestor(ancestor: &str, descendant: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git merge-base --is-ancestor {} {}", ancestor, descendant),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to check if commit is ancestor".to_string()),
        })?;

    Ok(output.status.success())
}

pub fn is_merged(branch: &str, into: &str) -> Result<bool> {
    // Check if branch is fully merged into 'into' branch
    // A branch is considered merged if there are no commits in the branch that are not in 'into'

    // Check if there are any commits in branch that are not in 'into'
    let output = Command::new("git")
        .args(["rev-list", &format!("{}..{}", into, branch)])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git rev-list {}..{}", into, branch),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to check if branch is merged".to_string()),
        })?;

    // If there are no commits in branch that are not in 'into', then it's merged
    Ok(!output.status.success() || output.stdout.is_empty())
}

pub fn is_identical(ref1: &str, ref2: &str) -> Result<bool> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{}^{{commit}}", ref1),
            &format!("{}^{{commit}}", ref2),
        ])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git rev-parse --verify --quiet {}^{{commit}} {}^{{commit}}", ref1, ref2),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to check if refs are identical".to_string()),
        })?;

    // If both refs resolve to the same commit, rev-parse will succeed
    // But we need to check if the commits are actually the same
    if !output.status.success() {
        return Ok(false);
    }

    let commit1 = str::from_utf8(&output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    let commit1 = commit1.trim();

    let commit2_output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", ref2])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git rev-parse --verify --quiet {}", ref2),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to get commit for ref".to_string()),
        })?;

    if !commit2_output.status.success() {
        return Ok(false);
    }

    let commit2 = str::from_utf8(&commit2_output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    let commit2 = commit2.trim();

    Ok(commit1 == commit2)
}

pub fn get_commit_sha(ref_spec: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", ref_spec])
        .output()
        .map_err(|e| GitSyncError::GitCommandError {
            command: format!("git rev-parse {}", ref_spec),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
            context: Some("Failed to get commit SHA".to_string()),
        })?;

    if !output.status.success() {
        return Err(GitSyncError::commit_sha_parse_error(ref_spec));
    }

    let result = str::from_utf8(&output.stdout)
        .map_err(|e| GitSyncError::Utf8Error(e.to_string()))?;
    Ok(result.trim().to_string())
}



