use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::{SELECTED_ITEM_STYLE, TEXT_FG_COLOR};
use arboard::Clipboard;
use chrono::{Local, TimeZone};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};
use std::time::SystemTime;


/// Параметры для отрисовки детальных запросов
struct DrawDetailedRequestsParams<'a, 'b> {
    frame: &'a mut Frame<'b>,
    area: Rect,
    ip_rows: Vec<Row<'a>>,
    request_items: Vec<ListItem<'a>>,
    selected_ip: Option<String>,
    ip_table_state: &'a mut TableState,
    request_list_state: &'a mut ListState,
}

pub struct DetailedTab {
    ip_table_state: TableState,
    request_list_state: ListState,
    top_n: usize,
}

impl DetailedTab {
    pub fn new() -> Self {
        let mut instance = Self {
            ip_table_state: TableState::default(),
            request_list_state: ListState::default(),
            top_n: 10,
        };

        // Инициализируем выделение для IP таблицы
        instance.ip_table_state.select(Some(0));

        instance
    }

    pub fn copy_selected_to_clipboard(&self) -> Option<String> {
        let db = &*GLOBAL_DB;
        
        // Если выбран IP
        if let Some(ip_index) = self.ip_table_state.selected() {
            let top_ips = db.get_top_ips(self.top_n);
            if let Some((ip, _)) = top_ips.get(ip_index) {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(ip).is_ok() {
                        return Some("Copied IP to clipboard".to_string());
                    }
                }
            }
        }

        // Если выбран запрос
        if let Some(request_index) = self.request_list_state.selected() {
            if let Some(ip_index) = self.ip_table_state.selected() {
                let top_ips = db.get_top_ips(self.top_n);
                if let Some((ip, _)) = top_ips.get(ip_index) {
                    let records = db.find_by_ip(ip);
                    if let Some(record) = records.get(request_index) {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if clipboard.set_text(&record.log_line).is_ok() {
                                // Обрезаем текст для модального окна
                                let display_text = if record.log_line.len() > 80 {
                                    format!("{}...", &record.log_line[..80])
                                } else {
                                    record.log_line.clone()
                                };
                                let message = format!("Copied request to clipboard:\n{}", display_text);
                                return Some(message);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Formats an IP table row
    fn format_ip_item(&self, ip: &str, count: usize, _is_active: bool) -> Row {
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
        
        let last_update_str = format!(
            "{}",
            Local
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

    /// Formats a request list item
    fn format_request_item(&self, request: &str, _index: usize) -> ListItem {
        // Обрезаем запрос для отображения
        let max_length = 100;
        let display_text = if request.len() > max_length {
            format!("{}...", &request[..max_length])
        } else {
            request.to_string()
        };

        ListItem::new(display_text).style(
            Style::new()
                .fg(TEXT_FG_COLOR)
                .add_modifier(Modifier::BOLD),
        )
    }

    /// Renders the detailed requests panel
    fn draw_detailed_requests(&self, params: DrawDetailedRequestsParams<'_, '_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(params.area);

        // IP Table
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
                .bg(Color::Rgb(80, 80, 80))
                .add_modifier(Modifier::BOLD),
        );

        params.frame.render_stateful_widget(
            Table::new(
                params.ip_rows,
                [
                    Constraint::Length(15), // IP
                    Constraint::Length(10), // Requests
                    Constraint::Min(20),    // Last Update
                ],
            )
            .header(ip_header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Top IPs"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[0],
            params.ip_table_state,
        );

        // Request List
        let request_header = if let Some(ip) = &params.selected_ip {
            format!("Requests for IP: {}", ip)
        } else {
            "Select an IP to view requests".to_string()
        };

        params.frame.render_stateful_widget(
            List::new(params.request_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title(request_header),
                )
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            params.request_list_state,
        );
    }

    fn on_left(&mut self) {
        // Устанавливаем выделение для IP таблицы
        if self.ip_table_state.selected().is_none() {
            self.ip_table_state.select(Some(0));
        }
        // Очищаем выделение списка запросов
        self.request_list_state.select(None);
    }

    fn on_right(&mut self) {
        let db = &*GLOBAL_DB;
        let top_ips = db.get_top_ips(self.top_n);
        if !top_ips.is_empty() {
            // Если IP не выбран, выбираем первый
            if self.ip_table_state.selected().is_none() {
                self.ip_table_state.select(Some(0));
            }
            // Устанавливаем выделение для списка запросов
            self.request_list_state.select(Some(0));
        }
    }
}

impl Default for DetailedTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for DetailedTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let db = &*GLOBAL_DB;
        let top_ips = db.get_top_ips(self.top_n);

        // Формируем строки для IP таблицы
        let ip_rows: Vec<Row> = top_ips
            .iter()
            .enumerate()
            .map(|(i, (ip, count))| {
                let is_active = self.ip_table_state.selected() == Some(i);
                self.format_ip_item(ip, *count, is_active)
            })
            .collect();

        // Получаем выбранный IP
        let selected_ip = if let Some(ip_index) = self.ip_table_state.selected() {
            if let Some((ip, _)) = top_ips.get(ip_index) {
                Some(ip.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Формируем список запросов для выбранного IP
        let request_items: Vec<ListItem> = if let Some(ip) = &selected_ip {
            let records = db.find_by_ip(ip);
            records
                .iter()
                .enumerate()
                .map(|(i, record)| self.format_request_item(&record.log_line, i))
                .collect()
        } else {
            vec![]
        };

        // Клонируем состояния для избежания конфликта заимствований
        let mut ip_table_state = self.ip_table_state.clone();
        let mut request_list_state = self.request_list_state.clone();

        let params = DrawDetailedRequestsParams {
            frame,
            area,
            ip_rows,
            request_items,
            selected_ip,
            ip_table_state: &mut ip_table_state,
            request_list_state: &mut request_list_state,
        };

        self.draw_detailed_requests(params);

        // Обновляем состояния
        self.ip_table_state = ip_table_state;
        self.request_list_state = request_list_state;
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
                // Определяем, какая панель активна
                if self.request_list_state.selected().is_some() {
                    // Список запросов активен
                    if let Some(selected) = self.request_list_state.selected() {
                        if selected > 0 {
                            self.request_list_state.select(Some(selected - 1));
                        }
                    }
                } else {
                    // IP таблица активна
                    if let Some(selected) = self.ip_table_state.selected() {
                        if selected > 0 {
                            self.ip_table_state.select(Some(selected - 1));
                        }
                    }
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                let db = &*GLOBAL_DB;
                
                if self.request_list_state.selected().is_some() {
                    // Список запросов активен
                    if let Some(ip_index) = self.ip_table_state.selected() {
                        let top_ips = db.get_top_ips(self.top_n);
                        if let Some((ip, _)) = top_ips.get(ip_index) {
                            let records = db.find_by_ip(ip);
                            if let Some(selected) = self.request_list_state.selected() {
                                if selected < records.len().saturating_sub(1) {
                                    self.request_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                    }
                } else {
                    // IP таблица активна
                    let top_ips = db.get_top_ips(self.top_n);
                    if let Some(selected) = self.ip_table_state.selected() {
                        if selected < top_ips.len().saturating_sub(1) {
                            self.ip_table_state.select(Some(selected + 1));
                        }
                    }
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
