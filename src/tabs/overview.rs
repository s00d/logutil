use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::SELECTED_ITEM_STYLE;
use arboard::Clipboard;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState,
    },
    Frame,
};
use std::time::SystemTime;
use chrono::TimeZone;

pub struct OverviewTab {
    top_ip_table_state: TableState,
    top_url_table_state: TableState,
    overview_panel: usize, // 0 - left panel (IP), 1 - right panel (URL)
    top_n: usize,
}

impl OverviewTab {
    pub fn new() -> Self {
        let mut instance = Self {
            top_ip_table_state: TableState::default(),
            top_url_table_state: TableState::default(),
            overview_panel: 0,
            top_n: 10,
        };

        // Инициализируем выделение для первой панели
        instance.top_ip_table_state.select(Some(0));

        instance
    }

    fn draw_overview(&mut self, frame: &mut Frame, area: Rect) {
        let db = &*GLOBAL_DB;
        let top_ips = db.get_top_ips(self.top_n);
        let top_urls = db.get_top_urls(self.top_n);
        


        let ip_items: Vec<Row> = top_ips
            .iter()
            .enumerate()
            .map(|(i, (ip, count))| self.format_ip_item(ip, *count, i))
            .collect();

        let url_items: Vec<Row> = top_urls
            .iter()
            .enumerate()
            .map(|(i, (url, count))| self.format_url_item(url, *count, i))
            .collect();

        // Разделяем область на основную часть и панель для полного URL
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(0),
                    Constraint::Length(3), // Высота панели для полного URL
                ]
                .as_ref(),
            )
            .split(area);

        // Разделяем основную часть на две колонки
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[0]);

        // Корректируем выделение для IP списка
        let ip_selected = self.top_ip_table_state.selected();
        let mut adjusted_ip_state = TableState::default();
        if let Some(idx) = ip_selected {
            adjusted_ip_state.select(Some(idx));
        }

        // Корректируем выделение для URL списка
        let url_selected = self.top_url_table_state.selected();
        let mut adjusted_url_state = TableState::default();
        if let Some(idx) = url_selected {
            adjusted_url_state.select(Some(idx));
        }

        // Draw IP list
        let ip_header = Row::new(vec![
            Cell::from("IP").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Requests").style(
                Style::new()
                    .fg(Color::Rgb(169, 169, 169))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Last Update").style(
                Style::new()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let ip_table = Table::new(
            ip_items,
            [
                Constraint::Length(15),
                Constraint::Length(10),
                Constraint::Min(20),
            ],
        )
        .header(ip_header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(255, 255, 0)))
                .title("Top IPs"),
        )
        .row_highlight_style(SELECTED_ITEM_STYLE);

        frame.render_stateful_widget(ip_table, main_chunks[0], &mut adjusted_ip_state);

        // Draw URL list
        let url_header = Row::new(vec![
            Cell::from("URL").style(
                Style::new()
                    .fg(Color::Rgb(255, 182, 193))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Requests").style(
                Style::new()
                    .fg(Color::Rgb(169, 169, 169))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Last Update").style(
                Style::new()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let url_table = Table::new(
            url_items,
            [
                Constraint::Length(50),
                Constraint::Length(10),
                Constraint::Min(20),
            ],
        )
        .header(url_header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(255, 182, 193)))
                .title("Top URLs"),
        )
        .row_highlight_style(SELECTED_ITEM_STYLE);

        frame.render_stateful_widget(url_table, main_chunks[1], &mut adjusted_url_state);

        // Draw scrollbars
        self.draw_scrollbar(top_ips.len(), adjusted_ip_state.selected().unwrap_or(0), frame, main_chunks[0]);
        self.draw_scrollbar(top_urls.len(), adjusted_url_state.selected().unwrap_or(0), frame, main_chunks[1]);

        // Draw full URL panel
        if let Some(selected_idx) = adjusted_url_state.selected() {
            if selected_idx < top_urls.len() {
                let (full_url, _) = &top_urls[selected_idx];
                let url_panel = Paragraph::new(full_url.clone())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(Style::new().fg(Color::Rgb(255, 182, 193)))
                            .title("Full URL"),
                    )
                    .style(Style::new().fg(Color::Rgb(255, 182, 193)));

                frame.render_widget(url_panel, chunks[1]);
            }
        }

        // Обновляем состояния
        self.top_ip_table_state = adjusted_ip_state;
        self.top_url_table_state = adjusted_url_state;
    }

    fn format_ip_item(&self, ip: &str, count: usize, _index: usize) -> Row {
        let db = &*GLOBAL_DB;
        let records = db.find_by_ip(ip);
        
        // Получаем время последнего запроса для этого IP
        let last_update = if let Some(latest_record) = records.iter().max_by_key(|r| r.timestamp) {
            latest_record.timestamp
        } else {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        };

        // Форматируем время в читаемый вид
        let last_update_str = format!(
            "{}",
            chrono::Local
                .timestamp_opt(last_update, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );

        Row::new(vec![
            Cell::from(ip.to_string()).style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", count)).style(Style::new().fg(Color::Rgb(169, 169, 169))),
            Cell::from(last_update_str).style(Style::new().fg(Color::Rgb(100, 149, 237))),
        ])
    }

    fn format_url_item(&self, url: &str, count: usize, _index: usize) -> Row {
        let db = &*GLOBAL_DB;
        let records = db.find_by_url(url);
        
        // Получаем время последнего запроса для этого URL
        let last_update = if let Some(latest_record) = records.iter().max_by_key(|r| r.timestamp) {
            latest_record.timestamp
        } else {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        };

        // Форматируем время в читаемый вид
        let last_update_str = format!(
            "{}",
            chrono::Local
                .timestamp_opt(last_update, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );

        let truncated_url = self.truncate_url(url, 45);

        Row::new(vec![
            Cell::from(truncated_url).style(
                Style::new()
                    .fg(Color::Rgb(255, 182, 193))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", count)).style(Style::new().fg(Color::Rgb(169, 169, 169))),
            Cell::from(last_update_str).style(Style::new().fg(Color::Rgb(100, 149, 237))),
        ])
    }

    fn truncate_url(&self, url: &str, max_length: usize) -> String {
        if url.len() <= max_length {
            url.to_string()
        } else {
            format!("{}...", &url[..max_length - 3])
        }
    }

    fn draw_scrollbar(&self, count: usize, selected_index: usize, frame: &mut Frame, rect: Rect) {
        if count > 0 {
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
    }

    fn on_left(&mut self) {
        self.overview_panel = 0;
        self.top_ip_table_state.select(Some(0));
        self.top_url_table_state.select(None);
    }

    fn on_right(&mut self) {
        self.overview_panel = 1;
        self.top_url_table_state.select(Some(0));
        self.top_ip_table_state.select(None);
    }

    pub fn copy_selected_to_clipboard(&self) -> Option<String> {
        let db = &*GLOBAL_DB;
        
        match self.overview_panel {
            0 => {
                // Copy selected IP
                if let Some(selected_idx) = self.top_ip_table_state.selected() {
                    let top_ips = db.get_top_ips(self.top_n);
                    if selected_idx < top_ips.len() {
                        let (ip, _) = &top_ips[selected_idx];
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if clipboard.set_text(ip).is_ok() {
                                return Some(format!("IP '{}' copied to clipboard", ip));
                            }
                        }
                    }
                }
            }
            1 => {
                // Copy selected URL
                if let Some(selected_idx) = self.top_url_table_state.selected() {
                    let top_urls = db.get_top_urls(self.top_n);
                    if selected_idx < top_urls.len() {
                        let (url, _) = &top_urls[selected_idx];
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if clipboard.set_text(url).is_ok() {
                                return Some(format!("URL '{}' copied to clipboard", url));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl Default for OverviewTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for OverviewTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.draw_overview(frame, area);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            crossterm::event::KeyCode::Left => {
                self.on_left();
                true
            }
            crossterm::event::KeyCode::Right => {
                self.on_right();
                true
            }
            crossterm::event::KeyCode::Up => {
                match self.overview_panel {
                    0 => {
                        if let Some(selected) = self.top_ip_table_state.selected() {
                            if selected > 0 {
                                self.top_ip_table_state.select(Some(selected - 1));
                            }
                        }
                    }
                    1 => {
                        if let Some(selected) = self.top_url_table_state.selected() {
                            if selected > 0 {
                                self.top_url_table_state.select(Some(selected - 1));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                let db = &*GLOBAL_DB;
                match self.overview_panel {
                    0 => {
                        let top_ips = db.get_top_ips(self.top_n);
                        if let Some(selected) = self.top_ip_table_state.selected() {
                            if selected < top_ips.len().saturating_sub(1) {
                                self.top_ip_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    1 => {
                        let top_urls = db.get_top_urls(self.top_n);
                        if let Some(selected) = self.top_url_table_state.selected() {
                            if selected < top_urls.len().saturating_sub(1) {
                                self.top_url_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Char('c') => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.copy_selected_to_clipboard();
                }
                true
            }
            _ => false,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
