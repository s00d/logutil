use crate::log_data::{LogData, LogEntry};
use crate::tui_manager::SELECTED_ITEM_STYLE;
use arboard::Clipboard;
use chrono::{Local, TimeZone};
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

    fn draw_overview(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let ip_items: Vec<Row> = top_ips
            .iter()
            .enumerate()
            .map(|(i, (ip, entry))| self.format_ip_item(ip, entry, i))
            .collect();

        let url_items: Vec<(Row, &LogEntry)> = top_urls
            .into_iter()
            .enumerate()
            .map(|(i, (url, entry))| (self.format_url_item(&url, entry, i), entry))
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
        ])
        .style(
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(80, 80, 80)) // Серый фон для заголовка
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(
            Table::new(
                ip_items.clone(),
                [
                    Constraint::Length(15), // IP
                    Constraint::Length(12), // Requests
                    Constraint::Min(20),    // Last Update
                ],
            )
            .header(ip_header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(105, 105, 105)))
                    .title("IP List")
                    .title_style(
                        Style::new()
                            .fg(Color::Rgb(105, 105, 105))
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[0],
            &mut adjusted_ip_state,
        );

        // Draw URL list
        let url_header = Row::new(vec![
            Cell::from("URL").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Type").style(
                Style::new()
                    .fg(Color::Rgb(169, 169, 169))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Domain").style(
                Style::new()
                    .fg(Color::Rgb(192, 192, 192))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Requests").style(
                Style::new()
                    .fg(Color::Rgb(128, 128, 128))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Last Update").style(
                Style::new()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(80, 80, 80)) // Серый фон для заголовка
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(
            Table::new(
                url_items
                    .iter()
                    .map(|(item, _)| item.clone())
                    .collect::<Vec<Row>>(),
                [
                    Constraint::Length(50), // URL
                    Constraint::Length(20), // Type
                    Constraint::Length(30), // Domain
                    Constraint::Length(12), // Requests
                    Constraint::Min(20),    // Last Update
                ],
            )
            .header(url_header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(105, 105, 105)))
                    .title("URL List")
                    .title_style(
                        Style::new()
                            .fg(Color::Rgb(105, 105, 105))
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[1],
            &mut adjusted_url_state,
        );

        // Draw scrollbars
        self.draw_scrollbar(
            ip_items.len(),
            adjusted_ip_state.selected().unwrap_or(0),
            frame,
            main_chunks[0],
        );
        self.draw_scrollbar(
            url_items.len(),
            adjusted_url_state.selected().unwrap_or(0),
            frame,
            main_chunks[1],
        );

        // Отображаем информацию в нижней панели в зависимости от активной панели
        if self.overview_panel == 0 {
            // IP List активна - показываем выбранный IP
            if let Some(idx) = adjusted_ip_state.selected() {
                if idx < top_ips.len() {
                    if let Some((ip, _)) = top_ips.get(idx) {
                        frame.render_widget(
                            Paragraph::new(format!("Selected IP: {}", ip))
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_type(ratatui::widgets::BorderType::Rounded)
                                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144))),
                                )
                                .style(Style::default().fg(Color::White)),
                            chunks[1],
                        );
                    }
                }
            }
        } else {
            // URL List активна - показываем строку лога
            if let Some(idx) = adjusted_url_state.selected() {
                if idx < url_items.len() {
                    if let Some((_, entry)) = url_items.get(idx) {
                        frame.render_widget(
                            Paragraph::new(entry.full_url.as_str())
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_type(ratatui::widgets::BorderType::Rounded)
                                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144))),
                                )
                                .style(Style::default().fg(Color::White)),
                            chunks[1],
                        );
                    }
                }
            }
        }

        // Обновляем оригинальные состояния
        if let Some(idx) = adjusted_ip_state.selected() {
            self.top_ip_table_state.select(Some(idx));
        } else {
            self.top_ip_table_state.select(None);
        }
        if let Some(idx) = adjusted_url_state.selected() {
            self.top_url_table_state.select(Some(idx));
        } else {
            self.top_url_table_state.select(None);
        }
    }

    // Методы из TuiManager
    fn format_ip_item(&self, ip: &str, entry: &LogEntry, _index: usize) -> Row {
        let last_update = entry
            .last_update
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last_update_str = format!(
            "{}",
            Local
                .timestamp_opt(last_update as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );

        Row::new(vec![
            Cell::from(ip.to_string()).style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ), // IP - желтый, жирный
            Cell::from(entry.count.to_string()).style(Style::new().fg(Color::Rgb(169, 169, 169))), // Requests - темно-серый
            Cell::from(last_update_str).style(Style::new().fg(Color::Rgb(100, 149, 237))), // Last Update - cornflower blue
        ])
    }

    fn format_url_item(&self, url: &str, entry: &LogEntry, _index: usize) -> Row {
        let last_update = entry
            .last_update
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last_update_str = format!(
            "{}",
            Local
                .timestamp_opt(last_update as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );

        // Определяем тип запроса
        let request_type = if url.contains("GET") {
            "GET"
        } else if url.contains("POST") {
            "POST"
        } else if url.contains("PUT") {
            "PUT"
        } else if url.contains("DELETE") {
            "DELETE"
        } else {
            "OTHER"
        };

        // Извлекаем домен из URL
        let domain = if let Some(start) = url.find("://") {
            if let Some(end) = url[start + 3..].find('/') {
                &url[start + 3..start + 3 + end]
            } else {
                &url[start + 3..]
            }
        } else {
            "Unknown"
        };

        // Обрезаем URL для отображения
        let truncated_url = self.truncate_url(url, 50);

        Row::new(vec![
            Cell::from(truncated_url).style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ), // URL - желтый, жирный
            Cell::from(request_type.to_string()).style(Style::new().fg(Color::Rgb(169, 169, 169))), // Type - темно-серый
            Cell::from(domain.to_string()).style(Style::new().fg(Color::Rgb(192, 192, 192))), // Domain - серебряный
            Cell::from(entry.count.to_string()).style(Style::new().fg(Color::Rgb(128, 128, 128))), // Requests - серый
            Cell::from(last_update_str).style(Style::new().fg(Color::Rgb(100, 149, 237))), // Last Update - cornflower blue
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

    fn on_left(&mut self, log_data: &LogData) {
        if self.overview_panel > 0 {
            self.overview_panel -= 1;
            if self.overview_panel == 0 {
                self.top_url_table_state.select(None);
                // Проверяем, есть ли IP в списке
                let (top_ips, _) = log_data.get_top_n(self.top_n);
                if !top_ips.is_empty() {
                    self.top_ip_table_state.select(Some(0));
                }
            }
        }
    }

    fn on_right(&mut self, log_data: &LogData) {
        if self.overview_panel < 1 {
            self.overview_panel += 1;
            if self.overview_panel == 1 {
                self.top_ip_table_state.select(None);
                // Проверяем, есть ли URL в списке
                let (_, top_urls) = log_data.get_top_n(self.top_n);
                if !top_urls.is_empty() {
                    self.top_url_table_state.select(Some(0));
                }
            }
        }
    }

    pub fn copy_selected_to_clipboard(&self, log_data: &LogData) -> Option<String> {
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let (text_to_copy, message) = match self.overview_panel {
            0 => {
                if let Some(selected) = self.top_ip_table_state.selected() {
                    if let Some((ip, entry)) = top_ips.get(selected) {
                        let last_update = entry
                            .last_update
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let last_update_str = format!(
                            "{}",
                            Local
                                .timestamp_opt(last_update as i64, 0)
                                .unwrap()
                                .format("%Y-%m-%d %H:%M:%S")
                        );
                        (
                            format!(
                                "IP: {}\nRequests: {}\nLast Update: {}",
                                ip, entry.count, last_update_str
                            ),
                            format!("IP copied\n{}", ip),
                        )
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            1 => {
                if let Some(selected) = self.top_url_table_state.selected() {
                    if let Some((url, entry)) = top_urls.get(selected) {
                        let last_update = entry
                            .last_update
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let last_update_str = format!(
                            "{}",
                            Local
                                .timestamp_opt(last_update as i64, 0)
                                .unwrap()
                                .format("%Y-%m-%d %H:%M:%S")
                        );
                        (
                            format!(
                                "URL: {}\nType: {}\nDomain: {}\nRequests: {}\nLast Update: {}",
                                url,
                                entry.request_type,
                                entry.request_domain,
                                entry.count,
                                last_update_str
                            ),
                            format!("URL copied\n{}", url),
                        )
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(text_to_copy).is_ok() {
                return Some(message);
            }
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
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_overview(frame, area, log_data);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, log_data: &LogData) -> bool {
        match key.code {
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
                match self.overview_panel {
                    0 => {
                        if let Some(selected) = self.top_ip_table_state.selected() {
                            // Получаем количество IP из log_data для определения максимального индекса
                            let (top_ips, _) = log_data.get_top_n(self.top_n);
                            if selected < top_ips.len().saturating_sub(1) {
                                self.top_ip_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    1 => {
                        if let Some(selected) = self.top_url_table_state.selected() {
                            // Получаем количество URL из log_data для определения максимального индекса
                            let (_, top_urls) = log_data.get_top_n(self.top_n);
                            if selected < top_urls.len().saturating_sub(1) {
                                self.top_url_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Left => {
                self.on_left(log_data);
                true
            }
            crossterm::event::KeyCode::Right => {
                self.on_right(log_data);
                true
            }
            _ => false,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
