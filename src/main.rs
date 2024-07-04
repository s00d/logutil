use chrono::Local;
use ratatui::{
    backend::{CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Terminal,
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use ratatui::layout::Margin;
use ratatui::prelude::Line;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use structopt::StructOpt;
use tokio::time::sleep;

#[derive(StructOpt)]
#[structopt(name = "Log Analyzer", about = "A tool to analyze Nginx access logs.")]
struct Cli {
    /// Path to the log file
    #[structopt(short, long, default_value = "access.log")]
    file: String,

    /// Number of lines to read from the end of the file (0 to start from the end, -1 to read the entire file)
    #[structopt(short = "c", long, default_value = "0")]
    count: isize,

    /// Regular expression to parse the log entries or path to a file containing the regex
    #[structopt(
        short,
        long,
        default_value = r#"^(\S+) - ".+" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*""#
    )]
    regex: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,

    /// Disable clearing of outdated entries
    #[structopt(long)]
    no_clear: bool,

    /// Filter results by IP address
    #[structopt(long)]
    filter_ip: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Обрабатываем аргументы командной строки
    let args = Cli::from_args();

    // Проверяем, был ли запрошен флаг помощи, чтобы избежать искажений в консоли
    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(());
    }

    if let Err(e) = env::set_current_dir(env::current_dir().expect("Failed to get current directory")) {
        eprintln!("Failed to set current directory: {:?}", e);
    }

    // Переходим в сырой режим и альтернативный экран
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let file_path = args.file.clone();
    let count = args.count;
    let regex_pattern = if Path::new(&args.regex).exists() {
        fs::read_to_string(&args.regex).expect("Could not read regex file")
    } else {
        args.regex.clone()
    };
    let top_n = args.top;
    let no_clear = args.no_clear;
    let filter_ip = args.filter_ip.clone();

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let log_data_clone = Arc::clone(&log_data);

    tokio::spawn(async move {
        let _ = tail_file(&file_path, count, &regex_pattern, &log_data_clone, no_clear).await;
        loop {
            if let Err(e) = tail_file(&file_path, 0, &regex_pattern, &log_data_clone, no_clear).await {
                eprintln!("Error reading file: {:?}", e);
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    let mut app = App::new(log_data, top_n, filter_ip);

    loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.quit();
                        break;
                    }
                    KeyCode::Char('t') => {
                        app.toggle_tab();
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.quit();
                        break;
                    }
                    KeyCode::Up => {
                        app.on_up();
                    }
                    KeyCode::Down => {
                        app.on_down();
                    }
                    KeyCode::Left => {
                        app.on_left();
                    }
                    KeyCode::Right => {
                        app.on_right();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}


struct LogEntry {
    count: usize,
    last_update: SystemTime,
    last_requests: Vec<String>,
}

struct LogData {
    by_ip: HashMap<String, LogEntry>,
    by_url: HashMap<String, LogEntry>,
    total_requests: usize,
}

impl LogData {
    fn new() -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
        }
    }

    fn add_entry(&mut self, ip: String, url: String, log_line: String, no_clear: bool) {
        let now = SystemTime::now();

        {
            let entry = self.by_ip.entry(ip.clone()).or_insert(LogEntry {
                count: 0,
                last_update: now,
                last_requests: Vec::new(),
            });
            entry.count += 1;
            entry.last_update = now;
            entry.last_requests.push(log_line.clone());
            if entry.last_requests.len() > 10 {
                entry.last_requests.remove(0);
            }
        }

        {
            let entry = self.by_url.entry(url).or_insert(LogEntry {
                count: 0,
                last_update: now,
                last_requests: Vec::new(),
            });
            entry.count += 1;
            entry.last_update = now;
            entry.last_requests.push(log_line.clone());
            if entry.last_requests.len() > 10 {
                entry.last_requests.remove(0);
            }
        }

        self.total_requests += 1;

        if self.by_ip.len() > 10000 && !no_clear {
            self.clear_outdated_entries();
        }
    }

    fn clear_outdated_entries(&mut self) {
        let threshold = SystemTime::now() - Duration::from_secs(1200); // 20 минут
        self.by_ip.retain(|_, entry| entry.last_update >= threshold);
        self.by_url.retain(|_, entry| entry.last_update >= threshold);
    }

    fn get_top_n(&self, n: usize) -> (Vec<(String, usize)>, Vec<(String, usize)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v.count))
                .collect(),
            top_url
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v.count))
                .collect(),
        )
    }

    fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }

    fn get_last_requests(&self, ip: &str) -> Vec<String> {
        self.by_ip
            .get(ip)
            .map_or(Vec::new(), |entry| entry.last_requests.clone())
    }
}

async fn tail_file(
    file_path: &str,
    count: isize,
    regex_pattern: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    if count > 0 {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let content = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > count as usize {
            lines.len() - count as usize
        } else {
            0
        };

        for line in &lines[start..] {
            process_line(line, &regex_pattern, log_data, no_clear).await?;
        }
    } else if count == -1 {
        reader.seek(SeekFrom::Start(0))?;
        let re = Regex::new(regex_pattern).unwrap();

        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line)?;

            if len == 0 {
                break;
            }

            if let Some(caps) = re.captures(&line) {
                let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let url = caps.get(2).map_or("", |m| m.as_str()).to_string();

                let mut log_data = log_data.lock().unwrap();
                log_data.add_entry(ip, url, line.clone(), no_clear);
            } else {
                println!("No match for line: {}", line);
            }
        }
    } else {
        reader.seek(SeekFrom::End(0))?;
    }

    let re = Regex::new(regex_pattern).unwrap();

    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;

        if len == 0 {
            break;
        }

        if let Some(caps) = re.captures(&line) {
            let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let url = caps.get(2).map_or("", |m| m.as_str()).to_string();

            let mut log_data = log_data.lock().unwrap();
            log_data.add_entry(ip, url, line.clone(), no_clear);
        } else {
            println!("No match for line: {}", line);
        }
    }

    Ok(())
}

async fn process_line(
    line: &str,
    regex_pattern: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
) -> std::io::Result<()> {
    let re = Regex::new(regex_pattern).unwrap();
    if let Some(caps) = re.captures(line) {
        let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let url = caps.get(2).map_or("", |m| m.as_str()).to_string();

        let mut log_data = log_data.lock().unwrap();
        log_data.add_entry(ip, url, line.to_string(), no_clear);
    } else {
        println!("No match for line: {}", line);
    }

    Ok(())
}

struct App {
    log_data: Arc<Mutex<LogData>>,
    should_quit: bool,
    top_n: usize,
    filter_ip: Option<String>,
    current_tab: usize,
    vertical_scroll: usize,
    horizontal_scroll: usize,
}

impl App {
    fn new(
        log_data: Arc<Mutex<LogData>>,
        top_n: usize,
        filter_ip: Option<String>,
    ) -> Self {
        Self {
            log_data,
            should_quit: false,
            top_n,
            filter_ip,
            current_tab: 0,
            vertical_scroll: 0,
            horizontal_scroll: 0,
        }
    }

    fn draw(&mut self, frame: &mut ratatui::terminal::Frame) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);

        let titles: Vec<String> = ["Overview", "Last Requests", "Detailed Requests"]
            .iter()
            .cloned()
            .map(|t| t.into())
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.current_tab)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|");
        frame.render_widget(tabs, chunks[0]);

        match self.current_tab {
            0 => self.draw_overview(frame, chunks[1]),
            1 => self.draw_last_requests(frame, chunks[1]),
            2 => self.draw_detailed_requests(frame, chunks[1]),
            _ => {}
        }
    }

    fn draw_overview(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);
        let (unique_ips, unique_urls) = log_data.get_unique_counts();

        let now = Local::now();
        let summary = format!("Total requests: {} | Unique IPs: {} | Unique URLs: {} | Last update: {}", log_data.total_requests, unique_ips, unique_urls, now.format("%Y-%m-%d %H:%M:%S"));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        frame.render_widget(
            Paragraph::new(summary)
                .block(Block::default().borders(Borders::ALL).title("Summary")),
            chunks[0],
        );

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        let ip_rows: Vec<Row> = top_ips
            .into_iter()
            .map(|(ip, count)| Row::new(vec![ip, count.to_string()]))
            .collect();
        let widths = [Constraint::Length(70), Constraint::Length(30)];
        let ip_table = Table::new(ip_rows, widths)
            .block(Block::default().borders(Borders::ALL).title("Top IPs"))
            .header(Row::new(vec!["IP", "Requests"]).style(Style::default().fg(Color::Yellow)));
        frame.render_widget(ip_table, chunks[0]);

        let url_rows: Vec<Row> = top_urls
            .into_iter()
            .map(|(url, count)| Row::new(vec![url, count.to_string()]))
            .collect();
        let widths = [Constraint::Length(70), Constraint::Length(30)];
        let url_table = Table::new(url_rows, widths)
            .block(Block::default().borders(Borders::ALL).title("Top URLs"))
            .header(Row::new(vec!["URL", "Requests"]).style(Style::default().fg(Color::Yellow)));
        frame.render_widget(url_table, chunks[1]);
    }

    fn draw_last_requests(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();

        let mut items: Vec<Line> = vec![];
        if let Some(filter_ip) = &self.filter_ip {
            if let Some(entry) = log_data.by_ip.get(filter_ip) {
                for request in &entry.last_requests {
                    items.push(Line::from(request.clone()));
                }
            }
        } else {
            for (ip, entry) in &log_data.by_ip {
                for request in &entry.last_requests {
                    items.push(Line::from(format!("{}: {}", ip, request)));
                }
            }
        }

        let paragraph = Paragraph::new(items.clone())
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16))
            .block(Block::default().borders(Borders::RIGHT));

        let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut vertical_scrollbar_state = ScrollbarState::new(items.len()).position(self.vertical_scroll);

        let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(Some("←"))
            .end_symbol(Some("→"));
        let max_line_length = items.iter().map(|line| line.width()).max().unwrap_or(0);
        let mut horizontal_scrollbar_state = ScrollbarState::new(max_line_length).position(self.horizontal_scroll);

        let paragraph_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });

        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(vertical_scrollbar, paragraph_area, &mut vertical_scrollbar_state);
        frame.render_stateful_widget(horizontal_scrollbar, paragraph_area, &mut horizontal_scrollbar_state);
    }


    fn draw_detailed_requests(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();

        let top_ips = log_data.get_top_n(self.top_n).0;
        let mut items: Vec<Line> = vec![];

        for (ip, _) in &top_ips {
            let last_requests = log_data.get_last_requests(ip);
            if !last_requests.is_empty() {
                items.push(Line::from(ip.clone()).style(Style::default().fg(Color::Yellow)));
                for request in last_requests {
                    items.push(Line::from(request));
                }
            }
        }

        let paragraph = Paragraph::new(items.clone())
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16))
            .block(Block::default().borders(Borders::RIGHT));

        let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut vertical_scrollbar_state = ScrollbarState::new(items.len()).position(self.vertical_scroll);

        let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(Some("←"))
            .end_symbol(Some("→"));
        let max_line_length = items.iter().map(|line| line.width()).max().unwrap_or(0);
        let mut horizontal_scrollbar_state = ScrollbarState::new(max_line_length).position(self.horizontal_scroll);

        let paragraph_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });

        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(vertical_scrollbar, paragraph_area, &mut vertical_scrollbar_state);
        frame.render_stateful_widget(horizontal_scrollbar, paragraph_area, &mut horizontal_scrollbar_state);
    }

    // fn draw_request_graph(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
    //     let sparkline = Sparkline::default()
    //         .block(Block::default().borders(Borders::ALL).title("Requests per Second"))
    //         .data(&self.request_counts)
    //         .style(Style::default().fg(Color::Yellow));
    //
    //     frame.render_widget(sparkline, area);
    // }

    fn on_up(&mut self) {
        if self.vertical_scroll > 0 {
            self.vertical_scroll -= 2;
        }
    }

    fn on_down(&mut self) {
        self.vertical_scroll += 2;
    }

    fn on_left(&mut self) {
        if self.horizontal_scroll > 0 {
            self.horizontal_scroll -= 5;
        }
    }

    fn on_right(&mut self) {
        self.horizontal_scroll += 10;
    }

    fn toggle_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}
