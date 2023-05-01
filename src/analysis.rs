use anyhow::Result;
use git2::{BranchType, Repository};

// TODO: cleanup the code - remove the unwraps and add error handling

// fn get_sorted_branches(repo_path: &str) -> Result<Vec<String>, git2::Error> {
fn get_recent_branches(repo: &Repository) -> Result<Vec<String>> {
    // TODO: reverse the order of the branches so that the most recent ones are first
    // and remove reverse() from main()

    // TODO: take only recent f.e. 10 branches

    // let repo = Repository::open(repo_path)?;
    let mut branches = repo
        .branches(Some(BranchType::Remote))?
        .map(|b| b.unwrap())
        .filter(|b| b.0.name().unwrap().unwrap() != "origin/HEAD")
        .collect::<Vec<_>>();

    println!("Received branches");
    for (branch, branch_type) in branches.iter() {
        println!(
            "branch - {} ({:?})",
            branch.name().unwrap().unwrap(),
            branch_type
        );
    }

    branches.sort_by_key(|b| b.0.get().peel_to_commit().unwrap().committer().when());

    // maps branches to branch name and removes "origin/" prefix
    Ok(branches
        .into_iter()
        .map(|(branch, _)| {
            branch
                .name()
                .unwrap()
                .unwrap()
                .to_string()
                .replacen("origin/", "", 1)
        })
        .collect())
}

pub(crate) fn analyse(repo: Repository, branches: Vec<String>) -> Result<Vec<Vec<String>>> {
    let mut answer: Vec<Vec<String>> = Vec::new();

    // get recent branches if none are provided
    let branches = match branches[..] {
        [] => get_recent_branches(&repo)?,
        _ => branches,
    };

    let starting_head = repo.head()?;

    for i in 0..branches.len() {
        //  result is not actually always symmetric
        // for j in i + 1..args.len() {
        for j in 0..branches.len() {
            if i == j {
                continue;
            }
            let mut row: Vec<String> = Vec::new();

            let into_branch = &branches[j];
            let from_branch = &branches[i];

            let their_head =
                repo.find_reference(&format!("refs/remotes/origin/{}", from_branch))?;
            let our_head = repo.find_reference(&format!("refs/remotes/origin/{}", into_branch))?;
            let their_commit = repo.reference_to_annotated_commit(&their_head)?;
            let analysis = repo.merge_analysis_for_ref(&our_head, &[&their_commit])?;

            // println!("\nComparing {} with {}:", into_branch, from_branch);
            if analysis.0.is_fast_forward() {
                row.push("🚀✅ No confilcts: fast-forward merge is possible.".to_string());
            } else if analysis.0.is_normal() {
                // println!("🛠️  A normal merge is possible."); // ⚠️ // 🚧 // 💣
                let out_commit = repo.reference_to_annotated_commit(&our_head)?;
                match check_normal_merge(&repo, &their_commit, &out_commit) {
                    Ok(result) => row.push(result.to_string()),
                    Err(error) => {
                        row.push(format!("❌❌ No merge is possible - {}.", error.message()))
                    }
                }

                // this is to clean up the repo after the merge, which can leave dirty files
                let starting_head_commit = starting_head.peel_to_commit()?;
                repo.reset(
                    starting_head_commit.as_object(),
                    git2::ResetType::Hard,
                    None,
                )?;
            // TODO - figure out if there are conflicts
            } else if analysis.0.is_up_to_date() {
                row.push("✅✅ No changes: the branches are already up-to-date.".to_string());
            } else if analysis.0.is_none() {
                row.push("❌❌ No merge is possible - analysis gave none.".to_string());
            } else {
                row.push("❌🤔 Unknown merge analysis result.".to_string());
            }
            row.push(format!(
                "{} into {}",
                from_branch.clone(),
                into_branch.clone()
            ));
            answer.push(row);
        }
    }

    Ok(answer)
}

fn check_normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<&'static str, git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok("🚧🔧 Found conflicts, have to resolve them manually.");
    }
    return Ok("🚧🍀 Found conflicts, but can resolve them automatically.");
}
