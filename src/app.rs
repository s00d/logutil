use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Instant};
use chrono::{Local, Timelike, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{ListItem, Block, Borders, ListState},
    Frame,
};
use textwrap::wrap;
use crate::log_data::LogData;
use crate::tui_manager::{TuiManager, TEXT_FG_COLOR, HEADER_STYLE};
use clipboard::{ClipboardContext, ClipboardProvider};

struct ModalState {
    message: String,
    show_until: Option<Instant>,
}

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
    overview_panel: usize, // 0 - left panel (IP), 1 - right panel (URL)
    modal_state: Option<ModalState>,
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
            overview_panel: 0, // Left panel selected by default
            modal_state: None,
        }
    }

    pub(crate) fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 100.0);
    }

    pub(crate) fn handle_input(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => self.should_quit = true,
            KeyCode::Enter => {
                if self.current_tab == 0 {
                    self.copy_selected_to_clipboard();
                }
            }
            KeyCode::Up => {
                match self.current_tab {
                    0 => {
                        match self.overview_panel {
                            0 => self.top_ip_list_state.select_previous(),
                            1 => self.top_url_list_state.select_previous(),
                            _ => {}
                        }
                    }
                    1 => {
                        self.last_requests_state.select_previous();
                    }
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
            KeyCode::Down => {
                match self.current_tab {
                    0 => {
                        match self.overview_panel {
                            0 => self.top_ip_list_state.select_next(),
                            1 => self.top_url_list_state.select_next(),
                            _ => {}
                        }
                    }
                    1 => {
                        self.last_requests_state.select_next();
                    }
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
            KeyCode::Tab | KeyCode::Char('t') => self.toggle_tab(),
            KeyCode::Left => self.on_left(),
            KeyCode::Right => self.on_right(),
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
        let size = frame.area();
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
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(60),
                Constraint::Percentage(10)
            ].as_ref())
            .split(chunks[0]);

        // Улучшенное отображение вкладок
        frame.render_widget(
            self.tui_manager.draw_tabs(
                vec!["Overview".into(), "Requests".into(), "Detailed".into(), "Sparkline".into(), "Heatmap".into()],
                self.current_tab,
                "Navigation"
            ).style(HEADER_STYLE)
            .highlight_style(Style::new().fg(Color::White).bg(Color::Rgb(0, 95, 135))),
            header_chunks[0]
        );

        // Возвращаем использование draw_summary
        frame.render_widget(
            self.tui_manager.draw_summary(&self.get_summary_text())
                .style(HEADER_STYLE)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))),
            header_chunks[1]
        );

        // Улучшенный прогресс-бар
        frame.render_widget(
            self.tui_manager.draw_progress_bar(self.progress)
                .style(Style::new().fg(Color::Rgb(144, 238, 144)).bg(Color::DarkGray)),
            header_chunks[2]
        );

        match self.current_tab {
            0 => self.draw_overview(frame, chunks[1]),
            1 => self.draw_last_requests(frame, chunks[1]),
            2 => self.draw_detailed_requests(frame, chunks[1]),
            3 => self.draw_requests_sparkline(frame, chunks[1]),
            4 => self.draw_heatmap(frame, chunks[1]),
            _ => {}
        }

        // Проверяем и обновляем состояние модального окна
        if let Some(modal) = &self.modal_state {
            if let Some(show_until) = modal.show_until {
                if Instant::now() > show_until {
                    self.modal_state = None;
                } else {
                    self.draw_modal(frame);
                }
            }
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

        // Format list items
        let ip_items: Vec<ListItem> = top_ips.iter().map(|(ip, entry)| {
            self.tui_manager.format_ip_item(ip, entry, self.overview_panel == 0)
        }).collect();

        let url_items: Vec<ListItem> = top_urls.iter().map(|(url, entry)| {
            self.tui_manager.format_url_item(url, entry, self.overview_panel == 1)
        }).collect();

        // Use TuiManager to draw the overview
        self.tui_manager.draw_overview(
            frame,
            area,
            ip_items,
            url_items,
            self.overview_panel,
            &mut self.top_ip_list_state,
            &mut self.top_url_list_state,
        );
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
                    ListItem::new(wrapped_text.join("\n"))
                        .style(Style::default().fg(TEXT_FG_COLOR))
                })
                .collect();
        }

        self.total_pages = total_pages;

        self.tui_manager.draw_last_requests(
            frame,
            area,
            items,
            &self.input,
            self.current_page,
            self.total_pages,
            &mut self.last_requests_state,
        );
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
                self.tui_manager.format_ip_item(ip, entry, self.ip_list_state.selected().is_some())
            })
            .collect();

        let selected_ip = self.ip_list_state.selected()
            .and_then(|i| top_ips.get(i).map(|(ip, _)| ip.clone()));

        let mut request_items: Vec<ListItem> = vec![];
        if let Some(ip) = selected_ip.clone() {
            let last_requests = log_data.get_last_requests(&ip);
            for request in last_requests {
                let wrapped_text = wrap(&request, (area.width as f64 * 0.7) as usize - 5);
                let list_item = ListItem::new(wrapped_text.join("\n"))
                    .style(Style::default().fg(TEXT_FG_COLOR));
                request_items.push(list_item);
            }
        }

        self.tui_manager.draw_detailed_requests(
            frame,
            area,
            ip_items,
            request_items,
            selected_ip,
            &mut self.ip_list_state,
            &mut self.request_list_state,
        );

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

        let (min_value, max_value, start_time, end_time) = self.tui_manager.get_sparkline_bounds(&data, &sorted_data);

        self.tui_manager.draw_requests_sparkline(
            frame,
            area,
            &data,
            min_value,
            max_value,
            start_time,
            end_time,
        );
    }

    fn draw_heatmap(&mut self, frame: &mut Frame, area: Rect) {
        let log_data = self.log_data.lock().unwrap();
        let mut sorted_data: Vec<_> = log_data.requests_per_interval.iter()
            .map(|(&k, &v)| (k, v as u64))
            .collect();
        
        // Сортируем по дате и времени
        sorted_data.sort_by_key(|&(timestamp, _)| {
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            (datetime.date_naive(), datetime.hour())
        });

        // Получаем минимальное и максимальное значения для нормализации
        let min_value = sorted_data.iter().map(|&(_, v)| v).min().unwrap_or(0);
        let max_value = sorted_data.iter().map(|&(_, v)| v).max().unwrap_or(1);

        // Получаем уникальные даты
        let mut unique_dates: Vec<_> = sorted_data.iter()
            .map(|&(timestamp, _)| {
                let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
                datetime.date_naive()
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        unique_dates.sort();

        let cells = self.tui_manager.generate_heatmap_cells(&sorted_data, min_value, max_value, &unique_dates);

        // Добавляем подписи для осей
        let x_labels: Vec<(f64, String)> = (0..24).map(|hour| (hour as f64 + 1.7, format!("{:02}:00", hour))).collect();

        let y_labels: Vec<(f64, String)> = unique_dates.iter()
            .enumerate()
            .map(|(index, date)| (index as f64 + 1.0, date.format("%Y-%m-%d").to_string()))
            .collect();

        self.tui_manager.render_heatmap(
            frame,
            area,
            cells,
            x_labels,
            y_labels,
            min_value,
            max_value,
        );
    }

    fn on_left(&mut self) {
        match self.current_tab {
            0 => {
                if self.overview_panel > 0 {
                    self.overview_panel -= 1;
                }
            }
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
            0 => {
                if self.overview_panel < 1 {
                    self.overview_panel += 1;
                }
            }
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

    fn draw_modal(&self, frame: &mut Frame) {
        if let Some(modal) = &self.modal_state {
            self.tui_manager.draw_modal(frame, &modal.message);
        }
    }

    fn copy_selected_to_clipboard(&mut self) {
        let log_data = self.log_data.lock().unwrap();
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let (text_to_copy, message) = match self.overview_panel {
            0 => {
                if let Some(selected) = self.top_ip_list_state.selected() {
                    if selected > 0 {
                        if let Some((ip, entry)) = top_ips.get(selected - 1) {
                            let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                            let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                            (
                                format!("IP: {}\nRequests: {}\nLast Update: {}", ip, entry.count, last_update_str),
                                format!("IP address copied: {}", ip)
                            )
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            1 => {
                if let Some(selected) = self.top_url_list_state.selected() {
                    if selected > 0 {
                        if let Some((url, entry)) = top_urls.get(selected - 1) {
                            let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                            let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
                            (
                                format!(
                                    "URL: {}\nType: {}\nDomain: {}\nRequests: {}\nLast Update: {}",
                                    url, entry.request_type, entry.request_domain, entry.count, last_update_str
                                ),
                                format!("URL copied\n{}", url)
                            )
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            _ => return,
        };

        if let Ok(mut ctx) = ClipboardContext::new() {
            if ctx.set_contents(text_to_copy).is_ok() {
                self.modal_state = Some(ModalState {
                    message,
                    show_until: Some(Instant::now() + std::time::Duration::from_millis(1500)),
                });
            }
        }
    }
}
