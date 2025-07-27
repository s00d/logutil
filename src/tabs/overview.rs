use crate::log_data::{LogData, LogEntry};
use crate::tui_manager::SELECTED_ITEM_STYLE;
use arboard::Clipboard;
use chrono::{Local, TimeZone};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use std::time::SystemTime;

pub struct OverviewTab {
    top_ip_list_state: ListState,
    top_url_list_state: ListState,
    overview_panel: usize, // 0 - left panel (IP), 1 - right panel (URL)
    top_n: usize,
}

impl OverviewTab {
    pub fn new() -> Self {
        Self {
            top_ip_list_state: ListState::default(),
            top_url_list_state: ListState::default(),
            overview_panel: 0,
            top_n: 10,
        }
    }

    fn draw_overview(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let ip_items: Vec<ListItem> = top_ips
            .iter()
            .map(|(ip, entry)| {
                self.format_ip_item(ip, entry, self.top_ip_list_state.selected().is_some())
            })
            .collect();

        let url_items: Vec<(ListItem, &LogEntry)> = top_urls
            .into_iter()
            .map(|(url, entry)| {
                (
                    self.format_url_item(&url, entry, self.top_url_list_state.selected().is_some()),
                    entry,
                )
            })
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

        // Добавляем заголовки в начало списков
        let mut ip_items_with_header = vec![ListItem::new(format!(
            "{:<15} │ {:<12} │ {}",
            "IP", "Requests", "Last Update"
        ))
        .style(
            Style::new()
                .fg(Color::Rgb(0, 191, 255))
                .add_modifier(Modifier::BOLD),
        )];
        ip_items_with_header.extend(ip_items);

        let mut url_items_with_header: Vec<(ListItem, Option<&LogEntry>)> = vec![(
            ListItem::new(format!(
                "{:<25} │ {:<20} │ {:<10} │ {:<12} │ {}",
                "URL", "Type", "Domain", "Requests", "Last Update"
            ))
            .style(
                Style::new()
                    .fg(Color::Rgb(0, 191, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            None,
        )];
        url_items_with_header.extend(
            url_items
                .into_iter()
                .map(|(item, entry)| (item, Some(entry))),
        );

        // Корректируем выделение для IP списка, учитывая заголовок
        let ip_selected = self.top_ip_list_state.selected().map(|idx| idx + 1);
        let mut adjusted_ip_state = ListState::default();
        if let Some(idx) = ip_selected {
            adjusted_ip_state.select(Some(idx));
        }

        // Корректируем выделение для URL списка, учитывая заголовок
        let url_selected = self.top_url_list_state.selected().map(|idx| idx + 1);
        let mut adjusted_url_state = ListState::default();
        if let Some(idx) = url_selected {
            adjusted_url_state.select(Some(idx));
        }

        // Draw IP list
        frame.render_stateful_widget(
            List::new(ip_items_with_header.clone())
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
                .highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[0],
            &mut adjusted_ip_state,
        );

        // Draw URL list
        frame.render_stateful_widget(
            List::new(
                url_items_with_header
                    .iter()
                    .map(|(item, _)| item.clone())
                    .collect::<Vec<ListItem>>(),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("URL List")
                    .title_style(
                        Style::new()
                            .fg(Color::Rgb(144, 238, 144))
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[1],
            &mut adjusted_url_state,
        );

        // Draw scrollbars
        self.draw_scrollbar(
            ip_items_with_header.len(),
            adjusted_ip_state.selected().unwrap_or(0),
            frame,
            main_chunks[0],
        );
        self.draw_scrollbar(
            url_items_with_header.len(),
            adjusted_url_state.selected().unwrap_or(0),
            frame,
            main_chunks[1],
        );

        // Отображаем полный URL в нижней панели, если URL выбран
        if let Some(idx) = adjusted_url_state.selected() {
            if idx > 0 {
                // Пропускаем заголовок
                if let Some((_, Some(entry))) = url_items_with_header.get(idx) {
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

        // Обновляем оригинальные состояния
        if let Some(idx) = adjusted_ip_state.selected() {
            if idx > 0 {
                self.top_ip_list_state.select(Some(idx - 1));
            } else {
                self.top_ip_list_state.select(None);
            }
        }
        if let Some(idx) = adjusted_url_state.selected() {
            if idx > 0 {
                self.top_url_list_state.select(Some(idx - 1));
            } else {
                self.top_url_list_state.select(None);
            }
        }
    }

    // Методы из TuiManager
    fn format_ip_item<'a>(&self, ip: &str, entry: &LogEntry, is_active: bool) -> ListItem<'a> {
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
        let style = if is_active {
            Style::new()
                .fg(Color::Rgb(144, 238, 144))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::Rgb(169, 169, 169))
        };
        ListItem::new(format!(
            "{:<15} │ {:<12} │ {}",
            ip, entry.count, last_update_str
        ))
        .style(style)
    }

    fn format_url_item<'a>(&self, url: &str, entry: &LogEntry, is_active: bool) -> ListItem<'a> {
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
        let style = if is_active {
            Style::new()
                .fg(Color::Rgb(144, 238, 144))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::Rgb(169, 169, 169))
        };

        // Обрезаем URL если он слишком длинный
        let truncated_url = self.truncate_url(url, 25);

        ListItem::new(format!(
            "{:<25} │ {:<20} │ {:<10} │ {:<12} │ {}",
            truncated_url, entry.request_type, entry.request_domain, entry.count, last_update_str
        ))
        .style(style)
    }

    fn truncate_url(&self, url: &str, max_length: usize) -> String {
        if url.len() <= max_length {
            return url.to_string();
        }
        format!("{}...", &url[..max_length - 3])
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
                self.top_url_list_state.select(None);
                // Проверяем, есть ли IP в списке
                let (top_ips, _) = log_data.get_top_n(self.top_n);
                if !top_ips.is_empty() {
                    self.top_ip_list_state.select(Some(0));
                }
            }
        }
    }

    fn on_right(&mut self, log_data: &LogData) {
        if self.overview_panel < 1 {
            self.overview_panel += 1;
            if self.overview_panel == 1 {
                self.top_ip_list_state.select(None);
                // Проверяем, есть ли URL в списке
                let (_, top_urls) = log_data.get_top_n(self.top_n);
                if !top_urls.is_empty() {
                    self.top_url_list_state.select(Some(0));
                }
            }
        }
    }

    pub fn copy_selected_to_clipboard(&self, log_data: &LogData) -> Option<String> {
        let (top_ips, top_urls) = log_data.get_top_n(self.top_n);

        let (text_to_copy, message) = match self.overview_panel {
            0 => {
                if let Some(selected) = self.top_ip_list_state.selected() {
                    if selected > 0 {
                        if let Some((ip, entry)) = top_ips.get(selected - 1) {
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
                                format!("IP address copied: {}", ip),
                            )
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            1 => {
                if let Some(selected) = self.top_url_list_state.selected() {
                    if selected > 0 {
                        if let Some((url, entry)) = top_urls.get(selected - 1) {
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
                    0 => self.top_ip_list_state.select_previous(),
                    1 => self.top_url_list_state.select_previous(),
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                match self.overview_panel {
                    0 => self.top_ip_list_state.select_next(),
                    1 => self.top_url_list_state.select_next(),
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
