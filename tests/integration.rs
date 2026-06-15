use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("gitsync")?;
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
    let mut cmd = Command::cargo_bin("gitsync")?;
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("gitsync"));

    Ok(())
}

#[test]
fn test_non_git_directory() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path();

    let mut cmd = Command::cargo_bin("gitsync")?;
    cmd.current_dir(path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("fatal: Not a git repository"));

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

    let mut cmd = Command::cargo_bin("gitsync")?;
    cmd.arg("--dry-run").current_dir(path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to get main remote"));

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

    let mut cmd = Command::cargo_bin("gitsync")?;
    cmd.arg("--dry-run").arg("--verbose").current_dir(path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Using remote: origin"))
        .stdout(predicate::str::contains("Default branch: main"))
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

    let mut cmd = Command::cargo_bin("gitsync")?;
    cmd.arg("--verbose").arg("--dry-run").current_dir(path);

    let output = cmd.output()?;

    // Check that verbose output contains expected messages
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Using remote: origin"));
    assert!(stdout.contains("Default branch: main"));
    assert!(stdout.contains("Current branch: main"));

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

    let mut cmd = Command::cargo_bin("gitsync")?;
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
