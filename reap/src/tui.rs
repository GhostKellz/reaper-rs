use crate::core;
use crate::aur;
use crate::aur::SearchResult;
use crossterm::event::{self, Event, KeyCode};
use mlua::Lua;
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, ListState};
use std::sync::{Arc, Mutex};

struct LogPane {
    lines: Arc<Mutex<Vec<String>>>,
}

impl LogPane {
    fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn push(&self, line: &str) {
        let mut lines = self.lines.lock().unwrap();
        lines.push(line.to_string());
        if lines.len() > 1000 {
            lines.remove(0);
        }
    }
    fn get(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
    pub fn clear(&self) {
        let mut lines = self.lines.lock().unwrap();
        lines.clear();
    }
}

fn run_lua_hook(hook: &str, pkg: &str) {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/reaper/brew.lua");
    if let Ok(script) = std::fs::read_to_string(&config_path) {
        let lua = Lua::new();
        if lua.load(&script).exec().is_ok() {
            let globals = lua.globals();
            if let Ok(func) = globals.get::<_, mlua::Function>(hook) {
                let _ = func.call::<_, ()>(pkg);
            }
        }
    }
}

pub async fn launch_tui() {
    let mut terminal = setup_terminal();
    let mut query = String::new();
    let mut results = Vec::new();
    let mut selected = 0usize;
    let mut preview = String::new();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(7),
                ])
                .split(f.size());
            let search = Paragraph::new(query.as_str())
                .block(Block::default().borders(Borders::ALL).title("Search"));
            f.render_widget(search, chunks[0]);
            let items: Vec<ListItem> = results.iter().map(|r: &SearchResult| {
                ListItem::new(format!("{} {} - {}", r.name, r.version, r.description.as_str()))
            }).collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .highlight_symbol("→ ");
            let mut state = ListState::default();
            state.select(Some(selected));
            f.render_stateful_widget(list, chunks[1], &mut state);
            let preview_box = Paragraph::new(preview.as_str())
                .block(Block::default().borders(Borders::ALL).title("Preview"));
            f.render_widget(preview_box, chunks[2]);
        }).unwrap();
        if crossterm::event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('/') => {
                        query.clear();
                    }
                    KeyCode::Char(c) => {
                        query.push(c);
                        results = aur::search(&query).await.unwrap_or_default();
                        selected = 0;
                        preview.clear();
                    }
                    KeyCode::Up => {
                        if selected > 0 { selected -= 1; }
                    }
                    KeyCode::Down => {
                        if selected + 1 < results.len() { selected += 1; }
                    }
                    KeyCode::Enter => {
                        if let Some(pkg) = results.get(selected) {
                            core::handle_install(vec![pkg.name.clone()]);
                        }
                    }
                    KeyCode::Tab => {
                        if let Some(pkg) = results.get(selected) {
                            preview = aur::get_pkgbuild_preview(&pkg.name);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    restore_terminal();
}

fn setup_terminal() -> ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> {
    use crossterm::execute;
    use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
    use std::io::stdout;
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    ratatui::Terminal::new(backend).unwrap()
}

fn restore_terminal() {
    use crossterm::execute;
    use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
    use std::io::stdout;
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).unwrap();
}

