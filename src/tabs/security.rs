use crate::log_data::LogData;
use crate::tui_manager::{HEADER_STYLE, SELECTED_ITEM_STYLE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct SecurityTab {
    table_state: TableState,
    log_detail_state: ListState,
    show_log_detail: bool,
    input: String,
    active_panel: usize, // 0 = left panel (IPs), 1 = right panel (logs)
}

impl SecurityTab {
    pub fn new() -> Self {
        let mut instance = Self {
            table_state: TableState::default(),
            log_detail_state: ListState::default(),
            show_log_detail: false,
            input: String::new(),
            active_panel: 0, // Начинаем с левой панели
        };
        
        // Инициализируем выделение для таблицы
        instance.table_state.select(Some(0));
        
        instance
    }

    fn draw_security_tab(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let (suspicious_count, attack_patterns_count, rate_limit_count) =
            log_data.get_security_summary();
        let top_suspicious = log_data.get_top_suspicious_ips();

        if self.show_log_detail {
            self.draw_log_detail_view(frame, area, log_data);
        } else {
            self.draw_main_security_view(
                frame,
                area,
                log_data,
                suspicious_count,
                attack_patterns_count,
                rate_limit_count,
                &top_suspicious,
            );
        }
    }

    fn draw_main_security_view(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        log_data: &LogData,
        suspicious_count: usize,
        attack_patterns_count: usize,
        rate_limit_count: usize,
        top_suspicious: &[(String, usize)],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4), // Увеличиваем высоту для дополнительной информации
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        // Расширенная Security summary с дополнительными детектами
        let additional_detections = self.get_additional_security_detections(log_data);
        let summary_text = format!(
            "Suspicious IPs: {} | Attack Patterns: {} | Rate Limit Violations: {} | {}",
            suspicious_count, attack_patterns_count, rate_limit_count, additional_detections
        );

        frame.render_widget(
            Paragraph::new(summary_text).style(HEADER_STYLE).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(255, 0, 0)))
                    .title("Security Overview"),
            ),
            chunks[0],
        );

        // Разделяем основную область на две панели
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        // Левая панель - список подозрительных IP
        self.draw_suspicious_ips_list(frame, main_chunks[0], top_suspicious, log_data);

        // Правая панель - детали логов или поиск
        self.draw_log_details_panel(frame, main_chunks[1], log_data);
    }

    fn draw_suspicious_ips_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        top_suspicious: &[(String, usize)],
        log_data: &LogData,
    ) {
        // Фильтруем результаты поиска
        let filtered_suspicious = if self.input.is_empty() {
            top_suspicious.to_owned()
        } else {
            top_suspicious
                .iter()
                .filter(|(ip, _)| ip.to_lowercase().contains(&self.input.to_lowercase()))
                .cloned()
                .collect()
        };

        let rows: Vec<Row> = filtered_suspicious
            .iter()
            .map(|(ip, count)| {
                let patterns = log_data.get_suspicious_patterns_for_ip(ip);
                let threat_level = self.get_threat_level(ip, count, &patterns);
                let pattern_text = if patterns.is_empty() {
                    "Suspicious Activity".to_string()
                } else {
                    format!("Patterns: {}", patterns.join(", "))
                };
                let threat_icon = match threat_level {
                    "HIGH" => "🔴",
                    "MEDIUM" => "🟡",
                    "LOW" => "🟢",
                    _ => "⚪",
                };
                Row::new(vec![
                    Cell::from(threat_icon),
                    Cell::from(ip.to_string()).style(Style::new().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)), // IP - желтый, жирный
                    Cell::from(count.to_string()).style(Style::new().fg(Color::Rgb(0, 255, 255))), // Count - голубой
                    Cell::from(threat_level.to_string()).style(Style::new().fg(Color::Rgb(255, 182, 193))), // Threat - розовый
                    Cell::from(pattern_text).style(Style::new().fg(Color::Rgb(144, 238, 144))), // Patterns - зеленый
                ])
            })
            .collect();

        // Создаем заголовок для таблицы
        let header = Row::new(vec![
            Cell::from("Level").style(Style::new().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)),
            Cell::from("IP").style(Style::new().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)),
            Cell::from("Count").style(Style::new().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
            Cell::from("Threat").style(Style::new().fg(Color::Rgb(255, 182, 193)).add_modifier(Modifier::BOLD)),
            Cell::from("Patterns").style(Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)),
        ]).style(
            Style::new()
                .fg(Color::Rgb(0, 191, 255))
                .add_modifier(Modifier::BOLD)
        );

        let border_style = if self.active_panel == 0 && !self.show_log_detail {
            Style::new().fg(Color::Rgb(255, 255, 0)) // Желтый для активной панели
        } else {
            Style::new().fg(Color::Rgb(255, 0, 0)) // Красный для неактивной
        };

        frame.render_stateful_widget(
            Table::new(rows, [
                Constraint::Length(4),   // Level (icon)
                Constraint::Length(15),  // IP
                Constraint::Length(10),  // Count
                Constraint::Length(8),   // Threat
                Constraint::Min(20),     // Patterns
            ])
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(border_style)
                    .title(format!(
                        "Suspicious IPs - Total: {}",
                        filtered_suspicious.len()
                    )),
            )
            .row_highlight_style(SELECTED_ITEM_STYLE),
            area,
            &mut self.table_state,
        );
    }

    fn draw_log_details_panel(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let border_style = if self.active_panel == 1 && !self.show_log_detail {
            Style::new().fg(Color::Rgb(255, 255, 0)) // Желтый для активной панели
        } else {
            Style::new().fg(Color::Rgb(255, 0, 0)) // Красный для неактивной
        };

        // Поисковая строка
        frame.render_widget(
            Paragraph::new(format!("Search: {}", self.input))
                .style(HEADER_STYLE)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(border_style)
                        .title("Search IPs"),
                ),
            chunks[0],
        );

        // Детали логов для выбранного IP
        if let Some(selected_ip) = self.get_selected_ip(log_data) {
            let log_lines = self.get_highlighted_log_lines(log_data, &selected_ip);
            let items: Vec<ListItem> = log_lines
                .iter()
                .map(|line| {
                    ListItem::new(line.clone()).style(Style::new().fg(Color::Rgb(255, 255, 255)))
                })
                .collect();

            frame.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(border_style)
                            .title(format!("Logs for IP: {}", selected_ip)),
                    )
                    .highlight_style(SELECTED_ITEM_STYLE),
                chunks[1],
                &mut self.log_detail_state,
            );
        } else {
            frame.render_widget(
                Paragraph::new("Select an IP to view log details").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(border_style)
                        .title("Log Details"),
                ),
                chunks[1],
            );
        }
    }

    fn draw_log_detail_view(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        // Header with navigation
        frame.render_widget(
            Paragraph::new("Log Detail View (Press 'q' to return)")
                .style(HEADER_STYLE)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::new().fg(Color::Rgb(255, 0, 0)))
                        .title("Security Log Analysis"),
                ),
            chunks[0],
        );

        // Log detail content
        if let Some(selected_ip) = self.get_selected_ip(log_data) {
            let log_lines = self.get_highlighted_log_lines(log_data, &selected_ip);
            let items: Vec<ListItem> = log_lines
                .iter()
                .map(|line| {
                    ListItem::new(line.clone()).style(Style::new().fg(Color::Rgb(255, 255, 255)))
                })
                .collect();

            frame.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded)
                            .border_style(Style::new().fg(Color::Rgb(255, 0, 0)))
                            .title(format!("Logs for IP: {}", selected_ip)),
                    )
                    .highlight_style(SELECTED_ITEM_STYLE),
                chunks[1],
                &mut self.log_detail_state,
            );
        }
    }

    fn get_additional_security_detections(&self, log_data: &LogData) -> String {
        let mut detections = Vec::new();

        // Детект SQL Injection
        let sql_injection_count = self.detect_sql_injection(log_data);
        if sql_injection_count > 0 {
            detections.push(format!("SQL Injection: {}", sql_injection_count));
        }

        // Детект XSS
        let xss_count = self.detect_xss(log_data);
        if xss_count > 0 {
            detections.push(format!("XSS: {}", xss_count));
        }

        // Детект Path Traversal
        let path_traversal_count = self.detect_path_traversal(log_data);
        if path_traversal_count > 0 {
            detections.push(format!("Path Traversal: {}", path_traversal_count));
        }

        // Детект Command Injection
        let cmd_injection_count = self.detect_command_injection(log_data);
        if cmd_injection_count > 0 {
            detections.push(format!("Command Injection: {}", cmd_injection_count));
        }

        // Детект Brute Force
        let brute_force_count = self.detect_brute_force(log_data);
        if brute_force_count > 0 {
            detections.push(format!("Brute Force: {}", brute_force_count));
        }

        if detections.is_empty() {
            "No additional threats detected".to_string()
        } else {
            detections.join(" | ")
        }
    }

    fn detect_sql_injection(&self, log_data: &LogData) -> usize {
        let sql_patterns = [
            "'", "union", "select", "drop", "insert", "update", "delete", "exec", "xp_",
        ];
        self.count_patterns_in_logs(log_data, &sql_patterns)
    }

    fn detect_xss(&self, log_data: &LogData) -> usize {
        let xss_patterns = [
            "<script>",
            "javascript:",
            "onload=",
            "onerror=",
            "onclick=",
            "alert(",
            "document.cookie",
        ];
        self.count_patterns_in_logs(log_data, &xss_patterns)
    }

    fn detect_path_traversal(&self, log_data: &LogData) -> usize {
        let path_patterns = ["../", "..\\", "/etc/", "/proc/", "c:\\", "windows\\"];
        self.count_patterns_in_logs(log_data, &path_patterns)
    }

    fn detect_command_injection(&self, log_data: &LogData) -> usize {
        let cmd_patterns = [";", "|", "&", "`", "$(", "eval(", "system(", "exec("];
        self.count_patterns_in_logs(log_data, &cmd_patterns)
    }

    fn detect_brute_force(&self, log_data: &LogData) -> usize {
        // Подсчитываем IP с большим количеством запросов к auth endpoints
        let auth_patterns = ["/login", "/auth", "/admin", "/wp-admin"];
        let mut brute_force_count = 0;

        for entry in log_data.by_ip.values() {
            let auth_requests = entry
                .last_requests
                .iter()
                .filter(|req| auth_patterns.iter().any(|pattern| req.contains(pattern)))
                .count();
            if auth_requests > 10 {
                brute_force_count += 1;
            }
        }

        brute_force_count
    }

    fn count_patterns_in_logs(&self, log_data: &LogData, patterns: &[&str]) -> usize {
        let mut count = 0;
        for entry in log_data.by_ip.values() {
            for request in &entry.last_requests {
                if patterns
                    .iter()
                    .any(|pattern| request.to_lowercase().contains(pattern))
                {
                    count += 1;
                }
            }
        }
        count
    }

    fn get_threat_level(&self, _ip: &str, count: &usize, patterns: &[String]) -> &'static str {
        if *count > 100 || patterns.len() > 3 {
            "HIGH"
        } else if *count > 50 || patterns.len() > 1 {
            "MEDIUM"
        } else {
            "LOW"
        }
    }

    fn get_selected_ip(&self, log_data: &LogData) -> Option<String> {
        if let Some(selected) = self.table_state.selected() {
            let top_suspicious = log_data.get_top_suspicious_ips();
            let filtered_suspicious = if self.input.is_empty() {
                top_suspicious
            } else {
                top_suspicious
                    .iter()
                    .filter(|(ip, _)| ip.to_lowercase().contains(&self.input.to_lowercase()))
                    .cloned()
                    .collect()
            };

            filtered_suspicious.get(selected).map(|(ip, _)| ip.clone())
        } else {
            None
        }
    }

    fn get_highlighted_log_lines(&self, log_data: &LogData, ip: &str) -> Vec<String> {
        let mut highlighted_lines = Vec::new();

        if let Some(entry) = log_data.by_ip.get(ip) {
            for request in &entry.last_requests {
                let highlighted = self.highlight_suspicious_patterns(request);
                highlighted_lines.push(highlighted);
            }
        }

        highlighted_lines
    }

    fn highlight_suspicious_patterns(&self, log_line: &str) -> String {
        let suspicious_patterns = [
            ("'", "🔴"),
            ("union", "🔴"),
            ("select", "🔴"),
            ("<script>", "🟡"),
            ("javascript:", "🟡"),
            ("../", "🟡"),
            (";", "🟡"),
            ("|", "🟡"),
            ("admin", "🟡"),
            ("login", "🟡"),
        ];

        let mut highlighted = log_line.to_string();
        for (pattern, icon) in suspicious_patterns {
            if highlighted.to_lowercase().contains(pattern) {
                highlighted =
                    highlighted.replace(pattern, &format!("{}[{}]{}", icon, pattern, icon));
            }
        }

        highlighted
    }
}

impl Default for SecurityTab {
    fn default() -> Self {
        Self::new()
    }
}

impl super::base::Tab for SecurityTab {
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &LogData) {
        self.draw_security_tab(frame, area, log_data);
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent, log_data: &LogData) -> bool {
        match key.code {
            crossterm::event::KeyCode::Up => {
                if self.show_log_detail {
                    self.log_detail_state.select_previous();
                } else if self.active_panel == 0 {
                    if let Some(selected) = self.table_state.selected() {
                        if selected > 0 {
                            self.table_state.select(Some(selected - 1));
                        }
                    }
                } else {
                    self.log_detail_state.select_previous();
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                if self.show_log_detail {
                    self.log_detail_state.select_next();
                } else if self.active_panel == 0 {
                    if let Some(selected) = self.table_state.selected() {
                        // Получаем количество подозрительных IP для определения максимального индекса
                        let top_suspicious = log_data.get_top_suspicious_ips();
                        let filtered_suspicious = if self.input.is_empty() {
                            top_suspicious.to_owned()
                        } else {
                            top_suspicious
                                .iter()
                                .filter(|(ip, _)| ip.to_lowercase().contains(&self.input.to_lowercase()))
                                .cloned()
                                .collect()
                        };
                        
                        if selected < filtered_suspicious.len().saturating_sub(1) {
                            self.table_state.select(Some(selected + 1));
                        }
                    }
                } else {
                    self.log_detail_state.select_next();
                }
                true
            }
            crossterm::event::KeyCode::Left => {
                if self.active_panel > 0 {
                    self.active_panel -= 1;
                }
                true
            }
            crossterm::event::KeyCode::Right => {
                if self.active_panel < 1 {
                    self.active_panel += 1;
                }
                true
            }
            crossterm::event::KeyCode::Enter => {
                if !self.show_log_detail {
                    self.show_log_detail = true;
                    self.log_detail_state.select(Some(0));
                }
                true
            }
            crossterm::event::KeyCode::Esc => {
                if self.show_log_detail {
                    self.show_log_detail = false;
                }
                true
            }
            crossterm::event::KeyCode::Backspace => {
                if self.active_panel == 0 {
                    self.table_state.select(None);
                    self.input.pop();
                }
                true
            }
            crossterm::event::KeyCode::Char(c) => {
                if self.active_panel == 0 {
                    self.table_state.select(None);
                    self.input.push(c);
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
