use std::fmt::Display;

use eyre::Result;
use git2::Repository;

use crate::recent::get_recent_branches;

pub(crate) enum MergeAnalysisStatus {
    UpToDate,
    FastForward,
    None,
    Error { message: String },
    Normal,
    Unknown,
    Conflicts,
}

impl Display for MergeAnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MergeAnalysisStatus::UpToDate => {
                write!(f, "✅✅ No changes: already up-to-date.")
            }
            MergeAnalysisStatus::FastForward => {
                write!(f, "🚀✅ No confilcts: fast-forward merge is possible.")
            }
            MergeAnalysisStatus::None => {
                write!(f, "❌❌ No merge is possible - analysis gave none.")
            }
            MergeAnalysisStatus::Error { message } => {
                write!(f, "❌❌ No merge is possible - {}.", message)
            }
            MergeAnalysisStatus::Unknown => write!(f, "❌🤔 Unknown merge analysis result."),
            MergeAnalysisStatus::Conflicts => {
                write!(f, "🚧🔧 Found conflicts, have to resolve them manually.")
            }
            MergeAnalysisStatus::Normal => {
                write!(f, "🤝✅ No conflicts: automatic merge is possible.")
            }
        }
    }
}

pub(crate) struct MergeAnalysisResult {
    pub from_branch: String,
    pub to_branch: String,
    pub status: MergeAnalysisStatus,
}

impl MergeAnalysisResult {
    // returns vector of Strings to show in the table
    pub(crate) fn to_table_row(&self) -> Vec<String> {
        vec![
            format!("{}", self.status),
            format!("{} -> {}", self.from_branch, self.to_branch),
        ]
    }
}

impl Display for MergeAnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} -> {} : {}",
            self.from_branch, self.to_branch, self.status
        )
    }
}

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

    let starting_head = repo.head()?;

    for i in 0..branches.len() {
        for j in 0..branches.len() {
            if i == j {
                continue;
            }
            let into_branch = &branches[j];
            let from_branch = &branches[i];

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
