use crate::log_data::LogData;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use chrono::{Datelike, TimeZone, Timelike, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct HeatmapTab {
    hourly_table_state: TableState,
    daily_table_state: TableState,
    weekly_table_state: TableState,
    active_panel: usize, // 0 = hourly, 1 = daily, 2 = weekly
}

impl HeatmapTab {
    pub fn new() -> Self {
        let mut instance = Self {
            hourly_table_state: TableState::default(),
            daily_table_state: TableState::default(),
            weekly_table_state: TableState::default(),
            active_panel: 0,
        };

        // Инициализируем выделение для всех панелей
        instance.hourly_table_state.select(Some(0));
        instance.daily_table_state.select(Some(0));
        instance.weekly_table_state.select(Some(0));

        instance
    }

    fn draw_heatmap(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        // Разделяем область на три равные панели
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ]
                .as_ref(),
            )
            .split(area);

        // Рисуем все три панели одновременно
        self.draw_hourly_view(frame, chunks[0], log_data);
        self.draw_daily_view(frame, chunks[1], log_data);
        self.draw_weekly_view(frame, chunks[2], log_data);
    }

    fn draw_hourly_view(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let hourly_data = self.generate_hourly_data(log_data);

        if hourly_data.is_empty() {
            frame.render_widget(
                Paragraph::new("No hourly data available")
                    .style(HEADER_STYLE)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(if self.active_panel == 0 {
                                Style::new().fg(Color::Rgb(255, 255, 255))
                            } else {
                                Style::new().fg(Color::Rgb(144, 238, 144))
                            })
                            .title("🕐 Hourly Activity"),
                    ),
                area,
            );
            return;
        }

        let items: Vec<Row> = hourly_data
            .iter()
            .map(|(hour, count, intensity)| {
                let bar = self.generate_intensity_bar(*intensity);
                let time_str = format!("{:02}:00", hour);
                Row::new(vec![
                    Cell::from(time_str),
                    Cell::from(bar),
                    Cell::from(count.to_string()),
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Time").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Bar").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Count").style(
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

        frame.render_stateful_widget(
            Table::new(
                items,
                [
                    Constraint::Length(20), // Time
                    Constraint::Length(20), // Bar
                    Constraint::Length(10), // Count
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(if self.active_panel == 0 {
                        Style::new().fg(Color::Rgb(255, 255, 255))
                    } else {
                        Style::new().fg(Color::Rgb(144, 238, 144))
                    })
                    .title("🕐 Hourly Request Distribution"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            area,
            &mut self.hourly_table_state,
        );
    }

    fn draw_daily_view(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let daily_data = self.generate_daily_data(log_data);

        if daily_data.is_empty() {
            frame.render_widget(
                Paragraph::new("No daily data available")
                    .style(HEADER_STYLE)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(if self.active_panel == 1 {
                                Style::new().fg(Color::Rgb(255, 255, 255))
                            } else {
                                Style::new().fg(Color::Rgb(144, 238, 144))
                            })
                            .title("📅 Daily Activity"),
                    ),
                area,
            );
            return;
        }

        let items: Vec<Row> = daily_data
            .iter()
            .map(|(day, count, intensity)| {
                let bar = self.generate_intensity_bar(*intensity);
                Row::new(vec![
                    Cell::from(day.to_string()),
                    Cell::from(bar),
                    Cell::from(count.to_string()),
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Date").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Bar").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Count").style(
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

        frame.render_stateful_widget(
            Table::new(
                items,
                [
                    Constraint::Length(20), // Time - увеличиваем для даты
                    Constraint::Length(20), // Bar
                    Constraint::Length(10), // Count
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(if self.active_panel == 1 {
                        Style::new().fg(Color::Rgb(255, 255, 255))
                    } else {
                        Style::new().fg(Color::Rgb(144, 238, 144))
                    })
                    .title("📅 Daily Request Distribution"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            area,
            &mut self.daily_table_state,
        );
    }

    fn draw_weekly_view(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let weekly_data = self.generate_weekly_data(log_data);

        if weekly_data.is_empty() {
            frame.render_widget(
                Paragraph::new("No weekly data available")
                    .style(HEADER_STYLE)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(if self.active_panel == 2 {
                                Style::new().fg(Color::Rgb(255, 255, 255))
                            } else {
                                Style::new().fg(Color::Rgb(144, 238, 144))
                            })
                            .title("📊 Weekly Activity"),
                    ),
                area,
            );
            return;
        }

        let items: Vec<Row> = weekly_data
            .iter()
            .map(|(week, count, intensity)| {
                let bar = self.generate_intensity_bar(*intensity);
                Row::new(vec![
                    Cell::from(week.to_string()),
                    Cell::from(bar),
                    Cell::from(count.to_string()),
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Week").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Bar").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Count").style(
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

        frame.render_stateful_widget(
            Table::new(
                items,
                [
                    Constraint::Length(20), // Time - увеличиваем для недели
                    Constraint::Length(20), // Bar
                    Constraint::Length(10), // Count
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(if self.active_panel == 2 {
                        Style::new().fg(Color::Rgb(255, 255, 255))
                    } else {
                        Style::new().fg(Color::Rgb(144, 238, 144))
                    })
                    .title("📊 Weekly Request Distribution"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            area,
            &mut self.weekly_table_state,
        );
    }

    fn generate_hourly_data(&self, log_data: &LogData) -> Vec<(u32, u64, f64)> {
        let mut hourly_counts: std::collections::HashMap<u32, u64> =
            std::collections::HashMap::new();

        for (&timestamp, &count) in &log_data.requests_per_interval {
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            let hour = datetime.hour();
            *hourly_counts.entry(hour).or_insert(0) += count as u64;
        }

        let max_count = *hourly_counts.values().max().unwrap_or(&1);

        let mut result: Vec<_> = hourly_counts
            .into_iter()
            .map(|(hour, count)| {
                let intensity = count as f64 / max_count as f64;
                (hour, count, intensity)
            })
            .collect();

        result.sort_by_key(|&(hour, _, _)| hour);
        result
    }

    fn generate_daily_data(&self, log_data: &LogData) -> Vec<(String, u64, f64)> {
        let mut daily_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for (&timestamp, &count) in &log_data.requests_per_interval {
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            let date_str = datetime.format("%Y-%m-%d").to_string();
            *daily_counts.entry(date_str).or_insert(0) += count as u64;
        }

        let max_count = *daily_counts.values().max().unwrap_or(&1);

        let mut result: Vec<_> = daily_counts
            .into_iter()
            .map(|(date, count)| {
                let intensity = count as f64 / max_count as f64;
                (date, count, intensity)
            })
            .collect();

        result.sort_by_key(|(date, _, _)| date.clone());
        result
    }

    fn generate_weekly_data(&self, log_data: &LogData) -> Vec<(String, u64, f64)> {
        let mut weekly_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for (&timestamp, &count) in &log_data.requests_per_interval {
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            let week_str = format!("Week {} of {}", datetime.iso_week().week(), datetime.year());
            *weekly_counts.entry(week_str).or_insert(0) += count as u64;
        }

        let max_count = *weekly_counts.values().max().unwrap_or(&1);

        let mut result: Vec<_> = weekly_counts
            .into_iter()
            .map(|(week, count)| {
                let intensity = count as f64 / max_count as f64;
                (week, count, intensity)
            })
            .collect();

        result.sort_by_key(|(week, _, _)| week.clone());
        result
    }

    fn generate_intensity_bar(&self, intensity: f64) -> String {
        let bar_length = 20;
        let filled_length = (intensity * bar_length as f64) as usize;
        let empty_length = bar_length - filled_length;

        let filled_char = match intensity {
            i if i > 0.8 => "█",
            i if i > 0.6 => "▓",
            i if i > 0.4 => "▒",
            i if i > 0.2 => "░",
            _ => " ",
        };

        let empty_char = " ";

        format!(
            "{}{}",
            filled_char.repeat(filled_length),
            empty_char.repeat(empty_length)
        )
    }
}

impl Default for HeatmapTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for HeatmapTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_heatmap(frame, area, log_data);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, log_data: &LogData) -> bool {
        match key.code {
            crossterm::event::KeyCode::Up => {
                match self.active_panel {
                    0 => {
                        if let Some(selected) = self.hourly_table_state.selected() {
                            if selected > 0 {
                                self.hourly_table_state.select(Some(selected - 1));
                            }
                        }
                    }
                    1 => {
                        if let Some(selected) = self.daily_table_state.selected() {
                            if selected > 0 {
                                self.daily_table_state.select(Some(selected - 1));
                            }
                        }
                    }
                    2 => {
                        if let Some(selected) = self.weekly_table_state.selected() {
                            if selected > 0 {
                                self.weekly_table_state.select(Some(selected - 1));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                match self.active_panel {
                    0 => {
                        if let Some(selected) = self.hourly_table_state.selected() {
                            // Получаем количество часов для определения максимального индекса
                            let hourly_data = self.generate_hourly_data(log_data);
                            if selected < hourly_data.len().saturating_sub(1) {
                                self.hourly_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    1 => {
                        if let Some(selected) = self.daily_table_state.selected() {
                            // Получаем количество дней для определения максимального индекса
                            let daily_data = self.generate_daily_data(log_data);
                            if selected < daily_data.len().saturating_sub(1) {
                                self.daily_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    2 => {
                        if let Some(selected) = self.weekly_table_state.selected() {
                            // Получаем количество недель для определения максимального индекса
                            let weekly_data = self.generate_weekly_data(log_data);
                            if selected < weekly_data.len().saturating_sub(1) {
                                self.weekly_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            crossterm::event::KeyCode::Left => {
                if self.active_panel > 0 {
                    self.active_panel -= 1;
                    // Устанавливаем выделение на первую строку для новой активной панели
                    match self.active_panel {
                        0 => {
                            self.hourly_table_state.select(Some(0));
                        }
                        1 => {
                            self.daily_table_state.select(Some(0));
                        }
                        2 => {
                            self.weekly_table_state.select(Some(0));
                        }
                        _ => {}
                    }
                }
                true
            }
            crossterm::event::KeyCode::Right => {
                if self.active_panel < 2 {
                    self.active_panel += 1;
                    // Устанавливаем выделение на первую строку для новой активной панели
                    match self.active_panel {
                        0 => {
                            self.hourly_table_state.select(Some(0));
                        }
                        1 => {
                            self.daily_table_state.select(Some(0));
                        }
                        2 => {
                            self.weekly_table_state.select(Some(0));
                        }
                        _ => {}
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
