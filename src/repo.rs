use eyre::{Context, Result};
use git2::Repository;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{env, fs};

use crate::clone::clone_repo;
use crate::interactive::Repo;
use crate::Probranchinator;

impl Repo for Probranchinator {
    fn get_repo(&self, remote_url: &str) -> Result<(Repository, PathBuf, bool)> {
        // Create the directory for the repositories under the system temporary directory
        let mut tmp_path = env::temp_dir();
        tmp_path.push("probranchinator");
        fs::create_dir_all(&tmp_path)
            .with_context(|| format!("Failed to create directory {:?}", tmp_path))?;

        // Generate subfolder name from hash of git remote url
        let mut hasher = DefaultHasher::new();
        remote_url.hash(&mut hasher);
        let subfolder_name = format!("{:x}", hasher.finish());

        // Create the full path to the new folder
        tmp_path.push(subfolder_name);

        let have_cached_repo = tmp_path.is_dir();

        if !have_cached_repo {
            // currently all kinds of clones via git2-rs or gitoxide
            // are in various states of being broken either on certain
            // repositories or on certain platforms:
            // git2 cannot be built for macos because it needs openssl and cross does not provide
            // also git2 cannot deal with SSH at the moment on windows
            // gitoxide fails on some random repositories with "not supported" errors
            // hence we use the git command line tool for now only for cloning
            clone_repo(remote_url, &tmp_path)?;
        } else {
            // fetch using git command line tool
            // again this is due to none of libraries being able to properly fetch on all platforms
            let mut cmd = std::process::Command::new("git");
            cmd.arg("fetch");
            cmd.arg("origin");
            cmd.current_dir(&tmp_path);
            let child = cmd.spawn()?;
            let output = child.wait_with_output()?;
            if !output.status.success() {
                eyre::bail!("Failed to fetch repository");
            }

            // then prune all branches that are not on origin anymore
            // also using git command line tool, because git2-rs
            // fails with "this remote has never connected", probably
            // due to the fact that we cloned with git command line tool

            let mut cmd = std::process::Command::new("git");
            cmd.arg("remote");
            cmd.arg("prune");
            cmd.arg("origin");
            cmd.current_dir(&tmp_path);
            let child = cmd.spawn()?;
            let output = child.wait_with_output()?;
            if !output.status.success() {
                eyre::bail!("Failed to prune repository");
            }
        }

        let repo = Repository::open(&tmp_path).with_context(|| {
            format!(
                "Failed to open repository in directory {:?}",
                tmp_path.display()
            )
        })?;

        Ok((repo, tmp_path, have_cached_repo))
    }
}
