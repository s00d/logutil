use chrono::{DateTime, Local, Offset, TimeZone, Utc};
use ratatui::{backend::{CrosstermBackend}, crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
}, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style, Modifier}, terminal::Terminal, widgets::{Block, Borders, Paragraph, Row, Table, Tabs, List, ListItem, ListState}};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::time::{Duration, SystemTime};
use crossterm::event::KeyModifiers;
use ratatui::widgets::{RenderDirection, Sparkline};
use structopt::StructOpt;
use textwrap::wrap;
use tokio::time::sleep;
const NORMAL_ROW_BG: Color = Color::Rgb(18, 18, 20);
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(0, 31, 63)).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = Color::Rgb(158, 158, 158);
#[derive(StructOpt)]
#[structopt(
    name = "log Util",
    author = "s00d",
    about = "A tool to analyze Nginx access logs.\n\n\
    GitHub: https://github.com/s00d/logutil"
)]
struct Cli {
    /// Path to the log file
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// Number of lines to read from the end of the file (0 to start from the end, -1 to read the entire file)
    #[structopt(short = "c", long, default_value = "0")]
    count: isize,

    /// Regular expression to parse the log entries or path to a file containing the regex
    #[structopt(
        short,
        long,
        default_value = r#"^(\S+) - ".+" \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)?""#
    )]
    regex: String,

    /// Date format to parse the log entries
    #[structopt(
        short = "d",
        long,
        default_value = "%d/%b/%Y:%H:%M:%S %z"
    )]
    date_format: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,

    /// Disable clearing of outdated entries
    #[structopt(long)]
    no_clear: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(());
    }

    if let Err(e) = env::set_current_dir(env::current_dir().expect("Failed to get current directory")) {
        eprintln!("Failed to set current directory: {:?}", e);
    }

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let file_path = args.file.clone();
    let count = args.count;
    let regex_pattern = if Path::new(&args.regex).exists() {
        fs::read_to_string(&args.regex).expect("Could not read regex file")
    } else {
        args.regex.clone()
    };
    let date_format = args.date_format.clone();
    let top_n = args.top;
    let no_clear = args.no_clear;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let log_data_clone = Arc::clone(&log_data);

    let (tx, rx) = mpsc::channel();

    let handle = tokio::spawn(async move {
        let mut last_processed_line = None;
        match tail_file(&file_path, count, &regex_pattern, &date_format, &log_data_clone, no_clear, None).await {
            Ok(last_line) => {
                last_processed_line = last_line;
            }
            Err(e) => {
                eprintln!("Error reading file: {:?}", e);
            }
        }
        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            match tail_file(&file_path, 0, &regex_pattern, &date_format, &log_data_clone, no_clear, last_processed_line.clone()).await {
                Ok(last_line) => {
                    last_processed_line = last_line;
                }
                Err(e) => {
                    eprintln!("Error reading file: {:?}", e);
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });


    let mut app = App::new(log_data, top_n);

    loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    tx.send(()).unwrap();
    handle.await.unwrap();

    Ok(())
}

struct LogEntry {
    count: usize,
    last_update: SystemTime,
    last_requests: Vec<String>,
    request_type: String,
    request_domain: String,
}

struct LogData {
    by_ip: HashMap<String, LogEntry>,
    by_url: HashMap<String, LogEntry>,
    total_requests: usize,
    requests_per_interval: HashMap<i64, usize>,
}

impl LogData {
    fn new() -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
            requests_per_interval: HashMap::new(),
        }
    }

    fn add_entry(&mut self, ip: String, url: String, log_line: String, timestamp: i64, request_type: String, request_domain: String, no_clear: bool) {
        let now = SystemTime::now();

        {
            let entry = self.by_ip.entry(ip.clone()).or_insert(LogEntry {
                count: 0,
                request_type: request_type.clone(),
                request_domain: request_domain.clone(),
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
                request_type: request_type.clone(),
                request_domain: request_domain.clone(),
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

        *self.requests_per_interval.entry(timestamp).or_insert(0) += 1;

        // Удаление устаревших данных
        let threshold = timestamp - (20 * 60); // 20 минут назад
        self.requests_per_interval.retain(|&k, _| k >= threshold);
    }

    fn clear_outdated_entries(&mut self) {
        let threshold = SystemTime::now() - Duration::from_secs(1200);
        self.by_ip.retain(|_, entry| entry.last_update >= threshold);
        self.by_url.retain(|_, entry| entry.last_update >= threshold);
    }

    fn get_top_n(&self, n: usize) -> (Vec<(String, &LogEntry)>, Vec<(String, &LogEntry)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
            top_url.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
        )
    }

    fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }

    fn get_last_requests(&self, ip: &str) -> Vec<String> {
        self.by_ip.get(ip).map_or(Vec::new(), |entry| entry.last_requests.clone())
    }
}

async fn tail_file(
    file_path: &PathBuf,
    count: isize,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
    last_processed_line: Option<String>,
) -> std::io::Result<Option<String>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut last_processed = last_processed_line.clone();
    if let Some(last_line) = last_processed_line {
        // Find the last processed line in the file and set the cursor position
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;
        if let Some(pos) = buffer.find(last_line.as_str()) {
            reader.seek(SeekFrom::Start(pos as u64 + last_line.len() as u64))?;
        }
    } else if count > 0 {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let content = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > count as usize { lines.len() - count as usize } else { 0 };

        for line in &lines[start..] {
            process_line(line, &regex_pattern, date_format, log_data, no_clear).await?;
            last_processed = Some(line.to_string());
        }
    } else if count == -1 {
        reader.seek(SeekFrom::Start(0))?;

        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line)?;

            if len == 0 {
                break;
            }

            process_line(&line, &regex_pattern, date_format, log_data, no_clear).await?;
            last_processed = Some(line);
        }
    } else {
        reader.seek(SeekFrom::End(0))?;
    }

    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;

        if len == 0 {
            break;
        }

        process_line(&line, &regex_pattern, date_format, log_data, no_clear).await?;
        last_processed = Some(line);
    }

    Ok(last_processed)
}

async fn process_line(
    line: &str,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
) -> std::io::Result<()> {
    let re = Regex::new(regex_pattern).unwrap();
    if let Some(caps) = re.captures(line) {
        let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let datetime_str = caps.get(2).map_or("", |m| m.as_str()).to_string();
        let request_domain = caps.get(3).map_or("", |m| m.as_str()).to_string();
        let request_type = caps.get(4).map_or("", |m| m.as_str()).to_string();
        let url = caps.get(5).map_or("", |m| m.as_str()).to_string();

        let datetime = DateTime::parse_from_str(&datetime_str, date_format)
            .or_else(|_| DateTime::parse_from_str(&datetime_str, "%d/%b/%Y:%H:%M:%S")
                .map(|dt| dt.with_timezone(&Utc.fix()))
                .map_err(|_: chrono::ParseError| ())
            )
            .unwrap_or_else(|_| Utc::now().with_timezone(&Utc.fix()));

        let mut log_data = log_data.lock().unwrap();
        log_data.add_entry(ip, url, line.to_string(), datetime.timestamp(), request_type, request_domain, no_clear);
    } else {
        println!("No match for line: {}", line);
    }

    Ok(())
}


struct App {
    log_data: Arc<Mutex<LogData>>,
    should_quit: bool,
    top_n: usize,
    current_tab: usize,
    last_requests_state: ListState,
    ip_list_state: ListState,
    request_list_state: ListState,
    input: String,
    current_page: usize,
    total_pages: usize,
    pages: Vec<String>,
}


impl App {
    fn new(log_data: Arc<Mutex<LogData>>, top_n: usize) -> Self {
        Self {
            log_data,
            should_quit: false,
            top_n,
            current_tab: 0,
            last_requests_state: ListState::default(),
            ip_list_state: ListState::default(),
            request_list_state: ListState::default(),
            input: String::new(),
            current_page: 0,
            total_pages: 0,
            pages: Vec::default(),
        }
    }

    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Tab => {
                self.toggle_tab();
            }
            KeyCode::Char('t') => {
                self.toggle_tab();
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit();
            }
            KeyCode::Up => {
                self.on_up();
            }
            KeyCode::Down => {
                self.on_down();
            }
            KeyCode::Left => {
                self.on_left();
            }
            KeyCode::Right => {
                self.on_right();
            }
            KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit();
            }
            KeyCode::Backspace => {
                self.last_requests_state.select(None);
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.last_requests_state.select(None);
                self.input.push(c);
            }
            _ => {}
        }
    }


    fn draw(&mut self, frame: &mut ratatui::terminal::Frame) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .split(chunks[0]);

        self.draw_tabs(frame, header_chunks[0]);
        self.draw_summary(frame, header_chunks[1]);

        match self.current_tab {
            0 => self.draw_overview(frame, chunks[1]),
            1 => self.draw_last_requests(frame, chunks[1]),
            2 => self.draw_detailed_requests(frame, chunks[1]),
            3 => self.draw_requests_chart(frame, chunks[1]),
            _ => {}
        }
    }

    fn draw_requests_chart(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();

        // Сортировка данных по ключам (временным меткам) в обратном порядке
        let mut sorted_data: Vec<_> = log_data.requests_per_interval.iter().collect();
        sorted_data.sort_by_key(|&(k, _)| k);
        sorted_data.reverse();

        // Преобразование вектора в данные для Sparkline
        let mut data: Vec<u64> = sorted_data.iter().map(|&(_, v)| *v as u64).collect();

        // Ограничение данных до ширины области для Sparkline
        if data.len() > area.width as usize {
            data.truncate(area.width as usize);
        }

        if data.is_empty() {
            return;
        }

        let min_value = data.iter().min().unwrap_or(&0);
        let max_value = data.iter().max().unwrap_or(&0);
        let start_time = sorted_data.last().map(|(&k, _)| k).unwrap_or(0);
        let end_time = sorted_data.first().map(|(&k, _)| k).unwrap_or(0);

        let sparkline_title = format!(
            "Requests over last 20 minutes (Min: {}, Max: {}, Start: {}, End: {})",
            min_value,
            max_value,
            start_time,
            end_time
        );

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(sparkline_title),
            )
            .data(&data)
            .direction(RenderDirection::RightToLeft)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(sparkline, area);
    }


    fn draw_tabs(&self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let titles: Vec<String> = ["Overview", "Last Requests", "Detailed Requests", "Requests Chart"]
            .iter()
            .cloned()
            .map(|t| t.into())
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.current_tab)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|");
        frame.render_widget(tabs, area);
    }

    fn draw_summary(&self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let (unique_ips, unique_urls) = log_data.get_unique_counts();

        let now = Local::now();
        let summary = format!(
            "Total requests: {} | Unique IPs: {} | Unique URLs: {} | Last update: {}",
            log_data.total_requests, unique_ips, unique_urls, now.format("%Y-%m-%d %H:%M:%S")
        );

        frame.render_widget(
            Paragraph::new(summary).block(Block::default().borders(Borders::ALL).title("Summary")),
            area,
        );
    }

    fn draw_overview(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Min(0)].as_ref())
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[1]);

        let ip_rows: Vec<Row> = top_ips
            .into_iter()
            .map(|(ip, entry)| {
                let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                Row::new(vec![ip, entry.count.to_string(), last_update_str])
            })
            .collect();
        let widths = [Constraint::Length(50), Constraint::Length(20), Constraint::Length(30)];
        let ip_table = Table::new(ip_rows, widths)
            .block(Block::default().borders(Borders::ALL).title("Top IPs"))
            .header(Row::new(vec!["IP", "Requests", "Last Update"]).style(Style::default().fg(Color::Yellow)));
        frame.render_widget(ip_table, chunks[0]);

        let url_rows: Vec<Row> = top_urls
            .into_iter()
            .map(|(url, entry)| {
                let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                Row::new(vec![entry.request_type.to_string(), entry.request_domain.to_string(), url, entry.count.to_string(), last_update_str])
            })
            .collect();
        let widths = [Constraint::Length(10), Constraint::Length(20),Constraint::Length(90), Constraint::Length(10), Constraint::Length(20)];
        let url_table = Table::new(url_rows, widths)
            .block(Block::default().borders(Borders::ALL).title("Top URLs"))
            .header(Row::new(vec!["Type", "Domain", "URL", "Requests", "Last Update"]).style(Style::default().fg(Color::Yellow)));
        frame.render_widget(url_table, chunks[1]);
    }



    fn draw_last_requests(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut items: Vec<ListItem> = vec![];
        let search_results: Vec<&String>;

        if !self.input.is_empty() {
            search_results = log_data
                .by_ip
                .iter()
                .flat_map(|(_, entry)| &entry.last_requests)
                .filter(|request| request.contains(&self.input))
                .collect();
        } else {
            search_results = log_data
                .by_ip
                .values()
                .flat_map(|entry| &entry.last_requests)
                .collect();
        }

        self.total_pages = (search_results.len() + 99) / 100;
        let start = self.current_page * 100;
        let end = (start + 100).min(search_results.len());

        for request in &search_results[start..end] {
            let wrapped_text = wrap(request, (area.width as f64 * 0.7) as usize - 5);
            let list_item = ListItem::new(wrapped_text.join("\n")).style(Style::default().fg(TEXT_FG_COLOR));
            items.push(list_item);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        frame.render_widget(input, chunks[0]);

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">");
        frame.render_stateful_widget(list, chunks[1], &mut self.last_requests_state);

        // Обновляем страницы
        self.pages = (1..=self.total_pages).map(|i| format!("{}", i)).collect();

        let page_titles = self.pages.iter().cloned().collect::<Vec<_>>();
        let pagination_tabs = Tabs::new(page_titles)
            .select(self.current_page)
            .block(Block::default().borders(Borders::ALL).title("Pages"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|");
        frame.render_widget(pagination_tabs, chunks[2]);
    }

    fn draw_detailed_requests(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();

        let top_ips = log_data.get_top_n(self.top_n).0;
        let ip_items: Vec<ListItem> = top_ips
            .iter()
            .map(|(ip, _)| ListItem::new(ip.clone()).style(Style::default().fg(Color::Yellow)))
            .collect();

        let selected_ip = self.ip_list_state.selected().and_then(|i| top_ips.get(i).map(|(ip, _)| ip.clone()));

        let mut request_items: Vec<ListItem> = vec![];
        if let Some(ip) = selected_ip {
            let last_requests = log_data.get_last_requests(&ip);
            for request in last_requests {
                let wrapped_text = wrap(&request, (area.width as f64 * 0.7) as usize - 5);
                let list_item = ListItem::new(wrapped_text.join("\n")).style(Style::default().fg(TEXT_FG_COLOR));
                request_items.push(list_item);
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(area);

        let ip_list = List::new(ip_items)
            .block(Block::default().borders(Borders::ALL).title("IP Addresses").style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ");
        frame.render_stateful_widget(ip_list, chunks[0], &mut self.ip_list_state);

        let request_list = List::new(request_items)
            .block(Block::default().borders(Borders::ALL).title("Requests").style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE);
        frame.render_stateful_widget(request_list, chunks[1], &mut self.request_list_state);

        if self.ip_list_state.selected().is_none() {
            self.ip_list_state.select(Some(0));
        }
    }


    fn on_up(&mut self) {
        match self.current_tab {
            1 => self.last_requests_state.select_previous(),
            2 => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select_previous();
                } else {
                    self.ip_list_state.select_previous();
                }
            },
            _ => {}
        }
    }

    fn on_down(&mut self) {
        match self.current_tab {
            1 => self.last_requests_state.select_next(),
            2 => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select_next();
                } else {
                    self.ip_list_state.select_next();
                }
            },
            _ => {}
        }
    }

    fn on_left(&mut self) {
        match self.current_tab {
            1 => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.last_requests_state.select_first()
                }
            },
            2 => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select(None);
                }
            },
            _ => {  },
        }
    }

    fn on_right(&mut self) {
        match self.current_tab {
            1 => {
                if self.current_page < self.total_pages - 1 {
                    self.current_page += 1;
                    self.last_requests_state.select_first()
                }
            },
            2 => {
                if self.ip_list_state.selected().is_some() {
                    self.request_list_state.select(Some(0));
                }
            },
            _ => {  },
        }
    }

    fn toggle_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}
