use crate::aur;
use crate::aur::SearchResult;
use crate::core;
use crate::core::get_installed_packages;
use crossterm::event::{self, Event, KeyCode};
use mlua::Lua;
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchSource {
    Aur,
    Flatpak,
    Pacman,
    ChaoticAur,
    GhostctlAur,
}

impl SearchSource {
    fn label(&self) -> &'static str {
        match self {
            SearchSource::Aur => "AUR",
            SearchSource::Flatpak => "Flatpak",
            SearchSource::Pacman => "Pacman",
            SearchSource::ChaoticAur => "ChaoticAUR",
            SearchSource::GhostctlAur => "GhostctlAUR",
        }
    }
    fn all() -> &'static [SearchSource] {
        &[
            Self::Aur,
            Self::Flatpak,
            Self::Pacman,
            Self::ChaoticAur,
            Self::GhostctlAur,
        ]
    }
}

struct SearchTab {
    query: String,
    source: SearchSource,
    results: Vec<SearchResult>,
    selected: usize,
    sources: Vec<SearchSource>,
}

impl SearchTab {
    fn new() -> Self {
        Self {
            query: String::new(),
            source: SearchSource::Aur,
            results: Vec::new(),
            selected: 0,
            sources: SearchSource::all().to_vec(),
        }
    }
    async fn do_search(&mut self) {
        match self.source {
            SearchSource::Aur => {
                self.results = aur::search(&self.query).await.unwrap_or_default();
            }
            SearchSource::Flatpak => {
                self.results = crate::flatpak::search(&self.query);
            }
            SearchSource::Pacman | SearchSource::ChaoticAur | SearchSource::GhostctlAur => {
                let repo = match self.source {
                    SearchSource::Pacman => None,
                    SearchSource::ChaoticAur => Some("chaotic-aur"),
                    SearchSource::GhostctlAur => Some("ghostctl-aur"),
                    _ => None,
                };
                self.results = search_pacman_like(&self.query, repo);
            }
        }
        self.selected = 0;
    }
}

fn search_pacman_like(query: &str, repo: Option<&str>) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut cmd = std::process::Command::new("pacman");
    cmd.arg("-Ss").arg(query);
    if let Some(repo) = repo {
        cmd.arg("| grep").arg(repo);
    }
    if let Ok(out) = cmd.output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let mut parts = line.split_whitespace();
            let pkg = parts.nth(0).unwrap_or("");
            let version = parts.nth(0).unwrap_or("");
            let desc = parts.collect::<Vec<_>>().join(" ");
            if !pkg.is_empty() {
                results.push(SearchResult {
                    name: pkg.to_string(),
                    version: version.to_string(),
                    description: desc,
                    source: crate::core::Source::Pacman,
                });
            }
        }
    }
    results
}

struct InstallQueue {
    tasks: Arc<Mutex<Vec<core::InstallTask>>>,
}

impl InstallQueue {
    fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn enqueue(&self, task: core::InstallTask) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }
    fn pop(&self) -> Option<core::InstallTask> {
        let mut tasks = self.tasks.lock().unwrap();
        if !tasks.is_empty() {
            Some(tasks.remove(0))
        } else {
            None
        }
    }
    fn is_empty(&self) -> bool {
        self.tasks.lock().unwrap().is_empty()
    }
}

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

struct DiffViewer {
    lines: Vec<(char, String)>, // ('-', '+', ' ') and line
    scroll: usize,
}

impl DiffViewer {
    fn new(old: &str, new: &str) -> Self {
        let mut lines = Vec::new();
        for d in diff::lines(old, new) {
            match d {
                diff::Result::Left(l) => lines.push(('-', l.to_string())),
                diff::Result::Right(r) => lines.push(('+', r.to_string())),
                diff::Result::Both(l, _) => lines.push((' ', l.to_string())),
            }
        }
        Self { lines, scroll: 0 }
    }
    fn render(
        &self,
        f: &mut Frame<'_>,
        area: ratatui::layout::Rect,
    ) {
        let items: Vec<Line> = self
            .lines
            .iter()
            .map(|(ty, line)| {
                let style = match ty {
                    '-' => Style::default().fg(Color::Red),
                    '+' => Style::default().fg(Color::Green),
                    _ => Style::default(),
                };
                Line::from(vec![Span::styled(format!("{} {}", ty, line), style)])
            })
            .collect();
        let block = Block::default()
            .borders(Borders::ALL)
            .title("PKGBUILD Diff");
        let para = Paragraph::new(items)
            .block(block)
            .scroll((self.scroll as u16, 0));
        f.render_widget(Clear, area);
        f.render_widget(para, area);
    }
}

#[allow(dead_code)]
pub async fn launch_tui() {
    let mut terminal = setup_terminal();
    let mut search_tab = SearchTab::new();
    let mut tab_idx = 0;
    let tab_titles = vec!["Search", "Queue", "Log"];
    let log_pane = Arc::new(LogPane::new());
    let mut log_scroll = 0usize;
    let install_queue = InstallQueue::new();
    let installed = core::get_installed_packages();
    let mut diff_viewer: Option<DiffViewer> = None;

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(7),
                        Constraint::Length(2),
                    ])
                    .split(size);
                let tabs = Tabs::new(tab_titles.iter().cloned().map(String::from).collect::<Vec<_>>())
                    .block(Block::default().borders(Borders::ALL).title("Tabs"))
                    .select(tab_idx)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                f.render_widget(tabs, chunks[0]);
                match tab_idx {
                    0 => {
                        let source_dropdown =
                            Paragraph::new(format!("Source: {}", search_tab.source.label()))
                                .block(Block::default().borders(Borders::ALL).title("Source"));
                        f.render_widget(source_dropdown, chunks[1]);
                        let search = Paragraph::new(search_tab.query.as_str())
                            .block(Block::default().borders(Borders::ALL).title("Search Query"));
                        f.render_widget(search, chunks[2]);
                        let items: Vec<ListItem> = search_tab
                            .results
                            .iter()
                            .map(|r| {
                                let badge = match &r.source {
                                    crate::core::Source::Aur => "[AUR]",
                                    crate::core::Source::Flatpak => "[Flatpak]",
                                    crate::core::Source::Pacman => "[Pacman]",
                                    crate::core::Source::BinaryRepo(repo) => repo.as_str(),
                                    crate::core::Source::ChaoticAUR => "chaotic",
                                    crate::core::Source::GhostctlAUR => "ghostctl",
                                    crate::core::Source::Custom(name) => name,
                                };
                                ListItem::new(format!(
                                    "{} {} {}\n{}",
                                    badge, r.name, r.version, r.description
                                ))
                            })
                            .collect();
                        let mut state = ListState::default();
                        state.select(Some(search_tab.selected));
                        let list = List::new(items)
                            .block(Block::default().borders(Borders::ALL).title("Results"))
                            .highlight_symbol("→ ");
                        f.render_stateful_widget(list, chunks[3], &mut state);
                    }
                    2 => {
                        let log_lines = log_pane.get();
                        let log_view: Vec<ListItem> = log_lines
                            .iter()
                            .skip(log_scroll)
                            .map(|l| ListItem::new(l.clone()))
                            .collect();
                        let log_widget = List::new(log_view).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Log (C=clear, ↑↓=scroll)"),
                        );
                        f.render_widget(log_widget, chunks[3]);
                    }
                    _ => {}
                }
                if let Some(diff) = &diff_viewer {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.size())[0];
                    diff.render(f, area);
                }
            })
            .unwrap();

        if crossterm::event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('/') => {
                        search_tab.query.clear();
                        search_tab.do_search().await;
                    }
                    KeyCode::Char('d') => {
                        if let Some(pkg) = search_tab.results.get(search_tab.selected) {
                            let pkgb = aur::get_pkgbuild_preview(&pkg.name);
                            let installed_pkgb = aur::get_pkgbuild_preview(&pkg.name);
                            diff_viewer = Some(DiffViewer::new(&installed_pkgb, &pkgb));
                            log_pane.push(&format!("[reap][diff] Shown for {}", pkg.name));
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        log_pane.clear();
                        log_scroll = 0;
                    }
                    KeyCode::Char('\t') => {
                        tab_idx = (tab_idx + 1) % tab_titles.len();
                    }
                    KeyCode::Char(c) => {
                        search_tab.query.push(c);
                        search_tab.do_search().await;
                    }
                    KeyCode::Up => {
                        search_tab.selected = search_tab.selected.saturating_sub(1);
                        log_scroll = log_scroll.saturating_sub(1);
                        if let Some(diff) = &mut diff_viewer {
                            diff.scroll = diff.scroll.saturating_sub(1);
                        }
                    }
                    KeyCode::Down => {
                        if search_tab.selected + 1 < search_tab.results.len() {
                            search_tab.selected += 1;
                        }
                        if log_scroll + 1 < log_pane.get().len() {
                            log_scroll += 1;
                        }
                        if let Some(diff) = &mut diff_viewer {
                            diff.scroll += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(pkg) = search_tab.results.get(search_tab.selected) {
                            if installed.contains_key(&pkg.name) {
                                log_pane.push(&format!("[reap] {} is already installed", pkg.name));
                            } else {
                                let task = core::InstallTask {
                                    pkg: pkg.name.clone(),
                                    edit: false,
                                    confirm: true,
                                    dry_run: false,
                                    source: pkg.source.clone(),
                                    repo_name: None,
                                };
                                install_queue.enqueue(task);
                                log_pane
                                    .push(&format!("[reap] Added {} to install queue", pkg.name));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    restore_terminal();
}

pub async fn run_ui() {
    use crate::utils;
    use crossterm::{event, terminal};
    use std::io::{Write, stdout};
    use std::time::Duration;
    let pkgs = get_installed_packages();
    let mut pinned = Vec::new();
    let mut selected = 0;
    let pkg_names: Vec<_> = pkgs.keys().cloned().collect();
    let _ = terminal::enable_raw_mode();
    let mut stdout = stdout();
    loop {
        println!("[reap TUI] Installed Packages (use ↑/↓, p=pin, q=quit):");
        for (i, pkg) in pkg_names.iter().enumerate() {
            if i == selected {
                print!("> ");
            } else {
                print!("  ");
            }
            let pin_mark = if pinned.contains(pkg) { "[*]" } else { "   " };
            println!("{} {}", pin_mark, pkg);
        }
        stdout.flush().unwrap();
        if event::poll(Duration::from_millis(200)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                match key.code {
                    event::KeyCode::Char('q') => break,
                    event::KeyCode::Up => selected = selected.saturating_sub(1),
                    event::KeyCode::Down => {
                        if selected + 1 < pkg_names.len() {
                            selected += 1;
                        }
                    }
                    event::KeyCode::Char('p') => {
                        let pkg = &pkg_names[selected];
                        if !pinned.contains(pkg) {
                            let _ = utils::pin_package(pkg);
                            pinned.push(pkg.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let _ = terminal::disable_raw_mode();
    println!("[reap TUI] Exited.");
}

pub fn setup_terminal() -> ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> {
    use crossterm::execute;
    use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
    use std::io::stdout;
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    ratatui::Terminal::new(backend).unwrap()
}

pub fn restore_terminal() {
    use crossterm::execute;
    use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
    use std::io::stdout;
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).unwrap();
}
