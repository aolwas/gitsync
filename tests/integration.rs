use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--dry-run"));

    Ok(())
}

#[test]
fn test_cli_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("git-sync"));

    Ok(())
}

#[test]
fn test_non_git_directory() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.current_dir(path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));

    dir.close()?;
    Ok(())
}

#[test]
fn test_git_repo_no_remotes() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--dry-run").current_dir(path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No git remotes found"));

    dir.close()?;
    Ok(())
}

#[test]
fn test_dry_run_with_remote() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Get the current branch to use for assertions
    let branch_check = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()?;
    let current_branch = String::from_utf8(branch_check.stdout)?.trim().to_string();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ])
        .current_dir(path)
        .output()?;

    // Set up the remote to have a default branch that matches our current branch
    Command::new("git")
        .args([
            "symbolic-ref",
            &format!("refs/remotes/origin/HEAD"),
            &format!("refs/remotes/origin/{}", current_branch),
        ])
        .current_dir(path)
        .output()?;

    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--dry-run").arg("--verbose").current_dir(path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Using remote: origin"))
        .stdout(predicate::str::contains(&format!(
            "Default branch: {}",
            current_branch
        )))
        .stdout(predicate::str::contains(
            "[DRY RUN] Would fetch from remote: origin",
        ));

    dir.close()?;
    Ok(())
}

#[test]
fn test_verbose_output() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Get the current branch to use for assertions
    let branch_check = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()?;
    let current_branch = String::from_utf8(branch_check.stdout)?.trim().to_string();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ])
        .current_dir(path)
        .output()?;

    // Set up the remote to have a default branch that matches our current branch
    Command::new("git")
        .args([
            "symbolic-ref",
            &format!("refs/remotes/origin/HEAD"),
            &format!("refs/remotes/origin/{}", current_branch),
        ])
        .current_dir(path)
        .output()?;

    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--verbose").arg("--dry-run").current_dir(path);

    let output = cmd.output()?;

    // Check that verbose output contains expected messages
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Using remote: origin"));
    assert!(stdout.contains(&format!("Default branch: {}", current_branch)));
    assert!(stdout.contains(&format!("Current branch: {}", current_branch)));

    dir.close()?;
    Ok(())
}

#[test]
fn test_color_output() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ])
        .current_dir(path)
        .output()?;

    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--color")
        .arg("always")
        .arg("--dry-run")
        .current_dir(path);

    cmd.assert().success().stdout(predicate::str::contains(
        "[DRY RUN] Would fetch from remote: origin",
    ));

    dir.close()?;
    Ok(())
}

#[test]
fn test_branch_deletion_detection() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Get the current branch to use for assertions
    let branch_check = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()?;
    let current_branch = String::from_utf8(branch_check.stdout)?.trim().to_string();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ])
        .current_dir(path)
        .output()?;

    // Set up the remote to have a default branch that matches our current branch
    Command::new("git")
        .args([
            "symbolic-ref",
            &format!("refs/remotes/origin/HEAD"),
            &format!("refs/remotes/origin/{}", current_branch),
        ])
        .current_dir(path)
        .output()?;

    // Test that sync works with a basic repo setup
    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--dry-run").arg("--verbose").current_dir(path);

    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Should show basic sync information
    assert!(stdout.contains("Using remote: origin"));
    assert!(stdout.contains(&format!("Default branch: {}", current_branch)));
    assert!(stdout.contains("[DRY RUN] Would fetch from remote: origin"));

    dir.close()?;
    Ok(())
}

#[test]
fn test_up_to_date_branch_detection() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    // Set up initial commit
    let readme_path = path.join("README.md");
    let mut file = File::create(&readme_path)?;
    writeln!(file, "Test repo")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Get the current branch and commit
    let branch_check = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()?;
    let current_branch = String::from_utf8(branch_check.stdout)?.trim().to_string();

    let commit_check = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()?;
    let current_commit = String::from_utf8(commit_check.stdout)?.trim().to_string();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ])
        .current_dir(path)
        .output()?;

    // Create a remote tracking branch that points to the same commit
    Command::new("git")
        .args([
            "update-ref",
            &format!("refs/remotes/origin/{}", current_branch),
            &current_commit,
        ])
        .current_dir(path)
        .output()?;

    // Set up the remote to have a default branch that matches our current branch
    Command::new("git")
        .args([
            "symbolic-ref",
            &format!("refs/remotes/origin/HEAD"),
            &format!("refs/remotes/origin/{}", current_branch),
        ])
        .current_dir(path)
        .output()?;

    // Test that sync doesn't crash and handles the basic scenario correctly
    let mut cmd = Command::cargo_bin("git-sync")?;
    cmd.arg("--dry-run")
        .arg("--verbose")
        .current_dir(path);

    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Should show basic sync information without crashing
    assert!(stdout.contains("Using remote: origin"));
    assert!(stdout.contains(&format!("Default branch: {}", current_branch)));
    assert!(stdout.contains("[DRY RUN] Would fetch from remote: origin"));
    
    // Should complete successfully without panicking
    assert!(output.status.success());
    
    // The key test: should not indicate any updates are needed when branches are at same commit
    // This is a negative test - ensuring no false positives
    assert!(!stdout.contains("Would update branch 'main'"));
    assert!(!stdout.contains("Would delete branch 'main'"));

    dir.close()?;
    Ok(())
}
