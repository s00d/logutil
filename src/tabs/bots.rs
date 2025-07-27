use crate::log_data::LogData;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct BotsTab {
    list_state: ListState,
}

impl BotsTab {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    fn draw_bots_tab(&self, frame: &mut Frame, area: Rect, log_data: &LogData) {
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
        let items: Vec<ListItem> = top_bot_types
            .iter()
            .map(|(bot_type, count)| {
                ListItem::new(format!("{:<20} │ {:<10} │ Bot Activity", bot_type, count))
                    .style(Style::new().fg(Color::Rgb(0, 255, 255)))
            })
            .collect();

        frame.render_stateful_widget(
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 255)))
                        .title("Bot Types"),
                )
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut self.list_state.clone(),
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
