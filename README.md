# GitSync

A CLI tool that syncs local git branches with their remote counterparts. Inspired by [`hub sync`](https://github.com/github/hub) and [`git-sync`](https://github.com/jacobwgillespie/git-sync).

## Features

- **Fast-forward branches**: Automatically advance local branches that are behind their upstream
- **Delete merged branches**: Remove branches that have been merged into your default branch
- **Delete stale branches**: Remove branches whose upstream has been deleted
- **Safety first**: Confirmation prompts before any deletions
- **Dry-run mode**: Preview changes before applying them
- **Smart filtering**: Only operates on branches with tracking remotes

## Installation

### From Source

```bash
git clone https://github.com/your-username/gitsync.git
cd gitsync
cargo build --release
# Binary will be at ./target/release/gitsync
```

## Usage

### Basic Usage (Interactive)

```bash
gitsync
```

Shows proposed changes and prompts for confirmation before deleting branches.

### Preview Changes (Dry-Run)

```bash
gitsync --dry-run
```

Shows what would be done without making any changes.

### Specify a Different Default Branch

```bash
gitsync --default-branch develop
```

By default, merged branch detection checks against `main`. Use this to check against a different branch.

### Specify a Different Repository

```bash
gitsync -C /path/to/repo
```

## Examples

### Clean up After Merging PRs

After merging several feature branches into `main`, your local might have branches like `feature1`, `feature2` that are merged but not deleted locally.

```bash
$ gitsync --dry-run
Opening repository at .
Fetching remotes...

Proposed changes:
  • feature1: Delete (merged into main)
  • feature2: Delete (merged into main)
  • bugfix: Fast-forward from origin/bugfix

[DRY RUN] No changes applied
```

Run without `--dry-run` and confirm to apply.

### Update Stale Branches

Your colleague deleted their branch on the remote, but you still have a local copy:

```bash
$ gitsync
Opening repository at .
Fetching remotes...

Proposed changes:
  • old-feature: Delete (upstream gone)

1 branch(es) will be deleted. Continue? (y/n) y
Applying changes...
  ✓ Deleted old-feature

✓ Done!
```

## How It Works

1. **Fetch**: Fetches from `origin` to update remote tracking branches
2. **Detect**: Identifies branches with tracking remotes and checks for:
   - Branches behind their upstream (candidates for fast-forward)
   - Branches merged into your default branch
   - Branches whose upstream no longer exists
3. **Prompt**: Shows proposed changes and asks for confirmation (unless `--dry-run`)
4. **Apply**: Fast-forwards and deletes branches as confirmed

## Safety

- The current checked-out branch is never deleted or modified
- Only branches with tracking remotes are considered (local-only branches are ignored)
- Confirmation is required before any deletions
- Use `--dry-run` to preview changes first

## Requirements

- Rust 1.56+ (for building from source)
- git2 dependencies (handled automatically by cargo)

## License

MIT
