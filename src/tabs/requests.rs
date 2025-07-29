use crate::memory_db::GLOBAL_DB;
use crate::tui_manager::{PANEL_TITLE_STYLE, SELECTED_ITEM_STYLE, TEXT_FG_COLOR};
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
    table_state: TableState,
    input: String,
    current_page: usize,
    total_pages: usize,
}

impl RequestsTab {
    pub fn new() -> Self {
        let mut instance = Self {
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
                params.rows,
                [Constraint::Min(50)],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Last Requests"),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            params.table_state,
        );
    }

    fn get_search_results(&mut self) -> Vec<String> {
        let db = GLOBAL_DB.read().unwrap();
        let records = db.get_all_records();
        let all_results: Vec<String> = records.iter().map(|record| record.log_line.clone()).collect();

        // Применяем фильтр поиска
        if self.input.is_empty() {
            all_results
        } else {
            all_results
                .iter()
                .filter(|record| record.to_lowercase().contains(&self.input.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    fn on_left(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
        self.table_state.select(Some(0));
    }

    fn on_right(&mut self) {
        self.current_page = self.current_page.saturating_add(1);
        self.table_state.select(Some(0));
    }

    pub fn copy_selected_to_clipboard(&mut self) -> Option<String> {
        if let Some(selected_idx) = self.table_state.selected() {
            let search_results = self.get_search_results();
            
            if selected_idx < search_results.len() {
                let selected_request = &search_results[selected_idx];
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if clipboard.set_text(selected_request).is_ok() {
                        return Some(format!("Request copied to clipboard"));
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
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let search_results = self.get_search_results();
        let items_per_page = 30; // Изменили на 30
        let total_items = search_results.len();
        self.total_pages = if total_items == 0 {
            1
        } else {
            (total_items + items_per_page - 1) / items_per_page
        };

        // Ограничиваем current_page
        if self.current_page >= self.total_pages {
            self.current_page = self.total_pages.saturating_sub(1);
        }

        let start_idx = self.current_page * items_per_page;
        let end_idx = std::cmp::min(start_idx + items_per_page, total_items);

        let rows: Vec<Row> = search_results
            .iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
            .enumerate()
            .map(|(_i, request)| {
                Row::new(vec![Cell::from(request.as_str()).style(
                    Style::new()
                        .fg(TEXT_FG_COLOR)
                        .add_modifier(Modifier::BOLD),
                )])
            })
            .collect();

        let mut table_state = self.table_state.clone();
        let params = DrawLastRequestsParams {
            frame,
            area,
            rows,
            input: &self.input,
            current_page: self.current_page,
            total_pages: self.total_pages,
            table_state: &mut table_state,
        };

        self.draw_last_requests(params);
        self.table_state = table_state;
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.input.push(c);
                self.current_page = 0;
                self.table_state.select(Some(0));
                true
            }
            crossterm::event::KeyCode::Enter => {
                self.copy_selected_to_clipboard();
                true
            }
            crossterm::event::KeyCode::Backspace => {
                self.input.pop();
                self.current_page = 0;
                self.table_state.select(Some(0));
                true
            }
            crossterm::event::KeyCode::Up => {
                if let Some(selected) = self.table_state.selected() {
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                let search_results = self.get_search_results();
                let items_per_page = 30; // Изменили на 30
                let start_idx = self.current_page * items_per_page;
                let end_idx = std::cmp::min(start_idx + items_per_page, search_results.len());
                let current_page_items = end_idx - start_idx;

                if let Some(selected) = self.table_state.selected() {
                    if selected < current_page_items.saturating_sub(1) {
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

            _ => false,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
