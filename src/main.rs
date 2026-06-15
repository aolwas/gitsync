use anyhow::{Context, Result};
use clap::Parser;

use std::process;

mod cli;

mod git;
mod output;
mod sync;

use cli::Args;
use output::OutputManager;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let output = OutputManager::new(args.verbose, args.color);

    // Check if current directory is a git repository
    if !git::is_git_repo().context("Failed to check if current directory is a git repository")? {
        return Err(anyhow::anyhow!("fatal: Not a git repository"));
    }

    // Get main remote
    let remote = git::get_main_remote().context("Failed to get main remote")?;
    output.verbose(&format!("Using remote: {}", remote.name));

    // Get default branch
    let default_branch =
        git::get_default_branch(&remote).context("Failed to get default branch")?;
    output.verbose(&format!("Default branch: {}", default_branch));

    // Fetch from remote
    git::fetch_from_remote(&remote, args.dry_run).context("Failed to fetch from remote")?;

    // Get current branch
    let current_branch = git::get_current_branch().context("Failed to get current branch")?;
    output.verbose(&format!("Current branch: {}", current_branch));

    // Get branch to remote mapping
    let branch_to_remote =
        git::get_branch_to_remote_mapping().context("Failed to get branch to remote mapping")?;

    // Get all local branches
    let branches = git::get_local_branches().context("Failed to get local branches")?;

    // Process each branch
    let full_default_branch = format!("refs/remotes/{}/{}", remote.name, default_branch);

    for branch in branches {
        sync::process_branch(
            &branch,
            &remote,
            &branch_to_remote,
            &current_branch,
            &default_branch,
            &full_default_branch,
            args.dry_run,
            &output,
        )
        .context(format!("Failed to process branch {}", branch))?;
    }

    Ok(())
}
