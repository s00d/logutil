use crate::log_data::LogData;
use crate::tui_manager::HEADER_STYLE;
use chrono::{TimeZone, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

pub struct SparklineTab;

impl SparklineTab {
    pub fn new() -> Self {
        Self
    }

    /// Gets bounds for the sparkline graph
    fn get_sparkline_bounds(
        &self,
        data: &[u64],
        sorted_data: &[(i64, u64)],
    ) -> (u64, u64, i64, i64) {
        if data.is_empty() {
            return (0, 0, 0, 0);
        }

        let min_value = *data
            .iter()
            .min()
            .expect("Data should not be empty after check");
        let max_value = *data
            .iter()
            .max()
            .expect("Data should not be empty after check");

        let start_time = sorted_data
            .first()
            .map(|&(timestamp, _)| timestamp)
            .unwrap_or(0);
        let end_time = sorted_data
            .last()
            .map(|&(timestamp, _)| timestamp)
            .unwrap_or(0);

        (min_value, max_value, start_time, end_time)
    }

    fn draw_sparkline<'a>(&self, data: &'a [u64], title: &'a str) -> Sparkline<'a> {
        // Разделяем заголовок на основную часть и статистику
        let parts: Vec<&str> = title.split(" (").collect();
        let main_title = parts[0];
        let stats = if parts.len() > 1 {
            format!("({}", parts[1])
        } else {
            String::new()
        };

        // Создаем блок с закругленными углами и стильным заголовком
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
            .title(format!("{} {}", main_title, stats))
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
                    .expect("Data should not be empty after check"),
            )
    }

    fn draw_requests_sparkline(&self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let mut sorted_data: Vec<_> = log_data
            .requests_per_interval
            .iter()
            .map(|(&k, &v)| (k, v as u64))
            .collect();
        sorted_data.sort_by_key(|&(k, _)| k);
        sorted_data.reverse();

        let mut data: Vec<u64> = sorted_data.iter().map(|&(_, v)| v).collect();
        if data.len() > area.width as usize {
            data.truncate(area.width as usize);
        }

        if data.is_empty() {
            // Показываем сообщение о том, что данных нет
            frame.render_widget(
                Paragraph::new("No request data available")
                    .style(HEADER_STYLE)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                            .title("Requests Timeline"),
                    ),
                area,
            );
            return;
        }

        let (min_value, max_value, start_time, end_time) =
            self.get_sparkline_bounds(&data, &sorted_data);

        // Разделяем область на график и текстовую информацию
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Заголовок с метриками
                    Constraint::Min(0),    // График
                    Constraint::Length(8), // Детальная статистика
                ]
                .as_ref(),
            )
            .split(area);

        // Заголовок с основными метриками
        let header_text = format!(
            "📊 Requests Timeline | Min: {} | Max: {} | Total: {} | Range: {} - {}",
            min_value,
            max_value,
            data.iter().sum::<u64>(),
            Utc.timestamp_opt(start_time, 0).unwrap().format("%H:%M:%S"),
            Utc.timestamp_opt(end_time, 0).unwrap().format("%H:%M:%S")
        );

        frame.render_widget(
            Paragraph::new(header_text).style(HEADER_STYLE).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("📈 Real-time Activity"),
            ),
            chunks[0],
        );

        // График sparkline
        let sparkline = self.draw_sparkline(&data, "Requests Timeline");
        frame.render_widget(sparkline, chunks[1]);

        // Детальная статистика
        let stats_text =
            self.generate_detailed_stats(&data, &sorted_data, log_data, min_value, max_value);
        frame.render_widget(
            Paragraph::new(stats_text)
                .style(Style::new().fg(Color::Rgb(200, 200, 200)))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("📋 Detailed Statistics"),
                ),
            chunks[2],
        );
    }

    fn generate_detailed_stats(
        &self,
        data: &[u64],
        sorted_data: &[(i64, u64)],
        log_data: &LogData,
        min_value: u64,
        max_value: u64,
    ) -> String {
        let total_requests: u64 = data.iter().sum();
        let avg_requests = if !data.is_empty() {
            total_requests / data.len() as u64
        } else {
            0
        };
        let peak_index = data
            .iter()
            .enumerate()
            .max_by_key(|(_, &val)| val)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let peak_time = if !sorted_data.is_empty() && peak_index < sorted_data.len() {
            let timestamp = sorted_data[peak_index].0;
            Utc.timestamp_opt(timestamp, 0)
                .unwrap()
                .format("%H:%M:%S")
                .to_string()
        } else {
            "N/A".to_string()
        };

        // Анализ трендов
        let trend_analysis = if data.len() >= 2 {
            let recent_avg: u64 =
                data.iter().take(data.len() / 2).sum::<u64>() / (data.len() / 2) as u64;
            let older_avg: u64 =
                data.iter().skip(data.len() / 2).sum::<u64>() / (data.len() / 2) as u64;

            match recent_avg.cmp(&older_avg) {
                std::cmp::Ordering::Greater => "📈 Increasing trend",
                std::cmp::Ordering::Less => "📉 Decreasing trend",
                std::cmp::Ordering::Equal => "➡️ Stable activity",
            }
        } else {
            "➡️ Insufficient data for trend analysis"
        };

        // Дополнительная информация из LogData
        let requests_per_second = log_data.get_requests_per_second();
        let (top_ips, _) = log_data.get_top_n(5);
        let top_ip_info = if !top_ips.is_empty() {
            format!("Top IP: {} ({} requests)", top_ips[0].0, top_ips[0].1.count)
        } else {
            "No IP data available".to_string()
        };

        format!(
            "📊 Activity Summary:\n\
             • Average requests per interval: {}\n\
             • Peak activity: {} requests at {}\n\
             • Current RPS: {:.1}\n\
             • {} intervals analyzed\n\
             • {}\n\
             • {}\n\
             \n\
             💡 Trend Analysis:\n\
             • {}\n\
             • Data spans {} time intervals\n\
             • Min-Max range: {} requests",
            avg_requests,
            data.iter().max().unwrap_or(&0),
            peak_time,
            requests_per_second,
            data.len(),
            top_ip_info,
            if total_requests > 1000 {
                "🔥 High traffic detected"
            } else {
                "⚡ Normal activity"
            },
            trend_analysis,
            data.len(),
            max_value - min_value
        )
    }
}

impl Default for SparklineTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for SparklineTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_requests_sparkline(frame, area, log_data);
    }

    fn handle_input(&mut self, _key: crossterm::event::KeyEvent, _log_data: &LogData) -> bool {
        // Sparkline не требует специальной обработки ввода
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
