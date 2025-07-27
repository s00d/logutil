use crate::log_data::LogData;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct ErrorsTab {
    list_state: ListState,
}

impl ErrorsTab {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    fn draw_errors_tab(&self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (error_codes_count, error_urls_count, error_ips_count) = log_data.get_error_summary();
        let top_errors = log_data.get_top_error_codes();

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
        let items: Vec<ListItem> = top_errors
            .iter()
            .map(|(code, count)| {
                let error_type = match code {
                    400..=499 => "Client Error",
                    500..=599 => "Server Error",
                    _ => "Other Error",
                };
                ListItem::new(format!(
                    "{:<10} │ {:<15} │ {:<10} │ {}",
                    code, error_type, count, "occurrences"
                ))
                .style(Style::new().fg(Color::Rgb(255, 0, 255)))
            })
            .collect();

        frame.render_stateful_widget(
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(255, 0, 255)))
                        .title("Error Codes"),
                )
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut self.list_state.clone(),
        );
    }
}

impl Default for ErrorsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for ErrorsTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_errors_tab(frame, area, log_data);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, _log_data: &LogData) -> bool {
        match key.code {
            crossterm::event::KeyCode::Up => {
                self.list_state.select_previous();
                true
            }
            crossterm::event::KeyCode::Down => {
                self.list_state.select_next();
                true
            }
            _ => false,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
