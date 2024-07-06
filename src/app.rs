use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use chrono::{Local, TimeZone};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, RenderDirection, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Sparkline, Table, Tabs};
use textwrap::wrap;
use crate::log_data::LogEntry;
use crate::LogData;

const NORMAL_ROW_BG: Color = Color::Rgb(18, 18, 20);
const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(0, 31, 63)).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = Color::Rgb(158, 158, 158);

pub struct App {
    log_data: Arc<Mutex<LogData>>,
    pub(crate) should_quit: bool,
    top_n: usize,
    current_tab: usize,
    last_requests_state: ListState,
    ip_list_state: ListState,
    request_list_state: ListState,
    input: String,
    current_page: usize,
    total_pages: usize,
    pages: Vec<String>,
    pub(crate) progress: f64,
}

impl App {
    pub(crate) fn new(log_data: Arc<Mutex<LogData>>, top_n: usize) -> Self {
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
            progress: 0.0,
        }
    }

    pub(crate) fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Tab | KeyCode::Char('t') => self.toggle_tab(),
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => self.quit(),
            KeyCode::Up => self.on_up(),
            KeyCode::Down => self.on_down(),
            KeyCode::Left => self.on_left(),
            KeyCode::Right => self.on_right(),
            KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => self.quit(),
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

    pub(crate) fn draw(&mut self, frame: &mut ratatui::terminal::Frame) {
        let size = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3) // добавлено для прогресс-бара

            ].as_ref())
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

        self.draw_progress_bar(frame, chunks[2]);
    }

    fn draw_progress_bar(&self, frame: &mut Frame, area: Rect) {  // добавлено
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Loading Progress"))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::ITALIC))
            .ratio(self.progress);
        frame.render_widget(gauge, area);
    }

    fn draw_requests_chart(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut sorted_data: Vec<_> = log_data.requests_per_interval.iter().collect();
        sorted_data.sort_by_key(|&(k, _)| k);
        sorted_data.reverse();

        let mut data: Vec<u64> = sorted_data.iter().map(|&(_, v)| *v as u64).collect();
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
            .block(Block::default().borders(Borders::ALL).title(sparkline_title))
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
            .collect();
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

        self.draw_ip_table(frame, chunks[0], &top_ips);
        self.draw_url_table(frame, chunks[1], &top_urls);
    }

    fn draw_ip_table(&self, frame: &mut ratatui::terminal::Frame, area: Rect, top_ips: &[(String, &LogEntry)]) {
        let ip_rows: Vec<Row> = top_ips
            .iter()
            .map(|(ip, entry)| {
                let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                Row::new(vec![ip.clone(), entry.count.to_string(), last_update_str])
            })
            .collect();

        let ip_table = Table::new(ip_rows, &[Constraint::Length(50), Constraint::Length(20), Constraint::Length(30)])
            .block(Block::default().borders(Borders::ALL).title("Top IPs"))
            .header(Row::new(vec!["IP", "Requests", "Last Update"]).style(Style::default().fg(Color::Yellow)));

        frame.render_widget(ip_table, area);
    }

    fn draw_url_table(&self, frame: &mut ratatui::terminal::Frame, area: Rect, top_urls: &[(String, &LogEntry)]) {
        let url_rows: Vec<Row> = top_urls
            .iter()
            .map(|(url, entry)| {
                let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                Row::new(vec![
                    entry.request_type.clone(),
                    entry.request_domain.clone(),
                    url.clone(),
                    entry.count.to_string(),
                    last_update_str,
                ])
            })
            .collect();

        let url_table = Table::new(url_rows, &[Constraint::Length(10), Constraint::Length(20), Constraint::Length(90), Constraint::Length(10), Constraint::Length(20)])
            .block(Block::default().borders(Borders::ALL).title("Top URLs"))
            .header(Row::new(vec!["Type", "Domain", "URL", "Requests", "Last Update"]).style(Style::default().fg(Color::Yellow)));

        frame.render_widget(url_table, area);
    }

    fn draw_last_requests(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        let items: Vec<ListItem>;
        let total_pages: usize;

        // Create a scope to limit the lifetime of the immutable borrow
        {
            let log_data = self.log_data.lock().unwrap();
            let search_results = self.get_search_results(&log_data);

            total_pages = (search_results.len() + 99) / 100;
            let start = self.current_page * 100;
            let end = (start + 100).min(search_results.len());

            items = search_results[start..end]
                .iter()
                .map(|request| {
                    let wrapped_text = wrap(request, (area.width as f64 * 0.7) as usize - 5);
                    ListItem::new(wrapped_text.join("\n")).style(Style::default().fg(TEXT_FG_COLOR))
                })
                .collect();
        } // End of the immutable borrow scope

        self.total_pages = total_pages;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        // Mutable borrows only occur after the immutable borrows are out of scope
        self.draw_input(frame, chunks[0]);
        self.draw_list(frame, chunks[1], items);
        self.draw_pagination(frame, chunks[2]);
    }

    fn get_search_results<'a>(&self, log_data: &'a LogData) -> Vec<&'a String> {
        if !self.input.is_empty() {
            log_data
                .by_ip
                .iter()
                .flat_map(|(_, entry)| &entry.last_requests)
                .filter(|request| request.contains(&self.input))
                .collect()
        } else {
            log_data
                .by_ip
                .values()
                .flat_map(|entry| &entry.last_requests)
                .collect()
        }
    }

    fn draw_input(&self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        frame.render_widget(input, area);
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect, items: Vec<ListItem>) {
        let count = items.len();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">");
        frame.render_stateful_widget(list, area, &mut self.last_requests_state);

        let selected_index = self.last_requests_state.selected().unwrap_or(0);
        self.draw_scrollbar(count, selected_index, frame, area);
    }

    fn draw_pagination(&mut self, frame: &mut ratatui::terminal::Frame, area: Rect) {
        self.pages = (1..=self.total_pages).map(|i| format!("{}", i)).collect();
        let page_titles = self.pages.iter().cloned().collect::<Vec<_>>();
        let pagination_tabs = Tabs::new(page_titles)
            .select(self.current_page)
            .block(Block::default().borders(Borders::ALL).title("Pages"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|");
        frame.render_widget(pagination_tabs, area);
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

        let count = ip_items.len();
        let ip_list = List::new(ip_items)
            .block(Block::default().borders(Borders::ALL).title("IP Addresses").style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ");
        frame.render_stateful_widget(ip_list, chunks[0], &mut self.ip_list_state);

        let selected_index = self.ip_list_state.selected().unwrap_or(0);
        self.draw_scrollbar(count, selected_index, frame, chunks[0]);

        let count = request_items.len();
        let request_list = List::new(request_items)
            .block(Block::default().borders(Borders::ALL).title("Requests").style(Style::default().bg(NORMAL_ROW_BG)))
            .highlight_style(SELECTED_STYLE);
        frame.render_stateful_widget(request_list, chunks[1], &mut self.request_list_state);

        let selected_index = self.request_list_state.selected().unwrap_or(0);
        self.draw_scrollbar(count, selected_index, frame, chunks[1]);

        if self.ip_list_state.selected().is_none() {
            self.ip_list_state.select(Some(0));
        }
    }

    fn draw_scrollbar(&self, count: usize, selected_index: usize, frame: &mut Frame, rect: Rect) {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(count)
            .position(selected_index);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            rect,
            &mut scrollbar_state,
        );
    }

    fn on_up(&mut self) {
        match self.current_tab {
            1 => {
                self.last_requests_state.select_previous()
            },
            2 => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select_previous();
                } else {
                    self.ip_list_state.select_previous();
                }
            }
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
            }
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
            }
            2 => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select(None);
                }
            }
            _ => {}
        }
    }

    fn on_right(&mut self) {
        match self.current_tab {
            1 => {
                if self.current_page < self.total_pages - 1 {
                    self.current_page += 1;
                    self.last_requests_state.select_first()
                }
            }
            2 => {
                if self.ip_list_state.selected().is_some() {
                    self.request_list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn toggle_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}

