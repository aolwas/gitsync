use std::process::Command;
use tempfile::TempDir;
use std::path::Path;

use gitsync::git::Repo;
use gitsync::sync::Syncer;

fn run(cmd: &mut Command, cwd: &Path) {
    let status = cmd.current_dir(cwd).status().expect("failed to run command");
    assert!(status.success());
}

#[test]
fn test_integration_full_flow() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();

    let remote_dir = base.join("remote.git");
    let local_dir = base.join("local");

    // init bare
    run(Command::new("git").arg("init").arg("--bare").arg(&remote_dir), base);

    // clone
    run(Command::new("git").arg("clone").arg(&remote_dir).arg(&local_dir), base);

    // configure git user in the cloned repo
    run(Command::new("git").arg("-C").arg(&local_dir).arg("config").arg("user.email").arg("test@example.com"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("config").arg("user.name").arg("Test User"), base);

    // initial commit on main
    run(Command::new("git").arg("-C").arg(&local_dir).arg("checkout").arg("-b").arg("main"), base);
    std::fs::write(local_dir.join("file.txt"), "initial").unwrap();
    run(Command::new("git").arg("-C").arg(&local_dir).arg("add").arg("file.txt"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("commit").arg("-m").arg("initial"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("push").arg("-u").arg("origin").arg("main"), base);

    // feature1 branch ahead and pushed
    run(Command::new("git").arg("-C").arg(&local_dir).arg("checkout").arg("-b").arg("feature1"), base);
    std::fs::write(local_dir.join("feature.txt"), "feature").unwrap();
    run(Command::new("git").arg("-C").arg(&local_dir).arg("add").arg("feature.txt"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("commit").arg("-m").arg("feature work"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("push").arg("-u").arg("origin").arg("feature1"), base);

    // Merge feature1 into main
    run(Command::new("git").arg("-C").arg(&local_dir).arg("checkout").arg("main"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("merge").arg("feature1").arg("--no-edit"), base);

    // old-branch: push then make local behind
    run(Command::new("git").arg("-C").arg(&local_dir).arg("checkout").arg("-b").arg("old-branch"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("push").arg("-u").arg("origin").arg("old-branch"), base);
    // make local behind by resetting to previous commit on this branch
    run(Command::new("git").arg("-C").arg(&local_dir).arg("reset").arg("--hard").arg("HEAD~1"), base);

    // merged-branch pushed
    run(Command::new("git").arg("-C").arg(&local_dir).arg("checkout").arg("-b").arg("merged-branch"), base);
    run(Command::new("git").arg("-C").arg(&local_dir).arg("push").arg("-u").arg("origin").arg("merged-branch"), base);

    // Now run detection
    let repo = Repo::open(&local_dir).expect("open repo");
    let syncer = Syncer::new(repo, "main").expect("create syncer");
    let changes = syncer.find_changes().expect("find changes");

    // Expect feature1 marked as merged -> deletion, and old-branch fast-forward
    let mut has_feature1_delete = false;
    let mut has_old_ff = false;
    for c in changes {
        if c.branch == "feature1" {
            match c.action {
                gitsync::sync::Action::DeleteMerged { .. } => has_feature1_delete = true,
                _ => {}
            }
        }
        if c.branch == "old-branch" {
            match c.action {
                gitsync::sync::Action::FastForward { .. } => has_old_ff = true,
                _ => {}
            }
        }
    }

    assert!(has_feature1_delete, "feature1 should be detected as merged");
    assert!(has_old_ff, "old-branch should be detected as fast-forward candidate");
}
