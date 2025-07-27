use crate::log_data::LogData;
use crate::tui_manager::{TuiManager, PANEL_TITLE_STYLE, SELECTED_ITEM_STYLE, TEXT_FG_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

/// Параметры для отрисовки последних запросов
struct DrawLastRequestsParams<'a, 'b> {
    frame: &'a mut Frame<'b>,
    area: Rect,
    items: Vec<ListItem<'a>>,
    input: &'a str,
    current_page: usize,
    total_pages: usize,
    list_state: &'a mut ListState,
}

pub struct RequestsTab {
    tui_manager: TuiManager,
    list_state: ListState,
    input: String,
    current_page: usize,
    total_pages: usize,
}

impl RequestsTab {
    pub fn new() -> Self {
        Self {
            tui_manager: TuiManager::new(),
            list_state: ListState::default(),
            input: String::new(),
            current_page: 0,
            total_pages: 0,
        }
    }

    /// Renders the last requests panel with search and pagination
    fn draw_last_requests(&self, params: DrawLastRequestsParams<'_, '_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(params.area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Search field
        params.frame.render_widget(
            Paragraph::new(params.input)
                .style(Style::new().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("Search")
                        .title_style(PANEL_TITLE_STYLE),
                ),
            header_chunks[0],
        );

        // Pagination
        let pages: Vec<String> = (1..=params.total_pages).map(|i| format!("{}", i)).collect();
        params.frame.render_widget(
            Tabs::new(pages)
                .select(params.current_page)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("Pages"),
                )
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider("|"),
            header_chunks[1],
        );

        // Request list
        params.frame.render_stateful_widget(
            List::new(params.items.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("Requests"),
                )
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            params.list_state,
        );

        self.tui_manager.draw_scrollbar(
            params.items.len(),
            params.list_state.selected().unwrap_or(0),
            params.frame,
            chunks[1],
        );
    }

    fn get_search_results<'a>(&self, log_data: &'a LogData) -> Vec<&'a String> {
        if !self.input.is_empty() {
            log_data
                .by_ip
                .iter()
                .flat_map(|(_, entry)| &entry.last_requests)
                .filter(|request| request.contains(&self.input))
                .collect()
        } else {
            log_data
                .by_ip
                .values()
                .flat_map(|entry| &entry.last_requests)
                .collect()
        }
    }

    fn on_left(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.list_state.select_first()
        }
    }

    fn on_right(&mut self) {
        if self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.list_state.select_first()
        }
    }

    pub fn copy_selected_to_clipboard(&self, log_data: &LogData) -> Option<String> {
        if let Some(selected_index) = self.list_state.selected() {
            let search_results = self.get_search_results(log_data);
            let start = self.current_page * 100;
            let end = (start + 100).min(search_results.len());

            if selected_index < (end - start) {
                let request = search_results[start + selected_index];

                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if clipboard.set_text(request).is_ok() {
                        let message = "Copied to clipboard".to_string();
                        return Some(message);
                    }
                }
            }
        }
        None
    }
}

impl Default for RequestsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for RequestsTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let search_results = self.get_search_results(log_data);
        let total_pages = (search_results.len() + 99) / 100;
        let start = self.current_page * 100;
        let end = (start + 100).min(search_results.len());

        let items: Vec<ListItem> = search_results[start..end]
            .iter()
            .map(|request| {
                let wrapped_text = textwrap::wrap(request, (area.width as f64 * 0.7) as usize - 5);
                ListItem::new(wrapped_text.join("\n")).style(Style::default().fg(TEXT_FG_COLOR))
            })
            .collect();

        // Обновляем total_pages для использования в on_right
        self.total_pages = total_pages;

        // Клонируем list_state для избежания конфликта заимствований
        let mut list_state_clone = self.list_state.clone();
        self.draw_last_requests(DrawLastRequestsParams {
            frame,
            area,
            items,
            input: &self.input,
            current_page: self.current_page,
            total_pages,
            list_state: &mut list_state_clone,
        });
        self.list_state = list_state_clone;
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
            crossterm::event::KeyCode::Left => {
                self.on_left();
                true
            }
            crossterm::event::KeyCode::Right => {
                self.on_right();
                true
            }
            crossterm::event::KeyCode::Backspace => {
                self.list_state.select(None);
                self.input.pop();
                true
            }
            crossterm::event::KeyCode::Char(c) => {
                self.list_state.select(None);
                self.input.push(c);
                true
            }
            _ => false,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
