use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use std::path::PathBuf;

pub struct Settings {
    selected_file: PathBuf,
    settings: Vec<Setting>,
    table_state: TableState,
    selected_index: usize,
    input_mode: bool,
    current_input: String,
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

impl Settings {
    pub fn new_with_args(selected_file: PathBuf, cli_args: &CliArgs) -> Self {
        let settings = vec![
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
        ];

        let mut instance = Self {
            selected_file,
            settings,
            table_state: TableState::default(),
            selected_index: 0,
            input_mode: false,
            current_input: String::new(),
        };

        // Инициализируем выделение
        instance.table_state.select(Some(0));

        instance
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
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
        let header_text = format!("⚙️ Settings for: {}", self.selected_file.display());
        frame.render_widget(
            Paragraph::new(header_text)
                .style(Style::new().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                        .title("🔧 Configuration"),
                ),
            chunks[0],
        );

        // Описание выбранной настройки
        if let Some(setting) = self.settings.get(self.selected_index) {
            let desc_text = format!("📝 {}", setting.description);
            frame.render_widget(
                Paragraph::new(desc_text)
                    .style(Style::new().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
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
                let selected = index == self.selected_index;
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
                        if self.input_mode && index == self.selected_index {
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

        // Добавляем пункт запуска анализа
        let start_style = if self.selected_index == self.settings.len() {
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
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Settings"),
            )
            .row_highlight_style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            chunks[2],
            &mut self.table_state,
        );

        // Подсказки
        let help_text = if self.input_mode {
            "Type value and press Enter to save | Esc to cancel | Ctrl+C: Exit"
        } else {
            "↑/↓: Navigate | Enter: Edit | F10: Start Analysis | Esc: Back | Ctrl+C: Exit"
        };
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
            chunks[3],
        );
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> SettingsAction {
        if self.input_mode {
            self.handle_input_mode(key)
        } else {
            self.handle_navigation_mode(key)
        }
    }

    fn handle_input_mode(&mut self, key: KeyEvent) -> SettingsAction {
        match key.code {
            KeyCode::Enter => {
                if let Some(setting) = self.settings.get_mut(self.selected_index) {
                    setting.value = self.current_input.clone();
                }
                self.input_mode = false;
                self.current_input.clear();
                SettingsAction::Continue
            }
            KeyCode::Esc => {
                self.input_mode = false;
                self.current_input.clear();
                SettingsAction::Continue
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Ctrl+C для завершения программы
                SettingsAction::Exit
            }
            KeyCode::Char(c) => {
                self.current_input.push(c);
                SettingsAction::Continue
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                SettingsAction::Continue
            }
            _ => SettingsAction::Continue,
        }
    }

    fn handle_navigation_mode(&mut self, key: KeyEvent) -> SettingsAction {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.table_state.select(Some(self.selected_index));
                }
                SettingsAction::Continue
            }
            KeyCode::Down => {
                if self.selected_index <= self.settings.len() {
                    self.selected_index += 1;
                    self.table_state.select(Some(self.selected_index));
                }
                SettingsAction::Continue
            }
            KeyCode::Enter => {
                if self.selected_index >= self.settings.len() {
                    // Запуск анализа
                    return SettingsAction::StartAnalysis(self.get_cli_args());
                }
                if let Some(setting) = self.settings.get_mut(self.selected_index) {
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
                SettingsAction::Continue
            }
            KeyCode::Esc => SettingsAction::Back,
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Ctrl+C для завершения программы
                SettingsAction::Exit
            }
            _ => SettingsAction::Continue,
        }
    }

    pub fn get_cli_args(&self) -> CliArgs {
        CliArgs {
            file: Some(self.selected_file.clone()),
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
}

#[derive(Debug)]
pub enum SettingsAction {
    Continue,
    Back,
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
