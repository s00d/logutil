use crate::memory_db::GLOBAL_DB;
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

    fn draw_bots_tab(&mut self, frame: &mut Frame, area: Rect) {
        let db = GLOBAL_DB.read().unwrap();
        let (bot_ips_count, bot_types_count, bot_urls_count) = db.get_bot_stats();
        let top_user_agents = db.get_top_user_agents(10);

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
        let items: Vec<Row> = top_user_agents
            .iter()
            .map(|(user_agent, count)| {
                let bot_type = if user_agent.contains("bot") || user_agent.contains("crawler") {
                    "Bot/Crawler"
                } else if user_agent.contains("spider") {
                    "Spider"
                } else if user_agent.contains("scraper") {
                    "Scraper"
                } else {
                    "Other"
                };
                
                Row::new(vec![
                    Cell::from(bot_type.to_string()).style(
                        Style::new()
                            .fg(Color::Rgb(255, 255, 0))
                            .add_modifier(Modifier::BOLD),
                    ), // Type - желтый, жирный
                    Cell::from(count.to_string()).style(Style::new().fg(Color::Rgb(0, 255, 255))), // Count - голубой
                    Cell::from("Bot Activity").style(Style::new().fg(Color::Rgb(255, 182, 193))), // Activity - розовый
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Type").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Count").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Activity").style(
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
                    Constraint::Length(20), // Type
                    Constraint::Length(10), // Count
                    Constraint::Min(15),    // Activity
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 255)))
                    .title("Top Bot Types"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut self.table_state,
        );
    }
}

impl Default for BotsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for BotsTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.draw_bots_tab(frame, area);
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
                let top_user_agents = db.get_top_user_agents(10);
                if let Some(selected) = self.table_state.selected() {
                    if selected < top_user_agents.len().saturating_sub(1) {
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
