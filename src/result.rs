use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Tabled, Debug)]
pub(crate) struct MergeAnalysisResult {
    pub from_branch: String,
    pub to_branch: String,
    pub status: MergeAnalysisStatus,
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_merge_analysis_status_display() {
        use super::MergeAnalysisStatus;
        assert_eq!(
            format!("{}", MergeAnalysisStatus::UpToDate),
            "✅✅ No changes: already up-to-date."
        );
        assert_eq!(
            format!("{}", MergeAnalysisStatus::FastForward),
            "🚀✅ No confilcts: fast-forward merge is possible."
        );
        assert_eq!(
            format!("{}", MergeAnalysisStatus::None),
            "❌❌ No merge is possible - analysis gave none."
        );
        assert_eq!(
            format!(
                "{}",
                MergeAnalysisStatus::Error {
                    message: "error".to_owned()
                }
            ),
            "❌❌ No merge is possible - error."
        );
        assert_eq!(
            format!("{}", MergeAnalysisStatus::Unknown),
            "❌🤔 Unknown merge analysis result."
        );
        assert_eq!(
            format!("{}", MergeAnalysisStatus::Conflicts),
            "🚧🔧 Found conflicts, have to resolve them manually."
        );
        assert_eq!(
            format!("{}", MergeAnalysisStatus::Normal),
            "🤝✅ No conflicts: automatic merge is possible."
        );
    }

    #[test]
    fn test_merge_analysis_result_display() {
        use super::{MergeAnalysisResult, MergeAnalysisStatus};
        assert_eq!(
            format!(
                "{}",
                MergeAnalysisResult {
                    from_branch: "from".to_owned(),
                    to_branch: "to".to_owned(),
                    status: MergeAnalysisStatus::UpToDate
                }
            ),
            "from -> to : ✅✅ No changes: already up-to-date."
        );
    }
}
