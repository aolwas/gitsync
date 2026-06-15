use anyhow::{Context, Result};
use regex::Regex;
use std::process::Command;
use std::str;

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
}

pub fn is_git_repo() -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to execute git rev-parse")?;

    Ok(output.status.success())
}

pub fn get_main_remote() -> Result<Remote> {
    let remotes = get_remotes()?;

    if remotes.is_empty() {
        return Err(anyhow::anyhow!("No git remotes found"));
    }

    // Priority order: upstream, github, origin, others
    let priority_order = ["upstream", "github", "origin"];

    for priority in priority_order {
        if let Some(remote) = remotes.iter().find(|r| r.name == priority) {
            return Ok(remote.clone());
        }
    }

    // Return first remote if no priority match
    Ok(remotes[0].clone())
}

pub fn get_remotes() -> Result<Vec<Remote>> {
    let output = Command::new("git")
        .args(["remote", "-v"])
        .output()
        .context("Failed to execute git remote -v")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get remotes"));
    }

    let output_str = str::from_utf8(&output.stdout)?;
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
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let ref_str = str::from_utf8(&output.stdout)?;
            let ref_str = ref_str.trim();
            let prefix = format!("refs/remotes/{}/", remote.name);
            if ref_str.starts_with(&prefix) {
                return Ok(ref_str[prefix.len()..].to_string());
            }
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
        .context("Failed to execute git symbolic-ref")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get current branch"));
    }

    let branch = str::from_utf8(&output.stdout)?;
    Ok(branch.trim().to_string())
}

pub fn has_remote_branch(remote_branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", remote_branch])
        .output()
        .context("Failed to execute git show-ref")?;

    Ok(output.status.success())
}

pub fn get_branch_to_remote_mapping() -> Result<std::collections::HashMap<String, String>> {
    let output = Command::new("git")
        .args(["config", "--get-regexp", r"^branch\..*\.remote$"])
        .output();

    let mut mapping = std::collections::HashMap::new();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = str::from_utf8(&output.stdout)?;
            let re = Regex::new(r"^branch\.(.+?)\.remote (.+)")?;

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
        .context("Failed to execute git branch")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get local branches"));
    }

    let output_str = str::from_utf8(&output.stdout)?;
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
        .context("Failed to execute git fetch")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to fetch from {}", remote.name));
    }

    Ok(())
}

pub fn is_ancestor(ancestor: &str, descendant: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .output()
        .context("Failed to execute git merge-base")?;

    Ok(output.status.success())
}

pub fn is_merged(branch: &str, into: &str) -> Result<bool> {
    // Check if branch is fully merged into 'into' branch
    // A branch is considered merged if there are no commits in the branch that are not in 'into'

    // Check if there are any commits in branch that are not in 'into'
    let output = Command::new("git")
        .args(["rev-list", &format!("{}..{}", into, branch)])
        .output()
        .context("Failed to execute git rev-list")?;

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
        .context("Failed to execute git rev-parse for identity check")?;

    // If both refs resolve to the same commit, rev-parse will succeed
    // But we need to check if the commits are actually the same
    if !output.status.success() {
        return Ok(false);
    }

    let commit1 = str::from_utf8(&output.stdout)?;
    let commit1 = commit1.trim();

    let commit2_output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", ref2])
        .output()
        .context("Failed to execute git rev-parse")?;

    if !commit2_output.status.success() {
        return Ok(false);
    }

    let commit2 = str::from_utf8(&commit2_output.stdout)?;
    let commit2 = commit2.trim();

    Ok(commit1 == commit2)
}

pub fn get_commit_sha(ref_spec: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", ref_spec])
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        return Ok("unknown".to_string());
    }

    let result = str::from_utf8(&output.stdout)?;
    Ok(result.trim().to_string())
}
