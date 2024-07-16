use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use chrono::{Local, Timelike, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{ListItem, ListState};
use ratatui::widgets::canvas::Rectangle;
use textwrap::wrap;
use crate::log_data::LogData;
use crate::tui_manager::{TuiManager, TEXT_FG_COLOR};


pub struct App {
    log_data: Arc<Mutex<LogData>>,
    pub(crate) should_quit: bool,
    top_n: usize,
    current_tab: usize,
    last_requests_state: ListState,
    ip_list_state: ListState,
    request_list_state: ListState,
    top_ip_list_state: ListState,
    top_url_list_state: ListState,
    input: String,
    current_page: usize,
    total_pages: usize,
    progress: f64,
    tui_manager: TuiManager,
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
            top_ip_list_state: ListState::default(),
            top_url_list_state: ListState::default(),
            input: String::new(),
            current_page: 0,
            total_pages: 0,
            progress: 0.0,
            tui_manager: TuiManager::new(),
        }
    }

    pub(crate) fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 100.0);
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

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ].as_ref())
            .split(size);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(60), Constraint::Percentage(10)].as_ref())
            .split(chunks[0]);

        frame.render_widget(self.tui_manager.draw_tabs(
            vec!["Overview".into(), "Requests".into(), "Detailed".into(), "Sparkline".into(), "Heatmap".into()],
            self.current_tab,
            "Tabs"
        ), header_chunks[0]);

        frame.render_widget(self.tui_manager.draw_summary(
            &self.get_summary_text()
        ), header_chunks[1]);

        frame.render_widget(self.tui_manager.draw_progress_bar(self.progress), header_chunks[2]);

        match self.current_tab {
            0 => self.draw_overview(frame, chunks[1]),
            1 => self.draw_last_requests(frame, chunks[1]),
            2 => self.draw_detailed_requests(frame, chunks[1]),
            3 => self.draw_requests_sparkline(frame, chunks[1]),
            4 => self.draw_heatmap(frame, chunks[1]),
            _ => {}
        }
    }

    fn get_summary_text(&self) -> String {
        let log_data = self.log_data.lock().unwrap();
        let (unique_ips, unique_urls) = log_data.get_unique_counts();
        let now = Local::now();
        format!(
            "Requests: {} | Unique IPs: {} | Unique URLs: {} | Update: {}",
            log_data.total_requests, unique_ips, unique_urls, now.format("%Y-%m-%d %H:%M:%S")
        )
    }

    fn draw_overview(&mut self, frame: &mut Frame, area: Rect) {
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


        // Top IPs
        let ip_items: Vec<ListItem> = top_ips.iter().map(|(ip, entry)| {
            let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
            ListItem::new(format!("{:<15} | {:<8} | {}", ip, entry.count, last_update_str))
        }).collect();


        frame.render_stateful_widget(self.tui_manager.draw_list(ip_items.clone(), format!("{:<15} | {:<8} | {}", "Top IPs", "Requests", "Last Update").to_string()), chunks[0], &mut self.top_ip_list_state);

        self.tui_manager.draw_scrollbar(ip_items.len(), self.top_ip_list_state.selected().unwrap_or(0), frame, chunks[0]);

        // Top URLs
        let url_items: Vec<ListItem> = top_urls.iter().map(|(url, entry)| {
            let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
            ListItem::new(format!("{:<50} | {:<20} | {:<6} | {:<8} | {}", url, entry.request_type, entry.request_domain, entry.count, last_update_str))
        }).collect();

        frame.render_stateful_widget(self.tui_manager.draw_list(url_items.clone(), format!("{:<50} | {:<20} | {:<6} | {:<8} | {}",  "Top URLs", "Type", "Domain", "Requests", "Last Update").to_string()), chunks[1], &mut self.top_url_list_state);

        self.tui_manager.draw_scrollbar(url_items.len(), self.top_url_list_state.selected().unwrap_or(0), frame, chunks[1]);
    }



    fn draw_last_requests(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem>;
        let total_pages: usize;

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
        }

        self.total_pages = total_pages;
        let pages: Vec<String> = (1..=self.total_pages).map(|i| format!("{}", i)).collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);


        frame.render_widget(self.tui_manager.draw_input(&self.input), header_chunks[0]);
        frame.render_widget(self.tui_manager.draw_pagination(pages.clone(), self.current_page), header_chunks[1]);

        frame.render_stateful_widget(self.tui_manager.draw_list(items.clone(), "".to_string()), chunks[1], &mut self.last_requests_state);
        self.tui_manager.draw_scrollbar(items.len(), self.last_requests_state.selected().unwrap_or(0), frame, chunks[1]);
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

    fn draw_detailed_requests(&mut self, frame: &mut Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut top_ips = log_data.get_top_n(self.top_n).0;

        top_ips.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        let ip_items: Vec<ListItem> = top_ips
            .iter()
            .map(|(ip, entry)| {
                ListItem::new(format!("{:<15} ({})", ip, entry.count)).style(Style::default().fg(Color::Yellow))
            })
            .collect();

        let selected_ip = self.ip_list_state.selected().and_then(|i| top_ips.get(i).map(|(ip, _)| ip.clone()));

        let mut request_items: Vec<ListItem> = vec![];
        if let Some(ip) = selected_ip.clone() {
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



        frame.render_stateful_widget(self.tui_manager.draw_list(ip_items.clone(), "Top IPs".to_string()), chunks[0], &mut self.ip_list_state);
        self.tui_manager.draw_scrollbar(ip_items.len(), self.ip_list_state.selected().unwrap_or(0), frame, chunks[0]);

        let request_list_title = if let Some(ip) = selected_ip.clone() {
            format!("Requests for IP: {}", ip)
        } else {
            "Requests".to_string()
        };

        frame.render_stateful_widget(self.tui_manager.draw_list(request_items.clone(), request_list_title), chunks[1], &mut self.request_list_state);
        self.tui_manager.draw_scrollbar(request_items.len(), self.request_list_state.selected().unwrap_or(0), frame, chunks[1]);

        if self.ip_list_state.selected().is_none() {
            self.ip_list_state.select(Some(0));
        }
    }

    fn draw_requests_sparkline(&mut self, frame: &mut Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut sorted_data: Vec<_> = log_data.requests_per_interval.iter().map(|(&k, &v)| (k, v as u64)).collect();
        sorted_data.sort_by_key(|&(k, _)| k);
        sorted_data.reverse();

        let mut data: Vec<u64> = sorted_data.iter().map(|&(_, v)| v).collect();
        if data.len() > area.width as usize {
            data.truncate(area.width as usize);
        }

        if data.is_empty() {
            return;
        }

        let (min_value, max_value, start_time, end_time) = self.get_sparkline_bounds(&data, &sorted_data);

        let sparkline_title = format!(
            "Requests over last 20 minutes (Min: {}, Max: {}, Start: {}, End: {})",
            min_value,
            max_value,
            start_time,
            end_time
        );

        frame.render_widget(self.tui_manager.draw_sparkline(&data, &sparkline_title), area);
    }

    fn get_sparkline_bounds(&self, data: &[u64], sorted_data: &[(i64, u64)]) -> (u64, u64, i64, i64) {
        let min_value = *data.iter().min().unwrap_or(&0);
        let max_value = *data.iter().max().unwrap_or(&0);
        let start_time = sorted_data.last().map(|&(k, _)| k).unwrap_or(0);
        let end_time = sorted_data.first().map(|&(k, _)| k).unwrap_or(0);
        (min_value, max_value, start_time, end_time)
    }

    fn draw_heatmap(&mut self, frame: &mut Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut sorted_data: Vec<_> = log_data.requests_per_interval.iter().map(|(&k, &v)| (k, v as u64)).collect();
        sorted_data.sort_by_key(|&(timestamp, _)| {
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            datetime.date_naive()
        });

        let min_value = sorted_data.iter().map(|&(_, v)| v).min().unwrap_or(0);
        let max_value = sorted_data.iter().map(|&(_, v)| v).max().unwrap_or(1);

        let mut unique_dates: Vec<_> = sorted_data.iter()
            .map(|&(timestamp, _)| {
                let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
                datetime.date_naive()
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        unique_dates.sort();

        let cells = self.generate_heatmap_cells(&sorted_data, min_value, max_value, &unique_dates);

        let x_labels: Vec<(f64, String)> = (0..24).map(|hour| (hour as f64 + 1.7, format!("{}", hour))).collect();
        let y_labels: Vec<(f64, String)> = unique_dates.iter().enumerate().map(|(index, date)| (index as f64 + 1.0, date.format("%Y-%m-%d").to_string())).collect();

        frame.render_widget(self.tui_manager.draw_heatmap(cells, x_labels, y_labels), area);
    }

    fn generate_heatmap_cells(&self, sorted_data: &[(i64, u64)], min_value: u64, max_value: u64, unique_dates: &[chrono::NaiveDate]) -> Vec<Rectangle> {
        let mut cells = Vec::new();

        for &(timestamp, value) in sorted_data.iter() {
            let intensity = (value as f64 - min_value as f64) / (max_value as f64 - min_value as f64);
            let color = Color::Rgb(
                (intensity * 255.0) as u8,
                0,
                (255.0 - intensity * 255.0) as u8,
            );

            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
            let hour = datetime.hour() as f64;
            let date_index = unique_dates.iter().position(|&d| d == datetime.date_naive()).unwrap() as f64;

            cells.push(Rectangle {
                x: hour + 1.3,
                y: date_index + 0.9,
                width: 0.8,
                height: 0.75,
                color,
            });
        }

        cells
    }

    fn on_up(&mut self) {
        match self.current_tab {
            0 => {
                self.top_ip_list_state.select_previous();
                self.top_url_list_state.select_previous();
            }
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
            0 => {
                self.top_ip_list_state.select_next();
                self.top_url_list_state.select_next();
            }
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
        self.current_tab = (self.current_tab + 1) % 5;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}
