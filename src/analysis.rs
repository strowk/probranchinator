use crate::{
    recent::get_recent_branches,
    result::{MergeAnalysisResult, MergeAnalysisStatus},
};
use eyre::Result;
use git2::Repository;
use indicatif::ProgressStyle;
use std::time::Duration;

pub(crate) fn analyse(
    repo: Repository,
    branches: Vec<String>,
    recent: usize,
) -> Result<Vec<MergeAnalysisResult>> {
    let mut answer: Vec<MergeAnalysisResult> = Vec::new();

    // get recent branches if none are provided
    let branches = match branches[..] {
        [] => get_recent_branches(&repo, recent)?,
        _ => branches,
    };

    // prepare progress indicator
    let branches_length = branches.len();
    let progress = indicatif::ProgressBar::new(
        // would be comparing each branch to each other branch except itself
        (branches_length * branches_length - branches_length).try_into()?,
    )
    .with_finish(indicatif::ProgressFinish::AndLeave);
    progress.enable_steady_tick(Duration::from_millis(100));
    progress.set_prefix("[2/2]");
    progress.set_style(
        ProgressStyle::with_template(
            "{prefix:.cyan/blue} {spinner} Analysing branches... [{bar:!20}] {wide_msg}",
        )?
        .progress_chars("=>-"),
    );

    let starting_head = repo.head()?;

    for i in 0..branches_length {
        for j in 0..branches_length {
            if i == j {
                continue;
            }

            let into_branch = &branches[j];
            let from_branch = &branches[i];

            progress.inc(1);
            progress.length().and_then(|length| {
                Some(progress.set_message(format!(
                    "{}/{}: [{} -> {}]",
                    progress.position(),
                    length,
                    from_branch,
                    into_branch
                )))
            });
            let their_head =
                repo.find_reference(&format!("refs/remotes/origin/{}", from_branch))?;
            let our_head = repo.find_reference(&format!("refs/remotes/origin/{}", into_branch))?;
            let their_commit = repo.reference_to_annotated_commit(&their_head)?;
            let analysis = repo.merge_analysis_for_ref(&our_head, &[&their_commit])?;
            let mut result = MergeAnalysisResult {
                from_branch: from_branch.clone(),
                to_branch: into_branch.clone(),
                status: MergeAnalysisStatus::Unknown,
            };
            if analysis.0.is_fast_forward() {
                result.status = MergeAnalysisStatus::FastForward;
            } else if analysis.0.is_normal() {
                let out_commit = repo.reference_to_annotated_commit(&our_head)?;
                match check_normal_merge(&repo, &their_commit, &out_commit) {
                    Ok(status) => result.status = status,
                    Err(error) => {
                        result.status = MergeAnalysisStatus::Error {
                            message: error.message().to_owned(),
                        }
                    }
                }

                // this is to clean up the repo after the merge, which can leave dirty files
                let starting_head_commit = starting_head.peel_to_commit()?;
                repo.reset(
                    starting_head_commit.as_object(),
                    git2::ResetType::Hard,
                    None,
                )?;
            } else if analysis.0.is_up_to_date() {
                result.status = MergeAnalysisStatus::UpToDate;
            } else if analysis.0.is_none() {
                result.status = MergeAnalysisStatus::None;
            } else {
                result.status = MergeAnalysisStatus::Unknown;
            }
            answer.push(result);
        }
    }

    // finish progress indicator and display elapsed time
    progress.set_style(ProgressStyle::with_template(&format!(
        "{} branches analysed in {{elapsed}}",
        branches_length
    ))?);
    progress.finish_using_style();

    Ok(answer)
}

fn check_normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<MergeAnalysisStatus, git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(MergeAnalysisStatus::Conflicts);
    }
    Ok(MergeAnalysisStatus::Normal)
}
