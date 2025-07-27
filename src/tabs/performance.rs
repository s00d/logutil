use crate::log_data::LogData;
use crate::tui_manager::HEADER_STYLE;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct PerformanceTab {
    list_state: ListState,
}

impl PerformanceTab {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    fn draw_performance_tab(&self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (avg_time, max_time, min_time, total_size) = log_data.get_performance_summary();
        let slow_requests = log_data.get_slow_requests(); // Requests slower than 1 second
        let requests_per_second = log_data.get_requests_per_second();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Performance summary with RPS
        let summary_text = format!(
            "Avg Response: {:.2}s | Max: {:.2}s | Min: {:.2}s | Total Size: {} bytes | RPS: {:.1}",
            avg_time, max_time, min_time, total_size, requests_per_second
        );

        frame.render_widget(
            Paragraph::new(summary_text).style(HEADER_STYLE).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 0))) // Green for performance
                    .title("Performance Metrics"),
            ),
            chunks[0],
        );

        // Slow requests list with detailed tracking
        let items: Vec<ListItem> = slow_requests
            .iter()
            .take(10)
            .map(|(ip, time)| {
                ListItem::new(format!("{:<15} │ {:.2}s", ip, time))
                    .style(Style::new().fg(Color::Rgb(255, 165, 0)))
            })
            .collect();

        frame.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(255, 165, 0)))
                    .title("Slow Requests (Top 10)"),
            ),
            area,
        );
    }
}

impl Default for PerformanceTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for PerformanceTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_performance_tab(frame, area, log_data);
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
