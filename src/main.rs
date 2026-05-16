mod git;
mod sync;

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "git-sync")]
#[command(about = "Sync local branches with remote", long_about = None)]
struct Args {
    /// Show what would be done without making changes
    #[arg(long)]
    dry_run: bool,

    /// Repository path (defaults to current directory)
    #[arg(short = 'C')]
    repo: Option<String>,

    /// Default branch to check for merged branches
    #[arg(short = 'd', long, default_value = "main")]
    default_branch: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let repo_path = args.repo.as_deref().unwrap_or(".");
    
    // Open repo and fetch
    println!("Opening repository at {}", repo_path);
    let repo = git::Repo::open(repo_path)?;
    
    println!("Fetching remotes...");
    repo.fetch()?;
    
    // Find changes
    let syncer = sync::Syncer::new(repo, &args.default_branch)?;
    let changes = syncer.find_changes()?;
    
    if changes.is_empty() {
        println!("✓ Everything up to date!");
        return Ok(());
    }
    
    // Display changes
    println!("\nProposed changes:");
    let mut deletions = Vec::new();
    for change in &changes {
        println!("  • {}: {}", change.branch, change.action);
        if matches!(change.action, sync::Action::DeleteMerged { .. } | sync::Action::DeleteUpstreamGone) {
            deletions.push(change.clone());
        }
    }
    
    if args.dry_run {
        println!("\n[DRY RUN] No changes applied");
        return Ok(());
    }
    
    // Prompt for confirmation if there are deletions
    if !deletions.is_empty() {
        println!("\n{} branch(es) will be deleted. Continue? (y/n) ", deletions.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }
    
    // Apply changes
    println!("\nApplying changes...");
    syncer.apply_changes(&changes)?;
    
    println!("\n✓ Done!");
    Ok(())
}
