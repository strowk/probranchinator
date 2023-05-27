use std::time::Duration;

use indicatif::{ProgressFinish, ProgressStyle};

use crate::{
    cli::{Args, BooleanCLI, OutputType},
    interactive::{run_interactive, Analyzer, Repo},
};

pub(crate) fn run_probranchinator<A: Analyzer, R: Repo>(
    Args {
        remote,
        branches,
        recent,
        output,
        pretty,
    }: Args,
    stdout: &mut dyn std::io::Write,
    analyzer: &A,
    repo: &R,
) -> eyre::Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner()
        .with_prefix("[1/2]")
        .with_message("Retrieving repository...")
        .with_finish(ProgressFinish::AndLeave)
        .with_style(ProgressStyle::with_template(
            "{prefix:.cyan/blue} {spinner} {msg}",
        )?);
    spinner.enable_steady_tick(Duration::from_millis(100));
    let (repo, tmp_path, have_cached_repo) = repo.get_repo(&remote)?;

    spinner.set_style(ProgressStyle::with_template(
        "Retrieved repository in {elapsed}",
    )?);

    spinner.finish();

    log::info!(
        "Using repository cache at {:?} (cached: {})",
        tmp_path,
        have_cached_repo
    );

    let answer = analyzer.analyse(repo, branches, recent)?;

    match output {
        OutputType::Markdown => {
            let table = tabled::Table::new(answer)
                .with(tabled::settings::Style::markdown())
                .to_string();
            writeln!(stdout, "{}", table)?;
        }
        OutputType::Table => {
            let table = tabled::Table::new(answer).to_string();
            writeln!(stdout, "{}", table)?;
        }
        OutputType::Simple => {
            answer
                .iter()
                .map(|analysis_result| {
                    writeln!(stdout, "{}", analysis_result)?;
                    Ok(())
                })
                .collect::<std::io::Result<Vec<_>>>()?;
        }
        OutputType::Json => {
            if pretty == BooleanCLI::True {
                writeln!(stdout, "{}", serde_json::to_string_pretty(&answer)?)?;
            } else {
                writeln!(stdout, "{}", serde_json::to_string(&answer)?)?;
            }
        }
        OutputType::Interactive => {
            answer.iter().for_each(|analysis_result| {
                log::info!("{}", analysis_result);
            });
            run_interactive(answer)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        interactive::{MockAnalyzer, MockRepo},
        result::{MergeAnalysisResult, MergeAnalysisStatus},
    };

    use super::*;
    use serde_json::json;

    #[test]
    fn test_run_probranchinator_table() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mock_analyzer = two_branches_analyzer();
        let mock_repo = this_repository();

        // // call run_probranchinator with mocks and buffer
        run_probranchinator(
            Args {
                output: crate::cli::OutputType::Table,
                remote: "".to_string(),
                branches: vec![],
                pretty: crate::cli::BooleanCLI::False,
                recent: 0,
            },
            &mut buf,
            &mock_analyzer,
            &mock_repo,
        )?;

        // // check if output is text table with two analysis results
        let text = String::from_utf8(buf).unwrap();
        let expected = r#"
+-------------+-----------+----------------------------------------------------+
| from_branch | to_branch | status                                             |
+-------------+-----------+----------------------------------------------------+
| feature     | master    | ✅✅ No changes: already up-to-date.               |
+-------------+-----------+----------------------------------------------------+
| master      | feature   | 🚀✅ No confilcts: fast-forward merge is possible. |
+-------------+-----------+----------------------------------------------------+
"#
        .trim_start();

        assert_eq!(text, expected);

        Ok(())
    }

    #[test]
    fn test_run_probranchinator_markdown() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mock_analyzer = two_branches_analyzer();
        let mock_repo = this_repository();

        // // call run_probranchinator with mocks and buffer
        run_probranchinator(
            Args {
                output: crate::cli::OutputType::Markdown,
                remote: "".to_string(),
                branches: vec![],
                pretty: crate::cli::BooleanCLI::False,
                recent: 0,
            },
            &mut buf,
            &mock_analyzer,
            &mock_repo,
        )?;

        // // check if output is text table with two analysis results
        let text = String::from_utf8(buf).unwrap();
        let expected = r#"
| from_branch | to_branch | status                                             |
|-------------|-----------|----------------------------------------------------|
| feature     | master    | ✅✅ No changes: already up-to-date.               |
| master      | feature   | 🚀✅ No confilcts: fast-forward merge is possible. |
"#
        .trim_start();

        assert_eq!(text, expected);

        Ok(())
    }

    #[test]
    fn test_run_probranchinator_simple() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mock_analyzer = two_branches_analyzer();
        let mock_repo = this_repository();

        // // call run_probranchinator with mocks and buffer
        run_probranchinator(
            Args {
                output: crate::cli::OutputType::Simple,
                remote: "".to_string(),
                branches: vec![],
                pretty: crate::cli::BooleanCLI::False,
                recent: 0,
            },
            &mut buf,
            &mock_analyzer,
            &mock_repo,
        )?;

        // // check if output is text table with two analysis results
        let text = String::from_utf8(buf).unwrap();
        let expected = r#"
feature -> master : ✅✅ No changes: already up-to-date.
master -> feature : 🚀✅ No confilcts: fast-forward merge is possible.
"#
        .trim_start();

        assert_eq!(text, expected);

        Ok(())
    }

    #[test]
    fn test_run_probranchinator_json() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mock_analyzer = two_branches_analyzer();
        let mock_repo = this_repository();

        // // call run_probranchinator with mocks and buffer
        run_probranchinator(
            Args {
                output: crate::cli::OutputType::Json,
                remote: "".to_string(),
                branches: vec![],
                pretty: crate::cli::BooleanCLI::False,
                recent: 0,
            },
            &mut buf,
            &mock_analyzer,
            &mock_repo,
        )?;

        // // check if output is valid json with array of two elements
        let json = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let expected = json!([
            {
                "status": "UpToDate",
                "from_branch": "feature",
                "to_branch": "master"
            },
            {
                "status": "FastForward",
                "from_branch": "master",
                "to_branch": "feature"
            }
        ]);
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed, expected);

        Ok(())
    }

    fn this_repository() -> MockRepo {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_get_repo().returning(|_| {
            Ok((
                git2::Repository::open_from_env().unwrap(),
                "master".to_string().into(),
                false,
            ))
        });
        mock_repo
    }

    fn two_branches_analyzer() -> MockAnalyzer {
        let mut mock_analyzer = MockAnalyzer::new();
        mock_analyzer.expect_analyse().returning(|_, _, _| {
            Ok(vec![
                MergeAnalysisResult {
                    status: MergeAnalysisStatus::UpToDate,
                    from_branch: "feature".to_string(),
                    to_branch: "master".to_string(),
                },
                MergeAnalysisResult {
                    status: MergeAnalysisStatus::FastForward,
                    from_branch: "master".to_string(),
                    to_branch: "feature".to_string(),
                },
            ])
        });
        mock_analyzer
    }
}
