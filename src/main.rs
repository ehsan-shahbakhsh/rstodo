use std::{env, fs, io::{self, Read}};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{backend::{Backend, CrosstermBackend}, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Span, Spans, Text}, widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph}, Frame, Terminal};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(PartialEq, Eq)]
enum InputMode {
    Normal,
    Adding,
    Search,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
}


struct App {
    mode: InputMode,
    list_state: ListState,
    tasks: Vec<Task>,
    search_txt: String,
    new_task: String,
    default_style: Style,
    db_path: String,
}

impl App {
    fn new() -> App {
        App {
            mode: InputMode::Normal,
            list_state: ListState::default(),
            tasks: vec![],
            search_txt: String::new(),
            new_task: String::new(),
            default_style: Style::default().fg(Color::White).bg(Color::Black),
            db_path: env::var("TODO_DB").unwrap_or_else(|_| panic!("Please create a variable named TODO_DB in your environment variables and put your json file name in it.")),
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut file = fs::File::open(app.db_path.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let tasks: Vec<Task> = serde_json::from_str(&contents).unwrap();
    app.tasks = tasks;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    if let Err(e) = result {
        println!("{}", e.to_string());
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => {
                                let tasks = serde_json::to_string(&app.tasks).unwrap();
                                fs::write(app.db_path.to_string(), &tasks.as_bytes()).expect("Unable to write file");
                                return Ok(());
                            },
                            KeyCode::Char('n') => {
                                app.mode = InputMode::Adding;
                                app.list_state.select(None);
                            }
                            KeyCode::Char('s') => {
                                app.mode = InputMode::Search;
                                app.list_state.select(None);
                            }
                            KeyCode::Up => {
                                let selected = match app.list_state.selected() {
                                    Some(v) => {
                                        if v == 0 {
                                            Some(v)
                                        } else {
                                            Some(v - 1)
                                        }
                                    },
                                    None => {
                                        Some(0)
                                    }
                                };
                                app.list_state.select(selected);
                            }
                            KeyCode::Down => {
                                let selected = match app.list_state.selected() {
                                    Some(v) => {
                                        if v == app.tasks.len() - 1 {
                                            Some(v)
                                        } else {
                                            Some(v + 1)
                                        }
                                    },
                                    None => {
                                        Some(0)
                                    }
                                };
                                app.list_state.select(selected);
                            }
                            KeyCode::Delete => {
                                if let Some(index) = app.list_state.selected() {
                                    app.tasks.remove(index);
                                    if index > 0 {
                                        app.list_state.select(Some(index - 1));
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(index) = app.list_state.selected() {
                                    app.tasks[index].completed = !app.tasks[index].completed;
                                }
                            }
                            KeyCode::Esc => {
                                app.list_state.select(None);
                            }
                            _ => {}
                        }
                    }
                    InputMode::Adding => {
                        match key.code {
                            KeyCode::Esc => {
                                app.mode = InputMode::Normal;
                                app.new_task.clear();
                            }
                            KeyCode::Char(c) => {
                                app.new_task.push(c);
                            }
                            KeyCode::Backspace => {
                                app.new_task.pop();
                            }
                            KeyCode::Enter => {
                                app.tasks.push(Task { text: app.new_task.to_owned(), completed: false });
                                app.new_task.clear();
                            }
                            _ => {}
                        }
                    }
                    InputMode::Search => {
                        match key.code {
                            KeyCode::Esc => {
                                app.mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                app.search_txt.push(c);
                            }
                            KeyCode::Backspace => {
                                app.search_txt.pop();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ].as_ref())
        .split(f.size());

        let input_widget = Paragraph::new(app.search_txt.as_ref())
        .style(match app.mode {
            InputMode::Normal => app.default_style,
            InputMode::Adding => app.default_style,
            InputMode::Search => Style::default().fg(Color::Yellow).bg(Color::Black),
        })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Search")
                    .style(match app.mode {
                        InputMode::Normal => app.default_style,
                        InputMode::Adding => app.default_style,
                        InputMode::Search => Style::default().fg(Color::Yellow).bg(Color::Black),
                    })
            );
        f.render_widget(input_widget, chunks[0]);

    let tasks: Vec<ListItem> = app.tasks.iter().filter_map(|t| {
        let style = app.default_style;

        let task = if t.completed {
            if t.text.starts_with(&app.search_txt) {
                ListItem::new(Spans::from(vec![
                    Span::styled("✔  ", style),
                    Span::styled(t.text.replace(&t.text.replace(&app.search_txt, "").to_owned(), "").to_owned(), style.fg(Color::Yellow).add_modifier(Modifier::CROSSED_OUT)),
                    Span::styled(t.text.replace(&app.search_txt, "").to_owned(), style.add_modifier(Modifier::CROSSED_OUT)),
                ]))
            }
            else {
                ListItem::new(Spans::from(vec![
                    Span::styled("✔  ", style),
                    Span::styled(t.text.to_owned(), style.add_modifier(Modifier::CROSSED_OUT)),
                ]))
            }
        }
        else {
            if t.text.starts_with(&app.search_txt) {
                ListItem::new(Spans::from(vec![
                    Span::styled("   ", style),
                    Span::styled(t.text.replace(&t.text.replace(&app.search_txt, "").to_owned(), "").to_owned(), style.fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(t.text.replace(&app.search_txt, "").to_owned(), style),
                ]))
            }
            else {
                ListItem::new(Spans::from(vec![
                    Span::styled("   ", style),
                    Span::styled(t.text.to_owned(), style),
                ]))
            }
        };
        if !app.search_txt.is_empty() {
            if t.text.starts_with(&app.search_txt) {
                Some(task.style(style))
            }
            else {
                None
            }
        }
        else {
            Some(task.style(style))
        }
    }).collect();
    let tasks_widget = List::new(tasks)
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        )
        .highlight_symbol("->")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(tasks_widget, chunks[1], &mut app.list_state);

    let input_widget = Paragraph::new(app.new_task.as_ref())
    .style(match app.mode {
        InputMode::Normal => app.default_style,
        InputMode::Adding => Style::default().fg(Color::Yellow).bg(Color::Black),
        InputMode::Search => app.default_style,
    })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("New Task")
                .style(match app.mode {
                    InputMode::Normal => app.default_style,
                    InputMode::Adding => Style::default().fg(Color::Yellow).bg(Color::Black),
                    InputMode::Search => app.default_style,
                })
        );
    f.render_widget(input_widget, chunks[2]);

    let (message_widget, style) = match app.mode {
        InputMode::Normal => (
            vec![
                Span::styled("q", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" exit | ", app.default_style),
                Span::styled("n", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" new quest | ", app.default_style),
                Span::styled("Enter", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" check/uncheck quest | ", app.default_style),
                Span::styled("↑/↓", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" navigate list | ", app.default_style),
                Span::styled("Delete", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" delete quest", app.default_style),
            ],
            app.default_style.add_modifier(Modifier::RAPID_BLINK)
        ),
        InputMode::Adding => (
            vec![
                Span::styled("Esc", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" stop adding | ", app.default_style),
                Span::styled("Enter", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" save task", app.default_style),
            ],
            app.default_style
        ),
        InputMode::Search => (
            vec![
                Span::styled("Esc", app.default_style.add_modifier(Modifier::BOLD)),
                Span::styled(" stop searching", app.default_style),
            ],
            app.default_style
        ),
    };

    let mut help_text = Text::from(Spans::from(message_widget));
    help_text.patch_style(style);
    let help_widget = Paragraph::new(help_text).style(app.default_style);
    f.render_widget(help_widget, chunks[3]);

    if app.mode == InputMode::Adding {
        f.set_cursor(
            chunks[2].x + app.new_task.len() as u16 + 1,
            chunks[2].y + 1,
        );
    }
    if app.mode == InputMode::Search {
        f.set_cursor(
            chunks[0].x + app.search_txt.len() as u16 + 1,
            chunks[0].y + 1,
        );
    }
}

