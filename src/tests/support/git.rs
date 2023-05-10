use git2::{Commit, Repository};
use tempfile::{tempdir, TempDir};

pub(crate) fn create_bare_repo() -> Result<(TempDir, Repository), git2::Error> {
    println!("Creating bare repository for testing in system temporary");
    let tmp_dir = tempdir().map_err(|e| git2::Error::from_str(&format!("{}", e)))?;
    let repo_path = tmp_dir.path().join("my-repo.git");
    let repo = Repository::init_bare(&repo_path)?;
    println!("Bare repo created at {:?}", repo_path);
    Ok((tmp_dir, repo))
}

pub(crate) fn create_commit<'repo>(
    repo: &'repo Repository,
    message: &str,
    parents: &[&Commit],
    update_ref: Option<&str>,
) -> Result<Commit<'repo>, git2::Error> {
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = repo.signature()?;
    let object_id = repo.commit(update_ref, &sig, &sig, message, &tree, parents)?;
    let commit = repo.find_commit(object_id)?;
    Ok(commit)
}

pub(crate) fn create_branch_with_commit(
    repo: &Repository,
    branch_name: &str,
    message: &str,
    branch_target: Option<&Commit>,
) -> Result<(), git2::Error> {
    let branch = create_branch(repo, branch_name, branch_target)?;
    create_commit(
        repo,
        message,
        &[&branch.get().peel_to_commit().unwrap()],
        Some(branch.get().name().unwrap()),
    )?;
    Ok(())
}

pub(crate) fn create_branch<'repo>(
    repo: &'repo Repository,
    branch_name: &str,
    branch_target: Option<&Commit>,
) -> Result<git2::Branch<'repo>, git2::Error> {
    let branch = repo.branch(
        branch_name,
        branch_target.unwrap_or(&repo.head().unwrap().peel_to_commit().unwrap()),
        false,
    )?;
    Ok(branch)
}

pub(crate) fn assert_result(
    result: &Vec<crate::analysis::MergeAnalysisResult>,
    from_branch: &str,
    to_branch: &str,
    expected_status: crate::analysis::MergeAnalysisStatus,
) {
    let found_matching: u32 = result
        .iter()
        .map(|merge_analysis| {
            if merge_analysis.from_branch == from_branch && merge_analysis.to_branch == to_branch {
                assert_eq!(merge_analysis.status, expected_status);
                return 1;
            }
            0
        })
        .sum();
    assert!(found_matching == 1);
}
