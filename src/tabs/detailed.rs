use crate::log_data::{LogData, LogEntry};
use crate::tui_manager::{TuiManager, PANEL_TITLE_STYLE, SELECTED_ITEM_STYLE, TEXT_FG_COLOR};
use arboard::Clipboard;
use chrono::{Local, TimeZone};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};
use std::time::SystemTime;
use textwrap::wrap;

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
    tui_manager: TuiManager,
    ip_table_state: TableState,
    request_list_state: ListState,
    top_n: usize,
}

impl DetailedTab {
    pub fn new() -> Self {
        let mut instance = Self {
            tui_manager: TuiManager::new(),
            ip_table_state: TableState::default(),
            request_list_state: ListState::default(),
            top_n: 10,
        };

        // Инициализируем выделение для IP таблицы
        instance.ip_table_state.select(Some(0));

        instance
    }

    pub fn copy_selected_to_clipboard(&self, log_data: &LogData) -> Option<String> {
        // Если выбран IP
        if let Some(ip_index) = self.ip_table_state.selected() {
            let top_ips = log_data.get_top_n(self.top_n).0;
            if let Some((ip, _)) = top_ips.get(ip_index) {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(ip).is_ok() {
                        let message = "Copied IP to clipboard".to_string();
                        return Some(message);
                    }
                }
            }
        }

        // Если выбран запрос
        if let Some(request_index) = self.request_list_state.selected() {
            if let Some(ip_index) = self.ip_table_state.selected() {
                let top_ips = log_data.get_top_n(self.top_n).0;
                if let Some((ip, _)) = top_ips.get(ip_index) {
                    let last_requests = log_data.get_last_requests(ip);
                    if let Some(request) = last_requests.get(request_index) {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if clipboard.set_text(request).is_ok() {
                                // Обрезаем текст для модального окна
                                let display_text = if request.len() > 80 {
                                    format!("{}...", &request[..80])
                                } else {
                                    request.to_string()
                                };
                                let message =
                                    format!("Copied request to clipboard:\n{}", display_text);
                                eprintln!("Debug: Modal message = '{}'", message);
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
    fn format_ip_item(&self, ip: &str, entry: &LogEntry, _is_active: bool) -> Row {
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
            Cell::from(entry.count.to_string()).style(Style::new().fg(Color::Rgb(0, 255, 255))), // Requests - голубой
            Cell::from(last_update_str).style(Style::new().fg(Color::Rgb(255, 182, 193))), // Last Update - розовый
        ])
    }

    /// Renders the detailed requests panel
    fn draw_detailed_requests(&self, params: DrawDetailedRequestsParams<'_, '_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(params.area);

        // Создаем заголовок для IP таблицы
        let ip_header = Row::new(vec![
            Cell::from("IP").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Requests").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Last Update").style(
                Style::new()
                    .fg(Color::Rgb(255, 182, 193))
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(80, 80, 80)) // Серый фон для заголовка
                .add_modifier(Modifier::BOLD),
        );

        let mut request_items_with_header = vec![];
        let has_ip_header = params.selected_ip.is_some();
        if let Some(ref ip) = params.selected_ip {
            request_items_with_header
                .push(ListItem::new(format!("Requests for IP: {}", ip)).style(PANEL_TITLE_STYLE));
        }
        request_items_with_header.extend(params.request_items);

        // Корректируем выделение для IP таблицы
        let ip_selected = params.ip_table_state.selected();
        let mut adjusted_ip_state = TableState::default();
        if let Some(idx) = ip_selected {
            adjusted_ip_state.select(Some(idx));
        }

        // Корректируем выделение для списка запросов, учитывая заголовок
        let request_selected = params
            .request_list_state
            .selected()
            .map(|idx| idx + if has_ip_header { 1 } else { 0 });
        let mut adjusted_request_state = ListState::default();
        if let Some(idx) = request_selected {
            adjusted_request_state.select(Some(idx));
        }

        // Draw IP table
        params.frame.render_stateful_widget(
            Table::new(
                params.ip_rows.clone(),
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
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("IP List")
                    .title_style(
                        Style::new()
                            .fg(Color::Rgb(144, 238, 144))
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[0],
            &mut adjusted_ip_state,
        );

        // Draw request list
        params.frame.render_stateful_widget(
            List::new(request_items_with_header.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("Request Details")
                        .title_style(
                            Style::new()
                                .fg(Color::Rgb(144, 238, 144))
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut adjusted_request_state,
        );

        // Draw scrollbars
        self.tui_manager.draw_scrollbar(
            params.ip_rows.len(),
            adjusted_ip_state.selected().unwrap_or(0),
            params.frame,
            chunks[0],
        );
        self.tui_manager.draw_scrollbar(
            request_items_with_header.len(),
            adjusted_request_state.selected().unwrap_or(0),
            params.frame,
            chunks[1],
        );

        // Обновляем оригинальные состояния
        if let Some(idx) = adjusted_ip_state.selected() {
            params.ip_table_state.select(Some(idx));
        } else {
            params.ip_table_state.select(None);
        }
        if let Some(idx) = adjusted_request_state.selected() {
            let offset = if params.selected_ip.is_some() { 1 } else { 0 };
            if idx >= offset {
                params.request_list_state.select(Some(idx - offset));
            } else {
                params.request_list_state.select(None);
            }
        }
    }

    fn on_left(&mut self) {
        if self.request_list_state.selected().is_some() {
            self.request_list_state.select(None);
        }
    }

    fn on_right(&mut self, log_data: &LogData) {
        if self.ip_table_state.selected().is_some() {
            // Проверяем, есть ли IP в данных
            if let Some(selected_ip_idx) = self.ip_table_state.selected() {
                let top_ips = log_data.get_top_n(self.top_n).0;
                if let Some((_ip, _)) = top_ips.get(selected_ip_idx) {
                    // Если IP существует в данных, то переключаемся на правую панель
                    self.request_list_state.select(Some(0));
                }
            }
        }
    }
}

impl Default for DetailedTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for DetailedTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let mut top_ips = log_data.get_top_n(self.top_n).0;
        top_ips.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        let ip_items: Vec<Row> = top_ips
            .iter()
            .map(|(ip, entry)| {
                self.format_ip_item(ip, entry, self.ip_table_state.selected().is_some())
            })
            .collect();

        let selected_ip = self
            .ip_table_state
            .selected()
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

        // Клонируем состояния для избежания конфликта заимствований
        let mut ip_table_state_clone = self.ip_table_state.clone();
        let mut request_list_state_clone = self.request_list_state.clone();

        self.draw_detailed_requests(DrawDetailedRequestsParams {
            frame,
            area,
            ip_rows: ip_items,
            request_items,
            selected_ip,
            ip_table_state: &mut ip_table_state_clone,
            request_list_state: &mut request_list_state_clone,
        });

        self.ip_table_state = ip_table_state_clone;
        self.request_list_state = request_list_state_clone;
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, log_data: &LogData) -> bool {
        match key.code {
            crossterm::event::KeyCode::Up => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select_previous();
                } else {
                    if let Some(selected) = self.ip_table_state.selected() {
                        if selected > 0 {
                            self.ip_table_state.select(Some(selected - 1));
                        }
                    }
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                if self.request_list_state.selected().is_some() {
                    self.request_list_state.select_next();
                } else {
                    if let Some(selected) = self.ip_table_state.selected() {
                        // Получаем количество IP для определения максимального индекса
                        let top_ips = log_data.get_top_n(self.top_n).0;
                        if selected < top_ips.len().saturating_sub(1) {
                            self.ip_table_state.select(Some(selected + 1));
                        }
                    }
                }
                true
            }
            crossterm::event::KeyCode::Left => {
                self.on_left();
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
