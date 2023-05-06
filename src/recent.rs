use eyre::{Context, Result};
use git2::{BranchType, Repository};

pub(crate) fn get_recent_branches(repo: &Repository, recent: usize) -> Result<Vec<String>> {
    // firstly we collect all branches
    let mut branches = repo
        .branches(Some(BranchType::Remote))
        .context("failed to retrieve git branches")?
        // exiting if could not get any of branches
        .collect::<Result<Vec<_>, _>>()
        .context("failed to retrieve some of git branches")?
        .into_iter()
        // extract their names
        .flat_map(|(branch, _)| match branch.name() {
            Ok(Some(name)) => {
                let name = name.to_owned();
                Some((branch, name))
            }
            Err(e) => {
                println!(
                    "Error getting one of branches name, skip that branch {:?}",
                    e
                );
                None
            }
            Ok(None) => {
                println!("Error getting one of branches name, skip that branch");
                None
            }
        })
        .filter(|(_, name)| name != &"origin/HEAD")
        // then we get the last commit of each branch
        .map(|(branch, name)| match branch.get().peel_to_commit() {
            Ok(commit) => Ok((commit, name)),
            Err(e) => Err(e),
        })
        // exit if there was an error getting the commit
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to read last commits of git branches")?;

    // then we sort them by last commit date, exiting on error
    branches.sort_unstable_by_key(|(commit, _)| commit.committer().when());

    // returns branch names without "origin/" prefix
    Ok(branches
        .iter()
        .map(|(_, name)| name.replacen("origin/", "", 1))
        .take(recent)
        .collect())
}
