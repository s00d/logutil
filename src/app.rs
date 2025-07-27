use crate::log_data::LogData;
use crate::tab_manager::TabManager;
use crate::tabs::base::Tab;
use crate::tabs::{
    BotsTab, DetailedTab, ErrorsTab, HeatmapTab, OverviewTab, PerformanceTab, RequestsTab,
    SecurityTab, SparklineTab,
};
use crate::tui_manager::{draw_tui_progress_bar, TuiManager, HEADER_STYLE};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;

/// Конфигурация для создания App
#[derive(Debug)]
pub struct AppConfig {
    pub log_data: Arc<StdMutex<LogData>>,
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
    log_data: Arc<StdMutex<LogData>>,
    pub(crate) should_quit: bool,
    tab_manager: TabManager,
    tabs: Vec<Box<dyn Tab>>,
    progress: f64,
    modal_state: Option<ModalState>,
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
            log_data: config.log_data,
            should_quit: false,
            tab_manager: TabManager::new(tab_names),
            tabs,
            progress: 0.0,
            modal_state: None,
        }
    }

    pub(crate) fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 100.0);
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
                            if let Ok(log_data_guard) = self.log_data.try_lock() {
                                if let Some(overview_tab) =
                                    _tab.as_any_mut().downcast_mut::<crate::tabs::OverviewTab>()
                                {
                                    if let Some(message) =
                                        overview_tab.copy_selected_to_clipboard(&log_data_guard)
                                    {
                                        self.modal_state = Some(ModalState {
                                            message,
                                            show_until: Some(
                                                Instant::now()
                                                    + std::time::Duration::from_millis(1500),
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                        "Requests" => {
                            if let Ok(log_data_guard) = self.log_data.try_lock() {
                                if let Some(requests_tab) =
                                    _tab.as_any_mut().downcast_mut::<crate::tabs::RequestsTab>()
                                {
                                    if let Some(message) =
                                        requests_tab.copy_selected_to_clipboard(&log_data_guard)
                                    {
                                        self.modal_state = Some(ModalState {
                                            message,
                                            show_until: Some(
                                                Instant::now()
                                                    + std::time::Duration::from_millis(1500),
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                        "Detailed" => {
                            if let Ok(log_data_guard) = self.log_data.try_lock() {
                                if let Some(detailed_tab) =
                                    _tab.as_any_mut().downcast_mut::<crate::tabs::DetailedTab>()
                                {
                                    if let Some(message) =
                                        detailed_tab.copy_selected_to_clipboard(&log_data_guard)
                                    {
                                        self.modal_state = Some(ModalState {
                                            message,
                                            show_until: Some(
                                                Instant::now()
                                                    + std::time::Duration::from_millis(1500),
                                            ),
                                        });
                                    }
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
                    if let Ok(log_data_guard) = self.log_data.try_lock() {
                        tab.handle_input(key_event, &log_data_guard);
                    }
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
        frame.render_widget(
            TuiManager::new()
                .draw_summary(&self.get_summary_text())
                .style(HEADER_STYLE)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(144, 238, 144))),
                ),
            header_chunks[1],
        );

        // Улучшенный прогресс-бар
        draw_tui_progress_bar(frame, header_chunks[2], self.progress / 100.0, "Progress");

        // Рисуем активный таб
        let idx = self.tab_manager.current_tab();
        if let Some(tab) = self.tabs.get_mut(idx) {
            if let Ok(log_data_guard) = self.log_data.try_lock() {
                tab.draw(frame, chunks[1], &log_data_guard);
            }
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

    fn get_summary_text(&self) -> String {
        let log_data = self
            .log_data
            .lock()
            .expect("Failed to acquire log data lock for summary");
        let (unique_ips, unique_urls) = log_data.get_unique_counts();
        let now = chrono::Local::now();
        format!(
            "Requests: {} | Unique IPs: {} | Unique URLs: {} | Update: {}",
            log_data.total_requests,
            unique_ips,
            unique_urls,
            now.format("%Y-%m-%d %H:%M:%S")
        )
    }

    fn draw_modal(&self, frame: &mut Frame) {
        if let Some(modal) = &self.modal_state {
            TuiManager::new().draw_modal(frame, &modal.message);
        }
    }
}
