use crate::repo::get_repo;
use crate::tests::support::git::{
    assert_result, create_bare_repo, create_branch, create_branch_with_commit, create_commit,
};
use std::env;
use std::path::PathBuf;

#[test]
fn test_analysis_one_branch() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"))?;

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
    let result = crate::analysis::analyse(cloned_repo, vec![], 2)?;

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
    create_commit(&origin, "initial commit", &[], Some("HEAD"))?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";

    create_branch_with_commit(&origin, branch_name, "first commit", None)?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    // Run analysis with 2 recent branches
    let result = crate::analysis::analyse(cloned_repo, vec![], 2)?;

    // With two branches we expect two results
    assert_eq!(result.len(), 2);

    // Master merge should be up to date with test-branch
    // as there are no commits in master that are missing in test-branch
    assert_result(
        &result,
        "master",
        branch_name,
        crate::analysis::MergeAnalysisStatus::UpToDate,
    );

    // Merging test-branch to master should be a fast-forward
    // as test-branch only has one commit that is missing in master
    // and there is no other difference between the branches
    assert_result(
        &result,
        branch_name,
        "master",
        crate::analysis::MergeAnalysisStatus::FastForward,
    );

    Ok(())
}

#[test]
fn test_analysis_unrelated_branches() -> eyre::Result<()> {
    let (_tmp_dir, origin) = create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    create_commit(&origin, "initial commit", &[], Some("HEAD"))?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";

    let commit = create_commit(&origin, "initial commit 2", &[], None)?;
    create_branch(&origin, branch_name, Some(&commit))?;

    // Clone the repository
    let (cloned_repo, _, _) = get_repo(&remote_url)?;

    // Run analysis specifically with the test-branch and master
    let result = crate::analysis::analyse(
        cloned_repo,
        vec![branch_name.to_string(), "master".to_string()],
        0,
    )?;

    // Check that master cannot be merged to test-branch and vice versa
    assert_result(
        &result,
        "master",
        branch_name,
        crate::analysis::MergeAnalysisStatus::Error {
            message: "no merge base found".to_string(),
        },
    );
    assert_result(
        &result,
        branch_name,
        "master",
        crate::analysis::MergeAnalysisStatus::Error {
            message: "no merge base found".to_string(),
        },
    );

    Ok(())
}
