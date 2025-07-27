use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Форматирует размер файла в читаемом виде
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        0..KB => format!("{} B", bytes),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        _ => format!("{:.1} GB", bytes as f64 / GB as f64),
    }
}

/// Форматирует дату в читаемом виде
fn format_datetime(time: SystemTime) -> String {
    let datetime: DateTime<Local> = DateTime::from(time);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

pub struct FileSelector {
    current_path: PathBuf,
    items: Vec<FileItem>,
    table_state: TableState,
    selected_index: usize,
}

#[derive(Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_parent: bool,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

impl FileSelector {
    pub fn new() -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut selector = Self {
            current_path,
            items: Vec::new(),
            table_state: TableState::default(),
            selected_index: 0,
        };
        selector.load_directory();
        selector
    }

    pub fn load_directory(&mut self) {
        self.items.clear();

        // Добавляем ".." для перехода вверх
        if let Some(parent) = self.current_path.parent() {
            self.items.push(FileItem {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                is_file: false,
                is_parent: true,
                size: None,
                modified: None,
            });
        }

        // Читаем содержимое текущей директории
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            let mut items = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Получаем метаданные файла
                let metadata = fs::metadata(&path).ok();
                let size =
                    metadata
                        .as_ref()
                        .and_then(|m| if m.is_file() { Some(m.len()) } else { None });
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());

                if path.is_dir() {
                    let is_parent = name == "..";
                    let is_current = name == ".";

                    if is_parent {
                        items.push(FileItem {
                            name: ".. (Parent Directory)".to_string(),
                            path: path.clone(),
                            is_dir: true,
                            is_file: false,
                            is_parent: true,
                            size: None,
                            modified: None,
                        });
                    } else if !is_current {
                        items.push(FileItem {
                            name,
                            path,
                            is_dir: true,
                            is_file: false,
                            is_parent: false,
                            size: None,
                            modified,
                        });
                    }
                } else {
                    items.push(FileItem {
                        name,
                        path,
                        is_dir: false,
                        is_file: true,
                        is_parent: false,
                        size,
                        modified,
                    });
                }
            }

            // Сортируем директории и файлы
            items.sort_by(|a, b| {
                // Сначала сравниваем по типу (папки вверху)
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name), // Если оба одного типа, сортируем по имени
                }
            });

            self.items.extend(items);
        }

        self.selected_index = 0;
        self.table_state.select(Some(0));
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Заголовок
                    Constraint::Min(0),    // Таблица
                    Constraint::Length(3), // Подсказки
                ]
                .as_ref(),
            )
            .split(area);

        // Заголовок с текущим путем
        let header_text = format!("📁 Current Directory: {}", self.current_path.display());
        frame.render_widget(
            Paragraph::new(header_text)
                .style(Style::new().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("📂 File Selector"),
                ),
            chunks[0],
        );

        // Заголовок таблицы
        let header = Row::new(vec![
            Cell::from("Type"),
            Cell::from("Name"),
            Cell::from("Size"),
            Cell::from("Modified"),
        ])
        .style(
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(100, 100, 100))
                .add_modifier(Modifier::BOLD),
        );

        // Данные таблицы
        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let icon = if item.is_parent {
                    "⬆️"
                } else if item.is_dir {
                    "📁"
                } else {
                    "📄"
                };
                let style = if index == self.selected_index {
                    Style::new()
                        .fg(Color::Rgb(255, 255, 255))
                        .bg(Color::Rgb(144, 238, 144))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(Color::White)
                };
                let size_str = item.size.map(format_size).unwrap_or_default();
                let date_str = item.modified.map(format_datetime).unwrap_or_default();
                Row::new(vec![
                    Cell::from(icon),
                    Cell::from(item.name.clone()),
                    Cell::from(size_str),
                    Cell::from(date_str),
                ])
                .style(style)
            })
            .collect();

        frame.render_stateful_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(4),  // Type (icon)
                    Constraint::Min(20),    // Name
                    Constraint::Length(15), // Size
                    Constraint::Length(20), // Modified
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Files and Directories"),
            )
            .row_highlight_style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[1],
            &mut self.table_state,
        );

        // Подсказки
        let help_text = "↑/k: Up | ↓/j: Down | PageUp/PageDown: Page | Home/End: Start/End | h: Parent | Enter: Select | Esc/q: Cancel | Ctrl+C: Exit";
        frame.render_widget(
            Paragraph::new(help_text)
                .style(Style::new().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("Help"),
                ),
            chunks[2],
        );
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> FileSelectorAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.table_state.select(Some(self.selected_index));
                }
                FileSelectorAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.items.len().saturating_sub(1) {
                    self.selected_index += 1;
                    self.table_state.select(Some(self.selected_index));
                }
                FileSelectorAction::Continue
            }
            KeyCode::PageUp => {
                // Переход на страницу вверх
                let page_size = 10;
                if self.selected_index >= page_size {
                    self.selected_index -= page_size;
                } else {
                    self.selected_index = 0;
                }
                self.table_state.select(Some(self.selected_index));
                FileSelectorAction::Continue
            }
            KeyCode::PageDown => {
                // Переход на страницу вниз
                let page_size = 10;
                let max_index = self.items.len().saturating_sub(1);
                if self.selected_index + page_size <= max_index {
                    self.selected_index += page_size;
                } else {
                    self.selected_index = max_index;
                }
                self.table_state.select(Some(self.selected_index));
                FileSelectorAction::Continue
            }
            KeyCode::Home => {
                // Переход в начало списка
                self.selected_index = 0;
                self.table_state.select(Some(0));
                FileSelectorAction::Continue
            }
            KeyCode::End => {
                // Переход в конец списка
                let max_index = self.items.len().saturating_sub(1);
                self.selected_index = max_index;
                self.table_state.select(Some(max_index));
                FileSelectorAction::Continue
            }
            KeyCode::Char('h') => {
                // Переход в родительскую директорию (как h в vim)
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    self.load_directory();
                }
                FileSelectorAction::Continue
            }
            KeyCode::Enter => {
                if let Some(item) = self.items.get(self.selected_index) {
                    if item.is_parent {
                        // Переходим в родительскую директорию
                        self.current_path = item.path.clone();
                        self.load_directory();
                        FileSelectorAction::Continue
                    } else if item.is_dir {
                        // Переходим в директорию
                        self.current_path = item.path.clone();
                        self.load_directory();
                        FileSelectorAction::Continue
                    } else if item.is_file {
                        // Выбираем файл
                        FileSelectorAction::FileSelected(item.path.clone())
                    } else {
                        FileSelectorAction::Continue
                    }
                } else {
                    FileSelectorAction::Continue
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => FileSelectorAction::Cancel,
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Ctrl+C для завершения программы
                FileSelectorAction::Exit
            }
            _ => FileSelectorAction::Continue,
        }
    }
}

#[derive(Debug)]
pub enum FileSelectorAction {
    Continue,
    FileSelected(PathBuf),
    Cancel,
    Exit,
}
