use crate::repo::get_repo;
use git2::Repository;
use std::env;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

fn create_bare_repo() -> Result<(TempDir, Repository), git2::Error> {
    println!("Creating bare repository for testing in system temporary");
    let tmp_dir = tempdir().map_err(|e| git2::Error::from_str(&format!("{}", e)))?;
    let repo_path = tmp_dir.path().join("my-repo.git");
    let repo = Repository::init_bare(&repo_path)?;
    println!("Bare repo created at {:?}", repo_path);
    Ok((tmp_dir, repo))
}

#[test]
fn test_get_repo() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp_dir, origin) = create_bare_repo()?;
    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // Test cloning the repository for the first time
    let cloned_repo_1 = get_repo(&remote_url)?;

    // Check that the repository path contains "probranchinator" and is under the system temporary directory
    assert!(cloned_repo_1
        .path()
        .to_string_lossy()
        .contains("probranchinator"));
    assert!(cloned_repo_1.path().starts_with(env::temp_dir()));

    // Test opening the existing repository in the second call
    let cloned_repo_2 = get_repo(&remote_url)?;
    assert_eq!(cloned_repo_2.path(), cloned_repo_1.path());

    // Test cloning a different repository
    let (_tmp_dir2, repo2) = create_bare_repo()?;
    let remote_url2 = format!("file:///{}", PathBuf::from(repo2.path()).display());
    let cloned_repo_3 = get_repo(&remote_url2)?;
    assert_ne!(cloned_repo_3.path(), cloned_repo_2.path());

    Ok(())
}
