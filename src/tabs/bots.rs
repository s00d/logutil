use crate::log_data::LogData;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct BotsTab {
    table_state: TableState,
}

impl BotsTab {
    pub fn new() -> Self {
        let mut instance = Self {
            table_state: TableState::default(),
        };
        
        // Инициализируем выделение
        instance.table_state.select(Some(0));
        
        instance
    }

    fn draw_bots_tab(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (bot_ips_count, bot_types_count, bot_urls_count) = log_data.get_bot_summary();
        let top_bot_types = log_data.get_top_bot_types();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Bot summary
        let summary_text = format!(
            "Bot IPs: {} | Bot Types: {} | Bot URLs: {}",
            bot_ips_count, bot_types_count, bot_urls_count
        );

        frame.render_widget(
            Paragraph::new(summary_text).style(HEADER_STYLE).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 255))) // Cyan for bots
                    .title("Bot Analysis"),
            ),
            chunks[0],
        );

        // Bot types list
        let items: Vec<Row> = top_bot_types
            .iter()
            .map(|(bot_type, count)| {
                Row::new(vec![
                    Cell::from(bot_type.to_string()).style(Style::new().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)), // Type - желтый, жирный
                    Cell::from(count.to_string()).style(Style::new().fg(Color::Rgb(0, 255, 255))), // Count - голубой
                    Cell::from("Bot Activity").style(Style::new().fg(Color::Rgb(255, 182, 193))), // Activity - розовый
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Type").style(Style::new().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)),
            Cell::from("Count").style(Style::new().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
            Cell::from("Activity").style(Style::new().fg(Color::Rgb(255, 182, 193)).add_modifier(Modifier::BOLD)),
        ]).style(
            Style::new()
                .fg(Color::Rgb(0, 191, 255))
                .add_modifier(Modifier::BOLD)
        );

        frame.render_stateful_widget(
            Table::new(items, [
                Constraint::Length(20), // Type
                Constraint::Length(10), // Count
                Constraint::Min(15),    // Activity
            ])
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 255)))
                    .title("Bot Types"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut self.table_state.clone(),
        );
    }
}

impl Default for BotsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for BotsTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_bots_tab(frame, area, log_data);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, log_data: &LogData) -> bool {
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
                if let Some(selected) = self.table_state.selected() {
                    // Получаем количество типов ботов для определения максимального индекса
                    let top_bot_types = log_data.get_top_bot_types();
                    if selected < top_bot_types.len().saturating_sub(1) {
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
