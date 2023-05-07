use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::{ProgressFinish, ProgressStyle};
use std::{
    error::Error,
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

use crate::{
    analysis::{self, MergeAnalysisResult},
    repo::get_repo,
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

pub(crate) fn run_app(
    remote: String,
    branches: Vec<String>,
    recent: usize,
) -> Result<(), Box<dyn Error>> {
    let spinner = indicatif::ProgressBar::new_spinner()
        .with_prefix("[1/2]")
        .with_message("Retrieving repository...")
        .with_finish(ProgressFinish::AndLeave)
        .with_style(ProgressStyle::with_template(
            "{prefix:.cyan/blue} {spinner} {msg}",
        )?);
    spinner.enable_steady_tick(Duration::from_millis(100));
    let (repo, tmp_path, have_cached_repo) = get_repo(&remote)?;

    spinner.set_style(ProgressStyle::with_template(
        "Retrieved repository in {elapsed}",
    )?);

    spinner.finish();

    eprintln!(
        "Using repository cache at {:?} (cached: {})",
        tmp_path, have_cached_repo
    );

    let answer = analysis::analyse(repo, branches, recent)?;
    answer.iter().for_each(|x| {
        eprintln!("{}", x);
    });

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(answer);
    let res = _run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    Ok(())
}

fn _run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> eyre::Result<()> {
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
