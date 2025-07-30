use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::HEADER_STYLE;
use ratatui::{
    layout::{Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

pub struct SparklineTab;

impl SparklineTab {
    pub fn new() -> Self {
        Self
    }

    fn draw_sparkline<'a>(&self, data: &'a [u64], title: &'a str) -> Sparkline<'a> {
        // Создаем блок с закругленными углами и стильным заголовком
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Center);

        // Настраиваем стиль графика
        let style = Style::new()
            .fg(Color::Rgb(0, 191, 255)) // Яркий голубой цвет для линии
            .bg(Color::Rgb(28, 28, 28)); // Темный фон

        Sparkline::default()
            .block(block)
            .data(data)
            .direction(ratatui::widgets::RenderDirection::RightToLeft)
            .style(style)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS) // Используем 9 уровней для более плавного градиента
            .max(
                *data
                    .iter()
                    .max()
                    .unwrap_or(&1),
            )
    }

    fn draw_requests_sparkline(&self, frame: &mut Frame, area: Rect) {
        let db = &*GLOBAL_DB;
        let time_series_data = db.get_time_series_data(3600); // 1 hour intervals

        let mut data: Vec<u64> = time_series_data.iter().map(|(_, count)| *count as u64).collect();
        if data.len() > area.width as usize {
            data.truncate(area.width as usize);
        }

        if data.is_empty() {
            frame.render_widget(
                Paragraph::new("No data available for sparkline")
                    .style(HEADER_STYLE)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                            .title("📈 Requests Sparkline"),
                    ),
                area,
            );
            return;
        }

        let total_requests: u64 = data.iter().sum();
        let avg_requests = if !data.is_empty() {
            total_requests / data.len() as u64
        } else {
            0
        };
        let max_requests = *data.iter().max().unwrap_or(&0);

        let title = format!(
            "📈 Requests Sparkline (Total: {}, Avg: {}, Max: {})",
            total_requests, avg_requests, max_requests
        );

        let sparkline = self.draw_sparkline(&data, &title);
        frame.render_widget(sparkline, area);
    }
}

impl Default for SparklineTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for SparklineTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.draw_requests_sparkline(frame, area);
    }

    fn handle_input(&mut self, _key: crossterm::event::KeyEvent) -> bool {
        // Sparkline tab doesn't handle input
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
