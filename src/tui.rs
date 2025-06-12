use crate::aur;
use crate::aur::SearchResult;
use crate::config::ReapConfig;
use crate::core;
use crossterm::event::{self, Event, KeyCode};
use mlua::Lua;
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::sync::{Arc, Mutex};

pub struct LogPane {
    lines: Arc<Mutex<Vec<String>>>,
}

impl LogPane {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn push(&self, line: &str) {
        let mut lines = self.lines.lock().unwrap();
        lines.push(line.to_string());
        if lines.len() > 1000 {
            lines.remove(0);
        }
    }
    pub fn get(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
    pub fn clear(&self) {
        let mut lines = self.lines.lock().unwrap();
        lines.clear();
    }
}

impl Default for LogPane {
    fn default() -> Self {
        Self::new()
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
    let log_pane = Arc::new(LogPane::new());
    let mut log_scroll = 0usize;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(7),
                    Constraint::Length(2),
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
            // LogPane display
            let log_lines = log_pane.get();
            let log_view: Vec<ListItem> = log_lines.iter().skip(log_scroll).map(|l| ListItem::new(l.clone())).collect();
            let log_widget = List::new(log_view)
                .block(Block::default().borders(Borders::ALL).title("Log (C=clear, ↑↓=scroll)"));
            f.render_widget(log_widget, chunks[3]);
            // Key mappings
            let help = Paragraph::new("Q: quit  /: search  ↑↓: select/scroll  Enter: install  Tab: preview  C: clear log  I: parallel install  U: parallel upgrade")
                .block(Block::default());
            f.render_widget(help, Layout::default().direction(Direction::Horizontal).constraints([Constraint::Min(1)]).split(f.size())[0]);
        }).unwrap();
        if crossterm::event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('/') => {
                        query.clear();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        log_pane.clear();
                        log_scroll = 0;
                    }
                    KeyCode::Char(c) => match c {
                        'i' => {
                            // Parallel install selected packages
                            let pkgs: Vec<String> =
                                results.iter().map(|r| r.name.clone()).collect();
                            tokio::spawn(async move {
                                core::parallel_install(
                                    &pkgs.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                                )
                                .await;
                            });
                        }
                        'u' => {
                            // Parallel upgrade all packages
                            let pkgs: Vec<String> =
                                results.iter().map(|r| r.name.clone()).collect();
                            let _config = Arc::new(ReapConfig::load());
                            let _log = log_pane.clone();
                            tokio::spawn(async move {
                                core::parallel_upgrade(pkgs, _config, Some(_log)).await;
                            });
                        }
                        _ => {
                            query.push(c);
                            results = aur::search(&query).await.unwrap_or_default();
                            selected = 0;
                            preview.clear();
                            log_pane.push(&format!(
                                "[reap] Searched for '{}', {} results",
                                query,
                                results.len()
                            ));
                        }
                    },
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                        log_scroll = log_scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected + 1 < results.len() {
                            selected += 1;
                        }
                        if log_scroll + 1 < log_pane.get().len() {
                            log_scroll += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(pkg) = results.get(selected) {
                            let deps = aur::get_deps(&pkg.name);
                            log_pane
                                .push(&format!("[reap] Dependencies for {}: {:?}", pkg.name, deps));
                            run_lua_hook("pre_install", &pkg.name);
                            core::handle_install(vec![pkg.name.clone()]);
                            log_pane.push(&format!("[reap] Installed {}", pkg.name));
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

// pub fn print_flatpak_sandbox_info(pkg: &str) { /* ... */ }
// pub fn show_gpg_key_info(keyid: &str) { /* ... */ }
