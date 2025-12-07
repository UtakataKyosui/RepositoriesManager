use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Topic {
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Repository {
    name: String,
    description: Option<String>,
    #[serde(rename = "repositoryTopics")]
    repository_topics: Option<Vec<Topic>>,
}

impl Repository {
    fn topics_as_string(&self) -> String {
        if let Some(topics) = &self.repository_topics {
            topics.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", ")
        } else {
            String::new()
        }
    }
}

#[derive(Debug, PartialEq)]
enum EditField {
    Description,
    Topics,
}

#[derive(Debug)]
enum AppMode {
    Normal,
    Editing {
        field: EditField,
        content: String,
    },
}

struct App {
    repositories: Vec<Repository>,
    table_state: TableState,
    mode: AppMode,
    status_message: String,
}

impl App {
    fn new(repositories: Vec<Repository>) -> Self {
        let mut table_state = TableState::default();
        if !repositories.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            repositories,
            table_state,
            mode: AppMode::Normal,
            status_message: String::from("q: Quit | ↑↓: Navigate | d: Edit Description | t: Edit Topics | Enter: Save"),
        }
    }

    fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.repositories.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.repositories.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn start_editing(&mut self, field: EditField) {
        if let Some(selected) = self.table_state.selected() {
            let repo = &self.repositories[selected];
            let content = match field {
                EditField::Description => repo.description.clone().unwrap_or_default(),
                EditField::Topics => repo.topics_as_string(),
            };
            self.mode = AppMode::Editing { field, content };
            self.status_message = "Editing... | Enter: Save | Esc: Cancel".to_string();
        }
    }

    fn cancel_editing(&mut self) {
        self.mode = AppMode::Normal;
        self.status_message = String::from("q: Quit | ↑↓: Navigate | d: Edit Description | t: Edit Topics | Enter: Save");
    }

    async fn save_edit(&mut self) -> Result<()> {
        if let AppMode::Editing { field, content } = &self.mode {
            if let Some(selected) = self.table_state.selected() {
                let repo_name = self.repositories[selected].name.clone();

                match field {
                    EditField::Description => {
                        self.status_message = format!("Updating description for {}...", repo_name);

                        let output = Command::new("gh")
                            .args(["repo", "edit", &repo_name, "--description", content])
                            .output()?;

                        if output.status.success() {
                            self.repositories[selected].description = Some(content.clone());
                            self.status_message = format!("✓ Description updated for {}", repo_name);
                        } else {
                            let error = String::from_utf8_lossy(&output.stderr);
                            self.status_message = format!("✗ Error: {}", error);
                        }
                    }
                    EditField::Topics => {
                        self.status_message = format!("Updating topics for {}...", repo_name);

                        let topics: Vec<&str> = content
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let mut cmd = Command::new("gh");
                        cmd.args(["repo", "edit", &repo_name]);

                        if topics.is_empty() {
                            cmd.arg("--remove-topic").arg("*");
                        } else {
                            for topic in &topics {
                                cmd.arg("--add-topic").arg(topic);
                            }
                        }

                        let output = cmd.output()?;

                        if output.status.success() {
                            self.repositories[selected].repository_topics = Some(
                                topics.iter().map(|t| Topic { name: t.to_string() }).collect()
                            );
                            self.status_message = format!("✓ Topics updated for {}", repo_name);
                        } else {
                            let error = String::from_utf8_lossy(&output.stderr);
                            self.status_message = format!("✗ Error: {}", error);
                        }
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
        Ok(())
    }

    fn handle_edit_input(&mut self, key_code: KeyCode) {
        if let AppMode::Editing { content, .. } = &mut self.mode {
            match key_code {
                KeyCode::Char(c) => {
                    content.push(c);
                }
                KeyCode::Backspace => {
                    content.pop();
                }
                _ => {}
            }
        }
    }
}

fn fetch_repositories() -> Result<Vec<Repository>> {
    let output = Command::new("gh")
        .args(["repo", "list", "--json", "name,description,repositoryTopics", "--limit", "1000"])
        .output()
        .context("Failed to execute gh command. Is gh CLI installed and authenticated?")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh command failed: {}", error);
    }

    let repos: Vec<Repository> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse gh output")?;

    Ok(repos)
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    render_table(f, app, chunks[0]);
    render_status(f, app, chunks[1]);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_cells = ["Name", "Description", "Topics"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.repositories.iter().map(|repo| {
        let cells = vec![
            Cell::from(repo.name.clone()),
            Cell::from(repo.description.clone().unwrap_or_default()),
            Cell::from(repo.topics_as_string()),
        ];
        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Repositories"))
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match &app.mode {
        AppMode::Normal => vec![Line::from(app.status_message.clone())],
        AppMode::Editing { field, content } => {
            let field_name = match field {
                EditField::Description => "Description",
                EditField::Topics => "Topics (comma-separated)",
            };
            vec![
                Line::from(vec![
                    Span::styled(format!("Editing {}: ", field_name), Style::default().fg(Color::Cyan)),
                    Span::raw(content),
                ]),
                Line::from(app.status_message.clone()),
            ]
        }
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(status, area);
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Fetching repositories...");
    let repositories = fetch_repositories()?;

    if repositories.is_empty() {
        println!("No repositories found.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repositories);
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match &app.mode {
                    AppMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(())
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('d') => app.start_editing(EditField::Description),
                        KeyCode::Char('t') => app.start_editing(EditField::Topics),
                        _ => {}
                    },
                    AppMode::Editing { .. } => match key.code {
                        KeyCode::Enter => {
                            app.save_edit().await?;
                        }
                        KeyCode::Esc => app.cancel_editing(),
                        code => app.handle_edit_input(code),
                    },
                }
            }
        }
    }
}
