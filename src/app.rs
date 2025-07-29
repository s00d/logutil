use crate::{
    memory_db::GLOBAL_DB,
    tab_manager::TabManager,
    tabs::{
        base::Tab,
        bots::BotsTab,
        detailed::DetailedTab,
        errors::ErrorsTab,
        heatmap::HeatmapTab,
        overview::OverviewTab,
        performance::PerformanceTab,
        requests::RequestsTab,
        security::SecurityTab,
        sparkline::SparklineTab,
    },
    tui_manager::{draw_tui_progress_bar, TuiManager, HEADER_STYLE},
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::time::Instant;

/// Конфигурация для создания App
#[derive(Debug)]
pub struct AppConfig {
    pub enable_security: bool,
    pub enable_performance: bool,
    pub enable_errors: bool,
    pub enable_bots: bool,
    pub enable_sparkline: bool,
    pub enable_heatmap: bool,
}

#[derive(Debug)]
struct ModalState {
    message: String,
    show_until: Option<Instant>,
}

pub struct App {
    pub(crate) should_quit: bool,
    tab_manager: TabManager,
    tabs: Vec<Box<dyn Tab>>,
    progress: f64,
    modal_state: Option<ModalState>,
    last_summary_update: std::time::Instant,
    cached_summary: Option<(String, String, String, String)>,
}

impl App {
    pub(crate) fn new(config: AppConfig) -> Self {
        let mut tab_names = vec![
            "Overview".to_string(),
            "Requests".to_string(),
            "Detailed".to_string(),
        ];
        let mut tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(OverviewTab::new()),
            Box::new(RequestsTab::new()),
            Box::new(DetailedTab::new()),
        ];

        // Добавляем дополнительные табы только если они включены
        if config.enable_sparkline {
            tab_names.push("Sparkline".to_string());
            tabs.push(Box::new(SparklineTab::new()));
        }
        if config.enable_heatmap {
            tab_names.push("Heatmap".to_string());
            tabs.push(Box::new(HeatmapTab::new()));
        }
        if config.enable_security {
            tab_names.push("Security".to_string());
            tabs.push(Box::new(SecurityTab::new()));
        }
        if config.enable_performance {
            tab_names.push("Performance".to_string());
            tabs.push(Box::new(PerformanceTab::new()));
        }
        if config.enable_errors {
            tab_names.push("Errors".to_string());
            tabs.push(Box::new(ErrorsTab::new()));
        }
        if config.enable_bots {
            tab_names.push("Bots".to_string());
            tabs.push(Box::new(BotsTab::new()));
        }

        Self {
            should_quit: false,
            tab_manager: TabManager::new(tab_names),
            tabs,
            progress: 0.0,
            modal_state: None,
            last_summary_update: std::time::Instant::now(),
            cached_summary: None,
        }
    }



    pub(crate) fn handle_input(
        &mut self,
        key: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) {
        let key_event = crossterm::event::KeyEvent::new(key, modifiers);
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            KeyCode::Enter => {
                let idx = self.tab_manager.current_tab();
                if let Some(_tab) = self.tabs.get_mut(idx) {
                    let tab_name = self.tab_manager.current_tab_name().unwrap_or("");

                    // Обработка Enter для разных табов по имени
                    match tab_name {
                        "Overview" => {
                            if let Some(overview_tab) =
                                _tab.as_any_mut().downcast_mut::<crate::tabs::OverviewTab>()
                            {
                                if let Some(message) = overview_tab.copy_selected_to_clipboard() {
                                    self.modal_state = Some(ModalState {
                                        message: message.to_string(),
                                        show_until: Some(
                                            Instant::now()
                                                + std::time::Duration::from_millis(1500),
                                        ),
                                    });
                                }
                            }
                        }
                        "Requests" => {
                            if let Some(requests_tab) =
                                _tab.as_any_mut().downcast_mut::<crate::tabs::RequestsTab>()
                            {
                                if let Some(message) = requests_tab.copy_selected_to_clipboard() {
                                    self.modal_state = Some(ModalState {
                                        message: message.to_string(),
                                        show_until: Some(
                                            Instant::now()
                                                + std::time::Duration::from_millis(1500),
                                        ),
                                    });
                                }
                            }
                        }
                        "Detailed" => {
                            if let Some(detailed_tab) =
                                _tab.as_any_mut().downcast_mut::<crate::tabs::DetailedTab>()
                            {
                                if let Some(message) = detailed_tab.copy_selected_to_clipboard() {
                                    self.modal_state = Some(ModalState {
                                        message: message.to_string(),
                                        show_until: Some(
                                            Instant::now()
                                                + std::time::Duration::from_millis(1500),
                                        ),
                                    });
                                }
                            }
                        }
                        "Security" => {
                            if let Some(security_tab) =
                                _tab.as_any_mut().downcast_mut::<crate::tabs::SecurityTab>()
                            {
                                if let Some(message) = security_tab.copy_selected_to_clipboard() {
                                    self.modal_state = Some(ModalState {
                                        message: message.to_string(),
                                        show_until: Some(
                                            Instant::now()
                                                + std::time::Duration::from_millis(1500),
                                        ),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Tab - переход на предыдущую вкладку
                    self.tab_manager.previous_tab();
                } else {
                    // Tab - переход на следующую вкладку
                    self.tab_manager.next_tab();
                }
            }
            KeyCode::BackTab => {
                // BackTab - это Shift+Tab
                self.tab_manager.previous_tab();
            }
            KeyCode::Char('t') => {
                // 't' - переход на следующую вкладку
                self.tab_manager.next_tab();
            }
            KeyCode::Char('T') if modifiers.contains(KeyModifiers::SHIFT) => {
                // Shift+T - переход на предыдущую вкладку
                self.tab_manager.previous_tab();
            }
            _ => {
                let idx = self.tab_manager.current_tab();
                if let Some(tab) = self.tabs.get_mut(idx) {
                    tab.handle_input(key_event);
                }
            }
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(60),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        // Улучшенное отображение вкладок
        frame.render_widget(
            TuiManager::new()
                .draw_tabs(
                    self.tab_manager
                        .tab_names()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    self.tab_manager.current_tab(),
                    "Navigation",
                )
                .style(HEADER_STYLE)
                .highlight_style(Style::new().fg(Color::White).bg(Color::Rgb(0, 95, 135))),
            header_chunks[0],
        );

        // Возвращаем использование draw_summary
        let (requests, ips, urls, update) = self.get_summary_text();

        // Создаем таблицу с разными цветами
        let summary_row = Row::new(vec![
            Cell::from(format!("Requests: {}", requests)).style(
                Style::new()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ), // Желтый
            Cell::from(format!("Unique IPs: {}", ips))
                .style(Style::new().fg(Color::Rgb(0, 255, 255))), // Голубой
            Cell::from(format!("Unique URLs: {}", urls))
                .style(Style::new().fg(Color::Rgb(255, 182, 193))), // Розовый
            Cell::from(format!("Update: {}", update))
                .style(Style::new().fg(Color::Rgb(144, 238, 144))), // Зеленый
        ]);

        frame.render_widget(
            Table::new(
                vec![summary_row],
                [
                    Constraint::Length(20), // Requests
                    Constraint::Length(20), // IPs
                    Constraint::Length(20), // URLs
                    Constraint::Min(30),    // Update
                ],
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Summary"),
            ),
            header_chunks[1],
        );

        // Улучшенный прогресс-бар
        draw_tui_progress_bar(frame, header_chunks[2], self.progress / 100.0, "Progress");

        // Рисуем активный таб
        let idx = self.tab_manager.current_tab();
        if let Some(tab) = self.tabs.get_mut(idx) {
            tab.draw(frame, chunks[1]);
        }

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

    fn get_summary_text(&mut self) -> (String, String, String, String) {
        // Кэшируем данные на 1 секунду
        let cache_duration = std::time::Duration::from_secs(1);
        let now = std::time::Instant::now();
        
        if let Some(cached) = &self.cached_summary {
            if now.duration_since(self.last_summary_update) < cache_duration {
                return cached.clone();
            }
        }
        
        let db = GLOBAL_DB.read().unwrap();
        let stats = db.get_stats();
        let now_time = chrono::Local::now();
        let result = (
            format!("{}", stats.total_records),
            format!("{}", stats.unique_ips),
            format!("{}", stats.unique_urls),
            format!("{}", now_time.format("%Y-%m-%d %H:%M:%S")),
        );
        
        // Обновляем кэш
        self.cached_summary = Some(result.clone());
        self.last_summary_update = now;
        
        result
    }

    fn draw_modal(&self, frame: &mut Frame) {
        if let Some(modal) = &self.modal_state {
            TuiManager::new().draw_modal(frame, &modal.message);
        }
    }
}
