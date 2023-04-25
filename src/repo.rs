use anyhow::{Context, Result};
use git2::{Cred, RemoteCallbacks, Repository};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::{env, fs};

use crate::clone::clone_repo;

pub(crate) fn get_repo(remote_url: &str) -> Result<Repository> {
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

    let repo = if have_cached_repo {
        // If the folder already exists, open the repository in it
        Repository::open(&tmp_path).with_context(|| {
            format!(
                "Failed to open repository in directory {:?}",
                tmp_path.display()
            )
        })?
    } else {
        // Otherwise, clone the repository to the new folder

        // currently all kinds of clones via git2-rs or gitoxide
        // are in various states of being broken either on certain
        // repositories or on certain platforms:
        // git2 cannot be built for macos because it needs openssl and cross does not provide
        // also git2 cannot deal with SSH at the moment on windows
        // gitoxide fails on some random repositories with "not supported" errors
        // hece we use the git command line tool for now only for cloning
        clone_repo(remote_url, &tmp_path)?;

        // and then we assume it was cloned Ok and just open it

        Repository::open(&tmp_path).with_context(|| {
            format!(
                "Failed to open repository in directory {:?}",
                tmp_path.display()
            )
        })?

        // this was attempt to use git2-rs for cloning, could not build it for darwin ATM

        // Repository::clone(remote_url, &tmp_path).with_context(|| {
        //     format!(
        //         "Failed to clone repository from {:?} to {:?}",
        //         remote_url,
        //         tmp_path.display()
        //     )
        // })?

        // Using more complex clone options to allow for SSH authentication
        // It is broken at the moment on Windows, see
        // https://github.com/rust-lang/git2-rs/issues/937

        // let mut callbacks = RemoteCallbacks::new();
        // callbacks.credentials(|_url, username_from_url, _allowed_types| {
        //     Cred::ssh_key(
        //         username_from_url.unwrap(),
        //         None,
        //         Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
        //         None,
        //     )
        // });

        // // Prepare fetch options.
        // let mut fo = git2::FetchOptions::new();
        // fo.remote_callbacks(callbacks);

        // // Prepare builder.
        // let mut builder = git2::build::RepoBuilder::new();
        // builder.fetch_options(fo);

        // // Clone the project.
        // builder.clone(remote_url, &tmp_path).with_context(|| {
        //     format!(
        //         "Failed to clone repository from {:?} to {:?}",
        //         remote_url,
        //         tmp_path.display()
        //     )
        // })?
    };

    if have_cached_repo {
        // let mut remote = repo.find_remote("origin")?;
        // let mut fetch_options = git2::FetchOptions::new();
        // remote.fetch::<String>(&[], Some(&mut fetch_options), None)?;

        // fetch using git command line tool
        // again this is due to none of libraries being able to properly fetch on all platforms
        let mut cmd = std::process::Command::new("git");
        cmd.arg("fetch");
        cmd.arg("origin");
        cmd.current_dir(&tmp_path);
        let child = cmd.spawn()?;
        let output = child.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!("Failed to fetch repository");
        }
    }

    Ok(repo)
}
