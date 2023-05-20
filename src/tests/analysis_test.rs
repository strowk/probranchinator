use crate::analysis::analyse;
use crate::repo::get_repo;
use crate::result::MergeAnalysisStatus;
use crate::tests::support::git::{
    assert_result, create_bare_repo, create_branch, create_branch_with_commit, create_commit,
    create_repo,
};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[test]
fn test_analysis_one_branch() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // there is going to be one default master branch

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    // Check that the repository path contains "probranchinator" and is under the system temporary directory
    assert!(cloned_repo
        .path()
        .to_string_lossy()
        .contains("probranchinator"));

    assert!(cloned_repo.path().starts_with(env::temp_dir()));

    // Run analysis
    let result = analyse(cloned_repo, vec![], 2)?;

    // As there is only one branch, we expect no results, since there is nothing to merge
    assert_eq!(result.len(), 0);

    Ok(())
}

#[test]
fn test_analysis_up_to_date_and_fast_forward() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";

    create_branch_with_commit(&origin, branch_name, "first commit", None)?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    // Run analysis with 2 recent branches
    let result = analyse(cloned_repo, vec![], 2)?;

    // With two branches we expect two results
    assert_eq!(result.len(), 2);

    // Master merge should be up to date with test-branch
    // as there are no commits in master that are missing in test-branch
    assert_result(
        &result,
        "master",
        branch_name,
        MergeAnalysisStatus::UpToDate,
    );

    // Merging test-branch to master should be a fast-forward
    // as test-branch only has one commit that is missing in master
    // and there is no other difference between the branches
    assert_result(
        &result,
        branch_name,
        "master",
        MergeAnalysisStatus::FastForward,
    );

    Ok(())
}

#[test]
fn test_analysis_unrelated_branches() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";

    let commit = create_commit(&origin, "initial commit 2", &[], None, None)?;
    create_branch(&origin, branch_name, Some(&commit))?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    // Run analysis specifically with the test-branch and master
    let result = analyse(
        cloned_repo,
        vec![branch_name.to_string(), "master".to_string()],
        0,
    )?;

    // Check that master cannot be merged to test-branch and vice versa
    assert_result(
        &result,
        "master",
        branch_name,
        MergeAnalysisStatus::Error {
            message: "no merge base found".to_string(),
        },
    );
    assert_result(
        &result,
        branch_name,
        "master",
        MergeAnalysisStatus::Error {
            message: "no merge base found".to_string(),
        },
    );

    Ok(())
}

fn create_and_commit_file(
    repo: &git2::Repository,
    file_name: &str,
    content: &str,
    message: &str,
    branch: &str,
) -> eyre::Result<()> {
    // Create file
    let file_path = PathBuf::from(repo.path()).parent().unwrap().join(file_name);
    println!("Creating file {:?}", file_path);
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);

    // Stage and commit the file
    let mut index = repo.index()?;
    index.add_path(Path::new(file_name))?;
    index.write()?;

    // Create a commit in the master branch
    create_commit(
        repo,
        message,
        &[
            &repo
                .find_branch(branch, git2::BranchType::Local)?
                .get()
                .peel_to_commit()
                .unwrap(), // master branch HEAD
        ],
        Some(&format!("refs/heads/{}", branch)),
        Some(index),
    )?;
    Ok(())
}

#[test]
fn test_analysis_normal_merge_no_conflicts() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // Initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";
    create_branch(&origin, branch_name, None)?;

    create_and_commit_file(&origin, "test.txt", "test", "test commit", "master")?;

    create_and_commit_file(&origin, "test2.txt", "test", "test commit", branch_name)?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    println!("Cloned repo at {:?}", cloned_repo.path());

    // Run analysis specifically with the test-branch and master
    let result = analyse(
        cloned_repo,
        vec![branch_name.to_string(), "master".to_string()],
        0,
    )?;

    // Check that master can be normally merged to test-branch and vice versa
    assert_result(&result, "master", branch_name, MergeAnalysisStatus::Normal);
    assert_result(&result, branch_name, "master", MergeAnalysisStatus::Normal);

    Ok(())
}

#[test]
fn test_analysis_normal_merge_with_conflicts() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // Initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";
    create_branch(&origin, branch_name, None)?;

    create_and_commit_file(&origin, "test.txt", "text 1", "test commit", "master")?;

    create_and_commit_file(&origin, "test.txt", "text 2", "test commit", branch_name)?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    println!("Cloned repo at {:?}", cloned_repo.path());

    // Run analysis specifically with the test-branch and master
    let result = analyse(
        cloned_repo,
        vec![branch_name.to_string(), "master".to_string()],
        0,
    )?;

    // Check that master causes conflicts when merged to test-branch and vice versa
    assert_result(
        &result,
        "master",
        branch_name,
        MergeAnalysisStatus::Conflicts,
    );
    assert_result(
        &result,
        branch_name,
        "master",
        MergeAnalysisStatus::Conflicts,
    );

    Ok(())
}
