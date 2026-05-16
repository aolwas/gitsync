use anyhow::{anyhow, Result};
use git2::{Repository, BranchType};
use std::path::Path;

pub struct Repo {
    repo: Repository,
}

impl Repo {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Repo { repo })
    }

    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let name = head.shorthand()
            .ok_or_else(|| anyhow!("Could not determine current branch"))?;
        Ok(name.to_string())
    }

    pub fn local_branches(&self) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();
        let branch_iter = self.repo.branches(Some(BranchType::Local))?;

        for result in branch_iter {
            let (branch, _) = result?;
            if let Ok(name) = branch.name() {
                if let Some(name) = name {
                    if let Ok(upstream) = branch.upstream() {
                        if let Ok(upstream_name) = upstream.name() {
                            if let Some(upstream_name) = upstream_name {
                                branches.push(Branch {
                                    name: name.to_string(),
                                    tracking: Some(upstream_name.to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(branches)
    }

    pub fn can_fastforward(&self, local_name: &str, upstream: &str) -> Result<bool> {
        let local_ref = self.repo.find_reference(&format!("refs/heads/{}", local_name))?;
        let upstream_ref = self.repo.find_reference(&format!("refs/remotes/{}", upstream))?;

        let local_oid = local_ref.target().ok_or_else(|| anyhow!("Invalid local ref"))?;
        let upstream_oid = upstream_ref.target().ok_or_else(|| anyhow!("Invalid upstream ref"))?;

        Ok(local_oid != upstream_oid)
    }

    pub fn fastforward(&self, branch_name: &str, upstream: &str) -> Result<bool> {
        let upstream_ref = self.repo.find_reference(&format!("refs/remotes/{}", upstream))?;
        let upstream_oid = upstream_ref.target().ok_or_else(|| anyhow!("Invalid upstream ref"))?;

        let mut local_ref = self.repo.find_reference(&format!("refs/heads/{}", branch_name))?;
        let local_oid = local_ref.target().ok_or_else(|| anyhow!("Invalid local ref"))?;

        if local_oid == upstream_oid {
            return Ok(false);
        }

        local_ref.set_target(upstream_oid, "fast-forward")?;
        Ok(true)
    }

    pub fn is_merged(&self, branch_name: &str, into_branch: &str) -> Result<bool> {
        let branch_ref = self.repo.find_reference(&format!("refs/heads/{}", branch_name))?;
        let branch_oid = branch_ref.target().ok_or_else(|| anyhow!("Invalid branch ref"))?;

        let into_ref = self.repo.find_reference(&format!("refs/heads/{}", into_branch))?;
        let into_oid = into_ref.target().ok_or_else(|| anyhow!("Invalid target ref"))?;

        if branch_oid == into_oid {
            return Ok(true);
        }

        // Use merge_base to check if branch is ancestor of into_branch
        match self.repo.merge_base(branch_oid, into_oid) {
            Ok(merge_oid) => Ok(merge_oid == branch_oid),
            Err(_) => Ok(false),
        }
    }

    pub fn upstream_exists(&self, upstream: &str) -> Result<bool> {
        self.repo
            .find_reference(&format!("refs/remotes/{}", upstream))
            .map(|_| true)
            .or_else(|_| Ok(false))
    }

    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        let mut branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn fetch(&self) -> Result<()> {
        match self.repo.find_remote("origin") {
            Ok(mut remote) => {
                let refspecs: &[&str] = &[];
                remote.fetch(refspecs, None, None)?;
                Ok(())
            }
            Err(e) => {
                // If origin doesn't exist, that's okay - maybe it's a local-only repo
                eprintln!("Warning: Could not fetch from 'origin': {}", e);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub tracking: Option<String>,
}
