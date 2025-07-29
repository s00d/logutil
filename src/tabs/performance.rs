use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::HEADER_STYLE;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct PerformanceTab {
    table_state: TableState,
}

impl PerformanceTab {
    pub fn new() -> Self {
        let mut instance = Self {
            table_state: TableState::default(),
        };

        // Инициализируем выделение
        instance.table_state.select(Some(0));

        instance
    }

    fn draw_performance_tab(&mut self, frame: &mut Frame, area: Rect) {
        let db = GLOBAL_DB.read().unwrap();
        let (avg_time, max_time, min_time) = db.get_response_time_stats();
        let slow_requests = db.get_slow_requests_with_limit(1.0, 10);
        let requests_per_second = db.get_requests_per_second();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Performance summary with RPS
        let summary_text = format!(
            "Avg Response: {:.2}s | Max: {:.2}s | Min: {:.2}s | RPS: {:.1}",
            avg_time, max_time, min_time, requests_per_second
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
        let items: Vec<Row> = slow_requests
            .iter()
            .map(|(ip, time)| {
                Row::new(vec![
                    Cell::from(ip.to_string()).style(
                        Style::new()
                            .fg(Color::Rgb(255, 255, 0))
                            .add_modifier(Modifier::BOLD),
                    ), // IP - желтый, жирный
                    Cell::from(format!("{:.2}s", time))
                        .style(Style::new().fg(Color::Rgb(0, 255, 255))), // Time - голубой
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("IP").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Time").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
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
                    Constraint::Length(15), // IP
                    Constraint::Length(10), // Time
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 0)))
                    .title("Slow Requests (>1s)"),
            )
            .row_highlight_style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[1],
            &mut self.table_state,
        );
    }
}

impl Default for PerformanceTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for PerformanceTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.draw_performance_tab(frame, area);
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
                let db = GLOBAL_DB.read().unwrap();
                let slow_requests = db.get_slow_requests_with_limit(1.0, 10);
                if let Some(selected) = self.table_state.selected() {
                    if selected < slow_requests.len().saturating_sub(1) {
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
