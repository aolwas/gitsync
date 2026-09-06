use clap::Parser;
use log::error;

use std::process;

mod cli;
mod error;

mod git;
mod output;
mod sync;

use cli::Args;
use error::GitSyncError;
use output::OutputManager;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), GitSyncError> {
    let args = Args::parse();
    let mut output = OutputManager::new(args.verbose, args.color);

    // Initialize logging
    env_logger::Builder::new()
        .filter_level(if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Error
        })
        .init();

    // Check if current directory is a git repository
    if !git::is_git_repo()? {
        return Err(GitSyncError::NotGitRepository);
    }

    // Get main remote
    let remote = if let Some(ref remote_name) = args.remote {
        git::get_remote_by_name(remote_name)?
    } else {
        git::get_main_remote()?
    };
    output.verbose(&format!("Using remote: {}", remote.name));

    // Get default branch
    let default_branch = git::get_default_branch(&remote)?;
    output.verbose(&format!("Default branch: {}", default_branch));

    // Fetch from remote
    git::fetch_from_remote(&remote, args.dry_run)?;

    // Get current branch
    let current_branch = git::get_current_branch()?;
    output.verbose(&format!("Current branch: {}", current_branch));

    // Get branch to remote mapping
    let branch_to_remote = git::get_branch_to_remote_mapping()?;

    // Get all local branches
    let branches = git::get_local_branches()?;

    // Process each branch
    let local_default_branch = format!("refs/heads/{}", default_branch);

    let mut processed_count = 0;
    let mut updated_count = 0;
    let mut deleted_count = 0;
    let mut warning_count = 0;
    
    // Start progress bar for non-verbose mode
    if !args.verbose {
        output.start_progress(branches.len() as u64, "Processing branches");
    }
    
    for branch in branches {
        match sync::process_branch(
            &branch,
            &remote,
            &branch_to_remote,
            &current_branch,
            &default_branch,
            &local_default_branch,
            args.dry_run,
            &output,
        ) {
            Ok(action) => {
                processed_count += 1;
                match action {
                    sync::BranchAction::Updated => updated_count += 1,
                    sync::BranchAction::Deleted => deleted_count += 1,
                    sync::BranchAction::Warning => warning_count += 1,
                    sync::BranchAction::NoChange => {},
                }
            }
            Err(e) => {
                let error_msg = format!("Error processing branch '{}': {}", branch, e);
                error!("{}", error_msg);
                if args.continue_on_error {
                    warning_count += 1;
                    output.warning(&error_msg);
                } else {
                    return Err(e.with_context(&format!("While processing branch '{}'", branch)));
                }
            }
        }
        
        // Update progress
        if !args.verbose {
            output.update_progress(1);
        }
    }

    // Finish progress bar
    if !args.verbose {
        output.finish_progress("Processing complete");
    }

    // Print summary
    output.info(&format!("Processed {} branches", processed_count));
    if updated_count > 0 {
        output.success(&format!("Updated {} branches", updated_count));
    }
    if deleted_count > 0 {
        output.success(&format!("Deleted {} branches", deleted_count));
    }
    if warning_count > 0 {
        output.warning(&format!("{} branches had warnings or errors", warning_count));
    }

    Ok(())
}
