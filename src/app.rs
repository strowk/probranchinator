use crate::{
    cli::{Args, BooleanCLI, OutputType},
    result::MergeAnalysisResult,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fehler::throws;
use git2::Repository;
use indicatif::{ProgressFinish, ProgressStyle};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

struct App {
    state: TableState,
    items: Vec<MergeAnalysisResult>,
}

impl App {
    fn new(answer: Vec<MergeAnalysisResult>) -> App {
        App {
            state: TableState::default(),
            items: answer,
        }
    }
    pub fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub(crate) trait Analyzer {
    fn analyse(
        &self,
        repo: Repository,
        branches: Vec<String>,
        recent: usize,
    ) -> eyre::Result<Vec<MergeAnalysisResult>>;
}

#[cfg_attr(test, automock)]
pub(crate) trait Repo {
    fn get_repo(&self, remote: &str) -> eyre::Result<(Repository, std::path::PathBuf, bool)>;
}

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
            output_interactive(answer)?;
        }
    }

    Ok(())
}

#[throws(eyre::Error)]
fn output_interactive(answer: Vec<MergeAnalysisResult>) {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(answer);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        log::error!("{:?}", err)
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> eyre::Result<()> {
    let received_sigint = Arc::new(AtomicBool::new(false));
    let sigint_reading = received_sigint.clone();
    ctrlc::set_handler(move || {
        received_sigint.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        if sigint_reading.load(Ordering::Relaxed) {
            return Ok(());
        }

        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') => {
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }
                }
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let rows = app.items.iter().map(|item| {
        let cells = vec![
            Cell::from(item.status.to_string()),
            Cell::from(Spans::from(vec![
                Span::styled(
                    &item.from_branch,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(" -> ", Style::default()),
                Span::styled(
                    &item.to_branch,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ])),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });

    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Analysis Result", "Merging Branches"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Merge Analysis"),
        )
        .highlight_style(selected_style)
        .widths(&[Constraint::Length(60), Constraint::Percentage(50)]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}

#[cfg(test)]
mod tests {

    use crate::{
        app::MockAnalyzer,
        cli::Args,
        result::{MergeAnalysisResult, MergeAnalysisStatus},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tui::buffer::Buffer;

    use super::{run_probranchinator, MockRepo};
    #[test]
    fn test_app_next_previous() {
        use super::*;
        let mut app = App::new(vec![
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
        ]);
        assert_eq!(app.state.selected(), None);
        app.next();
        assert_eq!(app.state.selected(), Some(0));
        app.next();
        assert_eq!(app.state.selected(), Some(1));
        app.next();
        assert_eq!(app.state.selected(), Some(0));
        app.previous();
        assert_eq!(app.state.selected(), Some(1));
        app.previous();
        assert_eq!(app.state.selected(), Some(0));
    }

    #[test]
    fn test_app_next_previous_empty() {
        use super::*;
        let mut app = App::new(vec![]);
        assert_eq!(app.state.selected(), None);
        app.next();
        assert_eq!(app.state.selected(), None);
        app.previous();
        assert_eq!(app.state.selected(), None);
    }

    #[test]
    fn test_ui() {
        use super::*;
        use tui::backend::TestBackend;

        let mut app = App::new(vec![
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
        ]);

        let backend = TestBackend::new(90, 7);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| ui(f, &mut app))
            .expect("Failed to draw UI");

        let mut expected = Buffer::with_lines(vec![
            "                                                                                          ",
            " ┌Merge Analysis────────────────────────────────────────────────────────────────────────┐ ",
            " │Analysis Result                                              Merging Branches         │ ",
            " │✅✅ No changes: already up-to-date.                         feature -> master        │ " ,
            " │🚀✅ No confilcts: fast-forward merge is possible.           master -> feature        │ ",
            " └──────────────────────────────────────────────────────────────────────────────────────┘ ",
            "                                                                                          "
          ]);

        // set blue style to table header

        for x in 2..62 {
            expected
                .get_mut(x, 2)
                .set_style(Style::default().bg(Color::Blue).fg(Color::Red));
        }

        expected
            .get_mut(62, 2)
            .set_style(Style::default().bg(Color::Blue));

        for x in 63..88 {
            expected
                .get_mut(x, 2)
                .set_style(Style::default().bg(Color::Blue).fg(Color::Red));
        }

        // set bold style to branch names

        for x in 63..70 {
            expected
                .get_mut(x, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        for x in 74..80 {
            expected
                .get_mut(x, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        for x in 63..69 {
            expected
                .get_mut(x, 4)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        for x in 73..80 {
            expected
                .get_mut(x, 4)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        terminal.backend().assert_buffer(&expected);
    }

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
