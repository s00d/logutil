use crate::log_data::LogData;
use crate::tui_manager::{TuiManager, PANEL_TITLE_STYLE, SELECTED_ITEM_STYLE, TEXT_FG_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs},
    Frame,
};

/// Параметры для отрисовки последних запросов
struct DrawLastRequestsParams<'a, 'b> {
    frame: &'a mut Frame<'b>,
    area: Rect,
    rows: Vec<Row<'a>>,
    input: &'a str,
    current_page: usize,
    total_pages: usize,
    table_state: &'a mut TableState,
}

pub struct RequestsTab {
    tui_manager: TuiManager,
    table_state: TableState,
    input: String,
    current_page: usize,
    total_pages: usize,
}

impl RequestsTab {
    pub fn new() -> Self {
        let mut instance = Self {
            tui_manager: TuiManager::new(),
            table_state: TableState::default(),
            input: String::new(),
            current_page: 0,
            total_pages: 0,
        };

        // Инициализируем выделение
        instance.table_state.select(Some(0));

        instance
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
        // Создаем заголовок для таблицы
        let header = Row::new(vec![Cell::from("Request Content").style(
            Style::new()
                .fg(Color::Rgb(255, 255, 0))
                .add_modifier(Modifier::BOLD),
        )])
        .style(
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(80, 80, 80)) // Серый фон для заголовка
                .add_modifier(Modifier::BOLD),
        );

        params.frame.render_stateful_widget(
            Table::new(
                params.rows.clone(),
                [
                    Constraint::Min(50), // Request content
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Requests"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            params.table_state,
        );

        self.tui_manager.draw_scrollbar(
            params.rows.len(),
            params.table_state.selected().unwrap_or(0),
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
            self.table_state.select(Some(0));
        }
    }

    fn on_right(&mut self) {
        if self.total_pages > 0 && self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.table_state.select(Some(0));
        }
    }

    pub fn copy_selected_to_clipboard(&self, log_data: &LogData) -> Option<String> {
        if let Some(selected_index) = self.table_state.selected() {
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

        let rows: Vec<Row> = search_results[start..end]
            .iter()
            .map(|request| {
                // Обрезаем запрос для отображения в таблице
                let max_length = (area.width as f64 * 0.7) as usize - 5;
                let display_text = if request.len() > max_length {
                    format!("{}...", &request[..max_length])
                } else {
                    request.to_string()
                };
                Row::new(vec![
                    Cell::from(display_text).style(Style::default().fg(TEXT_FG_COLOR))
                ])
            })
            .collect();

        // Обновляем total_pages для использования в on_right
        self.total_pages = total_pages;

        // Клонируем table_state для избежания конфликта заимствований
        let mut table_state_clone = self.table_state.clone();
        self.draw_last_requests(DrawLastRequestsParams {
            frame,
            area,
            rows,
            input: &self.input,
            current_page: self.current_page,
            total_pages,
            table_state: &mut table_state_clone,
        });
        self.table_state = table_state_clone;
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
                    // Получаем количество результатов для определения максимального индекса
                    let search_results = self.get_search_results(log_data);
                    let start = self.current_page * 100;
                    let end = (start + 100).min(search_results.len());
                    let page_items = end - start;

                    if selected < page_items.saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                }
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
                self.table_state.select(None);
                self.input.pop();
                true
            }
            crossterm::event::KeyCode::Char(c) => {
                self.table_state.select(None);
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
