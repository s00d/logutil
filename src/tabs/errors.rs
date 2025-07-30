use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct ErrorsTab {
    table_state: TableState,
}

impl ErrorsTab {
    pub fn new() -> Self {
        let mut instance = Self {
            table_state: TableState::default(),
        };

        // Инициализируем выделение
        instance.table_state.select(Some(0));

        instance
    }

    fn draw_errors_tab(&self, frame: &mut Frame, area: Rect) {
        let db = &*GLOBAL_DB;
        let (error_codes_count, error_urls_count, error_ips_count) = db.get_error_stats();
        let top_errors = db.get_top_status_codes(10);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Error summary
        let summary_text = format!(
            "Error Codes: {} | Error URLs: {} | Error IPs: {}",
            error_codes_count, error_urls_count, error_ips_count
        );

        frame.render_widget(
            Paragraph::new(summary_text).style(HEADER_STYLE).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(255, 0, 255))) // Magenta for errors
                    .title("Error Analysis"),
            ),
            chunks[0],
        );

        // Error codes list
        let items: Vec<Row> = top_errors
            .iter()
            .map(|(code, count)| {
                let status_code: i32 = code.parse().unwrap_or(0);
                let error_type = match status_code {
                    400..=499 => "Client Error",
                    500..=599 => "Server Error",
                    _ => "Other Error",
                };
                Row::new(vec![
                    Cell::from(code.to_string()).style(
                        Style::new()
                            .fg(Color::Rgb(255, 255, 0))
                            .add_modifier(Modifier::BOLD),
                    ), // Code - желтый, жирный
                    Cell::from(error_type).style(Style::new().fg(Color::Rgb(0, 255, 255))), // Type - голубой
                    Cell::from(count.to_string()).style(Style::new().fg(Color::Rgb(255, 182, 193))), // Count - розовый
                    Cell::from("occurrences").style(Style::new().fg(Color::Rgb(144, 238, 144))), // Text - зеленый
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Code").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Type").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Count").style(
                Style::new()
                    .fg(Color::Rgb(255, 182, 193))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Description").style(
                Style::new()
                    .fg(Color::Rgb(144, 238, 144))
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
                items,
                [
                    Constraint::Length(10), // Code
                    Constraint::Length(15), // Type
                    Constraint::Length(10), // Count
                    Constraint::Min(20),    // Description
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(255, 0, 255)))
                    .title("Top Error Codes"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut self.table_state.clone(),
        );
    }
}

impl Default for ErrorsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for ErrorsTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.draw_errors_tab(frame, area);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            crossterm::event::KeyCode::Up => {
                if let Some(selected) = self.table_state.selected() {
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                let db = &*GLOBAL_DB;
                let top_errors = db.get_top_status_codes(10);
                if let Some(selected) = self.table_state.selected() {
                    if selected < top_errors.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
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
