use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap, Clear},
    Frame,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, Instant};

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

#[derive(Debug)]
struct ModalState {
    message: String,
    show_until: Option<Instant>,
}

pub struct FileSettings {
    // File Selector
    current_path: PathBuf,
    file_items: Vec<FileItem>,
    file_table_state: TableState,
    selected_file_index: usize,
    
    // Settings
    selected_file: Option<PathBuf>,
    settings: Vec<Setting>,
    settings_table_state: TableState,
    selected_setting_index: usize,
    input_mode: bool,
    current_input: String,
    
    // Panel management
    active_panel: usize, // 0 - file selector, 1 - settings
    
    // Modal state
    modal_state: Option<ModalState>,
    
    // Double click tracking
    last_click_time: Option<Instant>,
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

#[derive(Clone)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub description: String,
    pub input_type: InputType,
}

#[derive(Clone)]
pub enum InputType {
    Number,
    Text,
    Boolean,
    Regex,
}

impl FileSettings {
    pub fn new_with_args(cli_args: &CliArgs) -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut instance = Self {
            current_path,
            file_items: Vec::new(),
            file_table_state: TableState::default(),
            selected_file_index: 0,
            selected_file: cli_args.file.clone(),
            settings: vec![
                Setting {
                    name: "Count".to_string(),
                    value: cli_args.count.to_string(),
                    description: "Number of lines to read from the end of the file (0 to start from the end, -1 to read the entire file)".to_string(),
                    input_type: InputType::Number,
                },
                Setting {
                    name: "Regex Pattern".to_string(),
                    value: cli_args.regex.clone(),
                    description: "Regular expression to parse the log entries".to_string(),
                    input_type: InputType::Regex,
                },
                Setting {
                    name: "Date Format".to_string(),
                    value: cli_args.date_format.clone(),
                    description: "Date format to parse the log entries".to_string(),
                    input_type: InputType::Text,
                },
                Setting {
                    name: "Top N".to_string(),
                    value: cli_args.top.to_string(),
                    description: "Number of top entries to display".to_string(),
                    input_type: InputType::Number,
                },
                Setting {
                    name: "Show URLs".to_string(),
                    value: cli_args.show_urls.to_string(),
                    description: "Show top URLs in console".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Show IPs".to_string(),
                    value: cli_args.show_ips.to_string(),
                    description: "Show top IPs in console".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Log to File".to_string(),
                    value: cli_args.log_to_file.to_string(),
                    description: "Enable logging to a file".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Security".to_string(),
                    value: cli_args.enable_security.to_string(),
                    description: "Enable Security tab (detect suspicious activity, attacks, etc.)".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Performance".to_string(),
                    value: cli_args.enable_performance.to_string(),
                    description: "Enable Performance tab (monitor response times, slow requests)".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Errors".to_string(),
                    value: cli_args.enable_errors.to_string(),
                    description: "Enable Errors tab (track error codes and failed requests)".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Bots".to_string(),
                    value: cli_args.enable_bots.to_string(),
                    description: "Enable Bots tab (detect bot activity and user agents)".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Sparkline".to_string(),
                    value: cli_args.enable_sparkline.to_string(),
                    description: "Enable Sparkline tab (show request trends over time)".to_string(),
                    input_type: InputType::Boolean,
                },
                Setting {
                    name: "Enable Heatmap".to_string(),
                    value: cli_args.enable_heatmap.to_string(),
                    description: "Enable Heatmap tab (show request distribution by time)".to_string(),
                    input_type: InputType::Boolean,
                },
            ],
            settings_table_state: TableState::default(),
            selected_setting_index: 0,
            input_mode: false,
            current_input: String::new(),
            active_panel: 0,
            modal_state: None,
            last_click_time: None,
        };
        instance.load_directory();
        instance.settings_table_state.select(Some(0));
        instance
    }

    pub fn load_directory(&mut self) {
        self.file_items.clear();

        // Добавляем ".." для перехода вверх
        if let Some(parent) = self.current_path.parent() {
            self.file_items.push(FileItem {
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

            self.file_items.extend(items);
        }

        self.selected_file_index = 0;
        self.file_table_state.select(Some(0));
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // Левая панель - File Selector
        self.draw_file_selector(frame, chunks[0]);
        
        // Правая панель - Settings
        self.draw_settings(frame, chunks[1]);
        
        // Проверяем и обновляем состояние модального окна
        if let Some(modal) = &self.modal_state {
            if let Some(show_until) = modal.show_until {
                if Instant::now() > show_until {
                    self.modal_state = None;
                } else {
                    self.draw_modal(frame);
                }
            }
        }
    }

    fn draw_file_selector(&mut self, frame: &mut Frame, area: Rect) {
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
        let header_style = if self.active_panel == 0 {
            Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(header_text)
                .style(header_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(if self.active_panel == 0 {
                            Style::new().fg(Color::Rgb(144, 238, 144))
                        } else {
                            Style::new().fg(Color::White)
                        })
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
            .file_items
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
                let style = if index == self.selected_file_index && self.active_panel == 0 {
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
                    .border_style(if self.active_panel == 0 {
                        Style::new().fg(Color::Rgb(144, 238, 144))
                    } else {
                        Style::new().fg(Color::White)
                    })
                    .title("Files and Directories"),
            )
            .row_highlight_style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[1],
            &mut self.file_table_state,
        );

        // Подсказки
        let help_text = "↑/k: Up | ↓/j: Down | Enter: Select File | ←/→/Tab: Switch Panel | Mouse: Click/Scroll";
        frame.render_widget(
            Paragraph::new(help_text)
                .style(Style::new().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(if self.active_panel == 0 {
                            Style::new().fg(Color::Rgb(144, 238, 144))
                        } else {
                            Style::new().fg(Color::White)
                        })
                        .title("Help"),
                ),
            chunks[2],
        );
    }

    fn draw_settings(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Заголовок
                    Constraint::Length(3), // Описание
                    Constraint::Min(0),    // Список настроек
                    Constraint::Length(3), // Подсказки
                ]
                .as_ref(),
            )
            .split(area);

        // Заголовок
        let header_text = if let Some(file) = &self.selected_file {
            format!("⚙️ Settings for: {}", file.display())
        } else {
            "⚙️ Settings (Select a file first)".to_string()
        };
        let header_style = if self.active_panel == 1 {
            Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(header_text)
                .style(header_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(if self.active_panel == 1 {
                            Style::new().fg(Color::Rgb(144, 238, 144))
                        } else {
                            Style::new().fg(Color::White)
                        })
                        .title("🔧 Configuration"),
                ),
            chunks[0],
        );

        // Описание выбранной настройки
        if let Some(setting) = self.settings.get(self.selected_setting_index) {
            let desc_text = format!("📝 {}", setting.description);
            frame.render_widget(
                Paragraph::new(desc_text)
                    .style(Style::new().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(if self.active_panel == 1 {
                                Style::new().fg(Color::Rgb(144, 238, 144))
                            } else {
                                Style::new().fg(Color::White)
                            })
                            .title("Description"),
                    ),
                chunks[1],
            );
        }

        // Список настроек
        let mut rows: Vec<Row> = self
            .settings
            .iter()
            .enumerate()
            .map(|(index, setting)| {
                let selected = index == self.selected_setting_index && self.active_panel == 1;
                let name_style = if selected {
                    Style::new()
                        .fg(Color::Rgb(255, 255, 255))
                        .bg(Color::Rgb(144, 238, 144))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(Color::White)
                };
                let value_style = match setting.input_type {
                    InputType::Boolean => {
                        if selected {
                            Style::new()
                                .fg(Color::Yellow)
                                .bg(Color::Rgb(144, 238, 144))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::new().fg(Color::Yellow)
                        }
                    }
                    _ => {
                        if selected {
                            Style::new()
                                .fg(Color::Cyan)
                                .bg(Color::Rgb(144, 238, 144))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::new().fg(Color::Cyan)
                        }
                    }
                };
                let value_display = match setting.input_type {
                    InputType::Boolean => {
                        let checked = setting.value == "true";
                        if checked { "[x]" } else { "[ ]" }.to_string()
                    }
                    _ => {
                        if self.input_mode && index == self.selected_setting_index && self.active_panel == 1 {
                            format!("{} █", self.current_input)
                        } else {
                            setting.value.clone()
                        }
                    }
                };
                Row::new(vec![
                    Cell::from(setting.name.clone()).style(name_style),
                    Cell::from(value_display).style(value_style),
                ])
            })
            .collect();

        // Добавляем пункт запуска анализа только если файл выбран
        if self.selected_file.is_some() {
            let start_style = if self.selected_setting_index == self.settings.len() && self.active_panel == 1 {
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::Yellow)
            };
            rows.push(Row::new(vec![
                Cell::from("▶ Start Analysis").style(start_style),
                Cell::from("").style(Style::default()),
            ]));
        }

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Setting").style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Value").style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(
            Style::new()
                .fg(Color::Rgb(0, 191, 255))
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(30), // Setting
                    Constraint::Min(20),    // Value
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(if self.active_panel == 1 {
                        Style::new().fg(Color::Rgb(144, 238, 144))
                    } else {
                        Style::new().fg(Color::White)
                    })
                    .title("Settings"),
            )
            .row_highlight_style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[2],
            &mut self.settings_table_state,
        );

        // Подсказки
        let help_text = if self.input_mode {
            "Type value and press Enter to save | Esc to cancel"
        } else {
            "↑/↓: Navigate | Enter: Edit | ←/→/Tab: Switch Panel | Mouse: Click/Scroll | F10: Start Analysis"
        };
        frame.render_widget(
            Paragraph::new(help_text)
                .style(Style::new().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(if self.active_panel == 1 {
                            Style::new().fg(Color::Rgb(144, 238, 144))
                        } else {
                            Style::new().fg(Color::White)
                        })
                        .title("Help"),
                ),
            chunks[3],
        );
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<FileSettingsAction> {
        if self.input_mode {
            self.handle_input_mode(key)
        } else {
            self.handle_navigation_mode(key)
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, file_selector_area: Rect, settings_area: Rect) -> Option<FileSettingsAction> {
        if self.input_mode {
            return None; // Игнорируем мышь в режиме ввода
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Определяем, в какой панели произошел клик
                if mouse.column < file_selector_area.x + file_selector_area.width {
                    // Клик в левой панели (File Selector)
                    self.active_panel = 0;
                    return self.handle_file_selector_click(mouse, file_selector_area);
                } else {
                    // Клик в правой панели (Settings)
                    self.active_panel = 1;
                    return self.handle_settings_click(mouse, settings_area);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                // Обработка двойного клика только для левой панели
                if mouse.column < file_selector_area.x + file_selector_area.width {
                    // Клик в левой панели (File Selector)
                    self.active_panel = 0;
                    return self.handle_file_selector_up(mouse, file_selector_area);
                }
                return None;
            }
            MouseEventKind::ScrollUp => {
                // Прокрутка вверх
                if self.active_panel == 0 {
                    if self.selected_file_index > 0 {
                        self.selected_file_index -= 1;
                        self.file_table_state.select(Some(self.selected_file_index));
                        // Обновляем выбранный файл
                        self.update_selected_file();
                    }
                } else {
                    if self.selected_setting_index > 0 {
                        self.selected_setting_index -= 1;
                        self.settings_table_state.select(Some(self.selected_setting_index));
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                // Прокрутка вниз
                if self.active_panel == 0 {
                    if self.selected_file_index < self.file_items.len().saturating_sub(1) {
                        self.selected_file_index += 1;
                        self.file_table_state.select(Some(self.selected_file_index));
                        // Обновляем выбранный файл
                        self.update_selected_file();
                    }
                } else {
                    let max_index = if self.selected_file.is_some() { self.settings.len() } else { self.settings.len() - 1 };
                    if self.selected_setting_index < max_index {
                        self.selected_setting_index += 1;
                        self.settings_table_state.select(Some(self.selected_setting_index));
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn handle_file_selector_up(&mut self, _mouse: MouseEvent, _panel_area: Rect) -> Option<FileSettingsAction> {  
        // Проверяем двойной клик
        if let Some(last_time) = self.last_click_time {
            let now = Instant::now();
            // Проверяем, что прошло менее 500мс с последнего клика
            if now.duration_since(last_time).as_millis() < 500 {
                // Двойной клик - запускаем анализ для выбранного файла
                if let Some(item) = self.file_items.get(self.selected_file_index) {
                    if item.is_file {
                        if item.path.exists() {
                            return Some(FileSettingsAction::StartAnalysis(self.get_cli_args()));
                        } else {
                            self.show_modal("Selected file does not exist!".to_string());
                            return None;
                        }
                    }
                }
            }
            // Сбрасываем отслеживание двойного клика
            self.last_click_time = None;
        }
         // Сохраняем информацию для двойного клика
        self.last_click_time = Some(Instant::now());
        None
    }



    fn handle_file_selector_click(&mut self, mouse: MouseEvent, panel_area: Rect) -> Option<FileSettingsAction> {
        // Получаем размеры layout из draw_file_selector
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
            .split(panel_area);
        
        let table_area = chunks[1]; // Область таблицы
        
        // Проверяем, что клик в области таблицы
        if mouse.row >= table_area.y && mouse.row < table_area.y + table_area.height {
            // Учитываем заголовок таблицы (1 строка) + верхнюю границу (1 строка)
            let data_start_y = table_area.y + 2;
            if mouse.row >= data_start_y {
                let row_index = (mouse.row - data_start_y) as usize;
                if row_index < self.file_items.len() {
                    self.selected_file_index = row_index;
                    self.file_table_state.select(Some(self.selected_file_index));
                    

                    
                    // Немедленно обрабатываем выбор файла
                    if let Some(item) = self.file_items.get(row_index) {
                        if item.is_parent {
                            // Переходим в родительскую директорию
                            self.current_path = item.path.clone();
                            self.selected_file = None; // Сбрасываем выбранный файл
                            self.load_directory();
                            return None;
                        } else if item.is_dir {
                            // Переходим в директорию
                            self.current_path = item.path.clone();
                            self.selected_file = None; // Сбрасываем выбранный файл
                            self.load_directory();
                            return None;
                        } else if item.is_file {
                            // Выбираем файл (но не запускаем анализ)
                            self.selected_file = Some(item.path.clone());
                            return None;
                        }
                    }
                }
            }
        }
        None
    }

    fn handle_settings_click(&mut self, mouse: MouseEvent, panel_area: Rect) -> Option<FileSettingsAction> {
        // Получаем размеры layout из draw_settings
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Заголовок
                    Constraint::Length(3), // Описание
                    Constraint::Min(0),    // Список настроек
                    Constraint::Length(3), // Подсказки
                ]
                .as_ref(),
            )
            .split(panel_area);
        
        let table_area = chunks[2]; // Область таблицы настроек
        
        // Проверяем, что клик в области таблицы
        if mouse.row >= table_area.y && mouse.row < table_area.y + table_area.height {
            // Учитываем заголовок таблицы (1 строка) + верхнюю границу (1 строка)
            let data_start_y = table_area.y + 2;
            if mouse.row >= data_start_y {
                let row_index = (mouse.row - data_start_y) as usize;
                if row_index < self.settings.len() {
                    self.selected_setting_index = row_index;
                    self.settings_table_state.select(Some(self.selected_setting_index));
                    
                    // Немедленно обрабатываем выбор настройки
                    if let Some(setting) = self.settings.get_mut(row_index) {
                        match setting.input_type {
                            InputType::Boolean => {
                                // Переключаем значение
                                setting.value = if setting.value == "true" {
                                    "false".to_string()
                                } else {
                                    "true".to_string()
                                };
                            }
                            _ => {
                                // Включаем режим редактирования
                                self.current_input = setting.value.clone();
                                self.input_mode = true;
                            }
                        }
                    }
                } else if row_index == self.settings.len() {
                    // Клик на "▶ Start Analysis"
                    if let Some(file) = &self.selected_file {
                        // Проверяем, что файл существует
                        if file.exists() {
                            // Запускаем анализ
                            return Some(FileSettingsAction::StartAnalysis(self.get_cli_args()));
                        }
                    }
                    // Если файл не выбран или не существует, показываем модальное окно
                    self.show_modal("Please select a file first!".to_string());
                    return None;
                } else if row_index == self.settings.len() + 1 {
                    // Клик на пустую строку после "▶ Start Analysis"
                    if let Some(file) = &self.selected_file {
                        // Проверяем, что файл существует
                        if file.exists() {
                            // Запускаем анализ
                            return Some(FileSettingsAction::StartAnalysis(self.get_cli_args()));
                        }
                    }
                    // Если файл не выбран или не существует, показываем модальное окно
                    self.show_modal("Please select a file first!".to_string());
                    return None;
                }
            }
        }
        None
    }

    fn handle_input_mode(&mut self, key: KeyEvent) -> Option<FileSettingsAction> {
        match key.code {
            KeyCode::Enter => {
                if let Some(setting) = self.settings.get_mut(self.selected_setting_index) {
                    setting.value = self.current_input.clone();
                }
                self.input_mode = false;
                self.current_input.clear();
                None
            }
            KeyCode::Esc => {
                if self.input_mode {
                    self.input_mode = false;
                    self.current_input.clear();
                    None
                } else {
                    Some(FileSettingsAction::Exit)
                }
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(FileSettingsAction::Exit)
            }
            KeyCode::Char(c) => {
                self.current_input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                None
            }
            _ => None,
        }
    }

    fn handle_navigation_mode(&mut self, key: KeyEvent) -> Option<FileSettingsAction> {
        match key.code {
            KeyCode::Tab | KeyCode::Right => {
                // Переключение между панелями (вправо)
                self.active_panel = if self.active_panel == 0 { 1 } else { 0 };
                None
            }
            KeyCode::Left => {
                // Переключение между панелями (влево)
                self.active_panel = if self.active_panel == 0 { 1 } else { 0 };
                None
            }
            KeyCode::Up => {
                if self.active_panel == 0 {
                    // Навигация в файловом селекторе
                    if self.selected_file_index > 0 {
                        self.selected_file_index -= 1;
                        self.file_table_state.select(Some(self.selected_file_index));
                        // Сбрасываем выбранный файл при навигации
                        self.selected_file = None;
                        // Обновляем выбранный файл
                        self.update_selected_file();
                    }
                } else {
                    // Навигация в настройках
                    if self.selected_setting_index > 0 {
                        self.selected_setting_index -= 1;
                        self.settings_table_state.select(Some(self.selected_setting_index));
                    }
                }
                None
            }
            KeyCode::Down => {
                if self.active_panel == 0 {
                    // Навигация в файловом селекторе
                    if self.selected_file_index < self.file_items.len().saturating_sub(1) {
                        self.selected_file_index += 1;
                        self.file_table_state.select(Some(self.selected_file_index));
                        // Сбрасываем выбранный файл при навигации
                        self.selected_file = None;
                        // Обновляем выбранный файл
                        self.update_selected_file();
                    }
                } else {
                    // Навигация в настройках
                    let max_index = if self.selected_file.is_some() { self.settings.len() } else { self.settings.len() - 1 };
                    if self.selected_setting_index <= max_index {
                        self.selected_setting_index += 1;
                        self.settings_table_state.select(Some(self.selected_setting_index));
                    }
                }
                None
            }
            KeyCode::Enter => {
                if self.active_panel == 0 {
                    // Обработка в файловом селекторе
                    if let Some(item) = self.file_items.get(self.selected_file_index) {
                        if item.is_parent {
                            // Переходим в родительскую директорию
                            self.current_path = item.path.clone();
                            self.selected_file = None; // Сбрасываем выбранный файл
                            self.load_directory();
                            None
                        } else if item.is_dir {
                            // Переходим в директорию
                            self.current_path = item.path.clone();
                            self.selected_file = None; // Сбрасываем выбранный файл
                            self.load_directory();
                            None
                        } else if item.is_file {
                            // Выбираем файл и сразу запускаем анализ
                            self.selected_file = Some(item.path.clone());
                            // Проверяем, что файл существует
                            if item.path.exists() {
                                return Some(FileSettingsAction::StartAnalysis(self.get_cli_args()));
                            } else {
                                self.show_modal("Selected file does not exist!".to_string());
                                return None;
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    // Обработка в настройках
                    if self.selected_setting_index >= self.settings.len() {
                        // Запуск анализа
                        if let Some(file) = &self.selected_file {
                            // Проверяем, что файл существует
                            if file.exists() {
                                return Some(FileSettingsAction::StartAnalysis(self.get_cli_args()));
                            }
                        }
                        // Если файл не выбран или не существует, показываем модальное окно
                        self.show_modal("Please select a file first!".to_string());
                        return None;
                    }
                    if let Some(setting) = self.settings.get_mut(self.selected_setting_index) {
                        match setting.input_type {
                            InputType::Boolean => {
                                // Переключаем значение
                                setting.value = if setting.value == "true" {
                                    "false".to_string()
                                } else {
                                    "true".to_string()
                                };
                            }
                            _ => {
                                self.current_input = setting.value.clone();
                                self.input_mode = true;
                            }
                        }
                    }
                    None
                }
            }
            KeyCode::Char('h') if self.active_panel == 0 => {
                // Переход в родительскую директорию (как h в vim)
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    self.selected_file = None; // Сбрасываем выбранный файл
                    self.load_directory();
                }
                None
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(FileSettingsAction::Exit)
            }
            _ => None,
        }
    }

    pub fn get_cli_args(&self) -> CliArgs {
        CliArgs {
            file: self.selected_file.clone(),
            regex: self.settings[1].value.clone(),
            date_format: self.settings[2].value.clone(),
            count: self.settings[0].value.parse().unwrap_or(0),
            top: self.settings[3].value.parse().unwrap_or(10),
            show_urls: self.settings[4].value.parse().unwrap_or(false),
            show_ips: self.settings[5].value.parse().unwrap_or(false),
            log_to_file: self.settings[6].value.parse().unwrap_or(false),
            enable_security: self.settings[7].value.parse().unwrap_or(false),
            enable_performance: self.settings[8].value.parse().unwrap_or(false),
            enable_errors: self.settings[9].value.parse().unwrap_or(false),
            enable_bots: self.settings[10].value.parse().unwrap_or(false),
            enable_sparkline: self.settings[11].value.parse().unwrap_or(false),
            enable_heatmap: self.settings[12].value.parse().unwrap_or(false),
        }
    }

    pub fn enable_mouse(&self) -> anyhow::Result<()> {
        execute!(std::io::stdout(), EnableMouseCapture)?;
        Ok(())
    }

    pub fn disable_mouse(&self) -> anyhow::Result<()> {
        execute!(std::io::stdout(), DisableMouseCapture)?;
        Ok(())
    }

    fn update_selected_file(&mut self) {
        if let Some(item) = self.file_items.get(self.selected_file_index) {
            if item.is_file {
                self.selected_file = Some(item.path.clone());
            }
        }
    }

    pub fn show_modal(&mut self, message: String) {
        self.modal_state = Some(ModalState {
            message,
            show_until: Some(Instant::now() + std::time::Duration::from_millis(2000)),
        });
    }

    fn draw_modal(&self, frame: &mut Frame) {
        if let Some(modal) = &self.modal_state {
            // Используем TuiManager для отрисовки модального окна
            // Создаем простую реализацию модального окна
            let area = frame.area();
            let popup_width = (area.width as f32 * 0.4) as u16;
            let popup_height = 8;
            
            let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
            let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
            
            let popup_area = Rect::new(x, y, popup_width, popup_height);
            
            // Очищаем область под попапом
            frame.render_widget(Clear, popup_area);
            
            // Создаем вертикальный layout для содержимого попапа
            let chunks = Layout::vertical([
                Constraint::Length(3), // Заголовок
                Constraint::Length(1), // Пустая строка для отступа
                Constraint::Length(3), // Основное сообщение
                Constraint::Length(1), // Пустая строка для отступа
            ])
            .spacing(0)
            .split(popup_area);
            
            // Рисуем основной блок попапа
            let block = Block::default()
                .title("⚠️ Warning")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(Style::default().bg(Color::Rgb(28, 28, 28)).fg(Color::White));
            
            frame.render_widget(block, popup_area);
            
            // Разбиваем сообщение на строки
            let lines: Vec<&str> = modal.message.split('\n').collect();
            
            // Рисуем иконку и основное сообщение
            let icon = "⚠️";
            let message = format!("{} {}", icon, lines[0]);
            let paragraph = Paragraph::new(message)
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 165, 0)) // Оранжевый цвет для предупреждения
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[2]);
            
            // Рисуем дополнительное сообщение (если есть)
            if lines.len() > 1 {
                let submessage = Paragraph::new(lines[1])
                    .style(Style::default().fg(Color::Rgb(200, 200, 200)))
                    .alignment(ratatui::layout::Alignment::Center)
                    .wrap(Wrap { trim: true });
                frame.render_widget(submessage, chunks[4]);
            }
        }
    }
}

#[derive(Debug)]
pub enum FileSettingsAction {
    StartAnalysis(CliArgs),
    Exit,
}

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub file: Option<PathBuf>,
    pub regex: String,
    pub date_format: String,
    pub count: isize,
    pub top: usize,
    pub show_urls: bool,
    pub show_ips: bool,
    pub log_to_file: bool,
    pub enable_security: bool,
    pub enable_performance: bool,
    pub enable_errors: bool,
    pub enable_bots: bool,
    pub enable_sparkline: bool,
    pub enable_heatmap: bool,
} 