use anyhow::{Context, Result};
use git2::{Cred, RemoteCallbacks, Repository};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::{env, fs};

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

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
                None,
            )
        });

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        // Clone the project.
        builder.clone(remote_url, &tmp_path).with_context(|| {
            format!(
                "Failed to clone repository from {:?} to {:?}",
                remote_url,
                tmp_path.display()
            )
        })?
    };

    if have_cached_repo {
        let mut remote = repo.find_remote("origin")?;
        let mut fetch_options = git2::FetchOptions::new();
        remote.fetch::<String>(&[], Some(&mut fetch_options), None)?;
    }

    Ok(repo)
}
