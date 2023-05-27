use std::path::PathBuf;

use crate::{interactive::Repo, recent, tests::support};

#[test]
fn test_recent() -> eyre::Result<()> {
    let probrahcninator = crate::Probranchinator {};

    let (_tmp_dir, origin) = support::git::create_bare_repo()?;

    let remote_url = format!("file:///{}", PathBuf::from(origin.path()).display());
    println!("Using bare repo from {:?}", remote_url);

    // initialize first commit in origin repository
    support::git::create_commit(&origin, "initial commit", &[], Some("HEAD"), None)?;

    // Create a branch in the origin repository
    let branch_name = "test-branch";

    // Wait a second to ensure that the branch is created at a different time

    std::thread::sleep(std::time::Duration::from_secs(1));

    support::git::create_branch_with_commit(&origin, branch_name, "first commit", None)?;

    // Clone the repository
    let (cloned_repo, _, _) = probrahcninator.get_repo(&remote_url)?;

    // Get recent branches
    let recent_branches = recent::get_recent_branches(&cloned_repo, 2)?;

    // Check that first branch is the one we just created
    assert_eq!(recent_branches[0], branch_name);

    // Check that second branch is master
    assert_eq!(recent_branches[1], "master");

    Ok(())
}
