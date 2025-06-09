use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Sparkline, Tabs, Clear, Wrap, ListState};
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::layout::{Rect, Constraint, Layout, Direction};
use ratatui::Frame;
use ratatui::symbols::Marker;
use ratatui::text::{Line};
use std::time::SystemTime;
use chrono::{Local, TimeZone, Utc, Timelike};
use crate::log_data::LogEntry;

pub struct TuiManager;

// UI element styles
pub const HEADER_STYLE: Style = Style::new()
    .fg(Color::Rgb(144, 238, 144))  // Light green (softer)
    .add_modifier(Modifier::BOLD);

pub const ACTIVE_PANEL_STYLE: Style = Style::new()
    .fg(Color::White)
    .add_modifier(Modifier::BOLD);

pub const INACTIVE_PANEL_STYLE: Style = Style::new()
    .fg(Color::Rgb(169, 169, 169));  // Dark gray (softer)

pub const SELECTED_ITEM_STYLE: Style = Style::new()
    .fg(Color::White)
    .bg(Color::Rgb(0, 95, 135))  // Dark blue background
    .add_modifier(Modifier::BOLD);

pub const PANEL_TITLE_STYLE: Style = Style::new()
    .fg(Color::Rgb(144, 238, 144))  // Light green (softer)
    .add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = Color::Rgb(158, 158, 158);

impl TuiManager {
    pub fn new() -> Self {
        TuiManager
    }

    pub fn draw_tabs<'a>(&self, tabs: Vec<String>, selected: usize, title: &'a str) -> Tabs<'a> {
        Tabs::new(tabs)
            .select(selected)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|")
    }

    pub fn draw_summary<'a>(&self, summary: &'a str) -> Paragraph<'a> {
        Paragraph::new(summary)
            .block(Block::default().borders(Borders::ALL).title("Summary"))
    }

    // pub fn draw_table<'a>(&self, rows: Vec<Row<'a>>, headers: Vec<&'a str>, title: &'a str, constraints: &'a [Constraint]) -> Table<'a> {
    //     Table::new(rows, constraints)
    //         .block(Block::default().borders(Borders::ALL).title(title))
    //         .header(Row::new(headers).style(Style::default().fg(Color::Yellow)))
    // }

    pub fn draw_input<'a>(&self, input: &'a str) -> Paragraph<'a> {
        Paragraph::new(input)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"))
    }

    pub fn draw_progress_bar(&self, progress: f64) -> Gauge {
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Loading Progress"))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::ITALIC))
            .ratio(progress)
    }

    pub fn draw_sparkline<'a>(&self, data: &'a [u64], title: &'a str) -> Sparkline<'a> {
        // Разделяем заголовок на основную часть и статистику
        let parts: Vec<&str> = title.split(" (").collect();
        let main_title = parts[0];
        let stats = if parts.len() > 1 {
            format!("({}", parts[1])
        } else {
            String::new()
        };

        // Создаем блок с закругленными углами и стильным заголовком
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
            .title(format!("{} {}", main_title, stats))
            .title_alignment(ratatui::layout::Alignment::Center);

        // Настраиваем стиль графика
        let style = Style::new()
            .fg(Color::Rgb(0, 191, 255))  // Яркий голубой цвет для линии
            .bg(Color::Rgb(28, 28, 28));  // Темный фон

        Sparkline::default()
            .block(block)
            .data(data)
            .direction(ratatui::widgets::RenderDirection::RightToLeft)
            .style(style)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS)  // Используем 9 уровней для более плавного градиента
            .max(*data.iter().max().unwrap_or(&1))
    }

    /// Renders a sparkline graph of requests
    pub fn draw_requests_sparkline<'a>(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &[u64],
        min_value: u64,
        max_value: u64,
        start_time: i64,
        end_time: i64,
    ) {
        // Форматируем время для отображения
        let start_dt = Utc.timestamp_opt(start_time, 0).unwrap();
        let end_dt = Utc.timestamp_opt(end_time, 0).unwrap();
        let time_range = format!(
            "{} - {}",
            start_dt.format("%H:%M:%S"),
            end_dt.format("%H:%M:%S")
        );

        // Создаем информативный заголовок
        let sparkline_title = format!(
            "Requests Timeline | {} | Min: {} | Max: {} | Range: {}",
            time_range,
            min_value,
            max_value,
            end_dt.signed_duration_since(start_dt).num_seconds()
        );

        // Разделяем область на график и информационную панель
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ].as_ref())
            .split(area);

        // Рендерим график
        frame.render_widget(self.draw_sparkline(data, &sparkline_title), chunks[0]);

        // Добавляем информационную панель с дополнительной статистикой
        let stats_text = format!(
            "Total Requests: {} | Avg: {:.1} | Peak: {} | Current: {}",
            data.iter().sum::<u64>(),
            data.iter().sum::<u64>() as f64 / data.len() as f64,
            max_value,
            data.first().unwrap_or(&0)
        );

        frame.render_widget(
            Paragraph::new(stats_text)
                .style(Style::new()
                    .fg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))),
            chunks[1]
        );
    }

    /// Рендерит тепловую карту
    pub fn render_heatmap<'a>(
        &'a self,
        frame: &mut Frame,
        area: Rect,
        cells: Vec<Rectangle>,
        x_labels: Vec<(f64, String)>,
        y_labels: Vec<(f64, String)>,
        min_value: u64,
        max_value: u64,
    ) {
        // Разделяем область на основную часть и информационную панель
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ].as_ref())
            .split(area);

        // Рендерим основную тепловую карту
        frame.render_widget(self.draw_heatmap(cells, x_labels, y_labels), chunks[0]);

        // Отображаем информацию о значениях в отдельном виджете
        let info_text = format!(
            "Min: {} requests | Max: {} requests | Scale: Blue (Low) → Green → Red (High)",
            min_value,
            max_value
        );
        frame.render_widget(
            Paragraph::new(info_text)
                .style(Style::new()
                    .fg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))),
            chunks[1]
        );
    }

    /// Генерирует ячейки для тепловой карты
    pub fn generate_heatmap_cells(
        &self,
        sorted_data: &[(i64, u64)],
        min_value: u64,
        max_value: u64,
        unique_dates: &[chrono::NaiveDate],
    ) -> Vec<Rectangle> {
        let mut cells = Vec::new();

        for &(timestamp, value) in sorted_data.iter() {
            // Нормализуем значение от 0 до 1
            let normalized_value = if max_value == min_value {
                0.5 // Если все значения одинаковые, используем средний цвет
            } else {
                (value as f64 - min_value as f64) / (max_value as f64 - min_value as f64)
            };

            let (r, g, b) = self.get_heatmap_color(normalized_value);

            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
            let hour = datetime.hour() as f64;
            let date_index = unique_dates.iter().position(|&d| d == datetime.date_naive()).unwrap() as f64;

            // Добавляем тень для эффекта глубины
            cells.push(Rectangle {
                x: hour + 1.4,
                y: date_index + 1.0,
                width: 0.8,
                height: 0.75,
                color: Color::Rgb(20, 20, 20),
            });

            // Основная ячейка
            cells.push(Rectangle {
                x: hour + 1.3,
                y: date_index + 0.9,
                width: 0.8,
                height: 0.75,
                color: Color::Rgb(r, g, b),
            });
        }

        cells
    }

    /// Gets bounds for the sparkline graph
    pub fn get_sparkline_bounds(&self, data: &[u64], sorted_data: &[(i64, u64)]) -> (u64, u64, i64, i64) {
        let min_value = *data.iter().min().unwrap_or(&0);
        let max_value = *data.iter().max().unwrap_or(&0);
        let start_time = sorted_data.last().map(|&(k, _)| k).unwrap_or(0);
        let end_time = sorted_data.first().map(|&(k, _)| k).unwrap_or(0);
        (min_value, max_value, start_time, end_time)
    }

    pub fn draw_scrollbar(&self, count: usize, selected_index: usize, frame: &mut Frame, rect: Rect) {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(count)
            .position(selected_index);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            rect,
            &mut scrollbar_state,
        );
    }

    /// Renders a modal window with a message
    pub fn draw_modal<'a>(&self, frame: &mut Frame, message: &'a str) {
        let area = frame.area();
        let popup_area = self.popup_area(area, 40, 20);

        // Clear the area under the popup
        frame.render_widget(Clear, popup_area);

        // Create vertical layout for popup content
        let chunks = Layout::vertical([
            Constraint::Length(3),  // Header
            Constraint::Length(1),  // Empty line for spacing
            Constraint::Length(3),  // Main message
            Constraint::Length(1),  // Empty line for spacing
            Constraint::Min(0),     // Additional message
        ]).spacing(0).split(popup_area);

        // Draw the main popup block
        let block = Block::default()
            .title("Success")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default()
                .bg(Color::Rgb(28, 28, 28))
                .fg(Color::White));

        frame.render_widget(block, popup_area);

        // Split message into lines
        let lines: Vec<&str> = message.split('\n').collect();
        
        // Draw icon and main message
        let icon = "✓";
        let message = format!("{} {}", icon, lines[0]);
        let paragraph = Paragraph::new(message)
            .style(Style::default()
                .fg(Color::Rgb(144, 238, 144))
                .add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, chunks[2]);

        // Draw additional message (if any)
        if lines.len() > 1 {
            let submessage = Paragraph::new(lines[1])
                .style(Style::default().fg(Color::Rgb(200, 200, 200)))
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(submessage, chunks[4]);
        }
    }

    /// Helper function to create a centered rectangle
    fn popup_area(&self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let popup_width = (area.width as f32 * (percent_x as f32 / 100.0)) as u16;
        let popup_height = (area.height as f32 * (percent_y as f32 / 100.0)) as u16;
        
        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        
        Rect::new(x, y, popup_width, popup_height)
    }

    /// Truncates URL with ellipsis if it's too long
    fn truncate_url(&self, url: &str, max_length: usize) -> String {
        if url.len() <= max_length {
            return url.to_string();
        }
        format!("{}...", &url[..max_length-3])
    }

    /// Renders the overview panel with IP and URL lists
    pub fn draw_overview<'a>(
        &self,
        frame: &mut Frame,
        area: Rect,
        ip_items: Vec<ListItem<'a>>,
        url_items: Vec<(ListItem<'a>, &'a LogEntry)>,
        _overview_panel: usize,
        ip_list_state: &mut ListState,
        url_list_state: &mut ListState,
    ) {
        // Разделяем область на основную часть и панель для полного URL
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3) // Высота панели для полного URL
            ].as_ref())
            .split(area);

        // Разделяем основную часть на две колонки
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ].as_ref())
            .split(chunks[0]);

        // Добавляем заголовки в начало списков
        let mut ip_items_with_header = vec![self.format_ip_header()];
        ip_items_with_header.extend(ip_items);

        let mut url_items_with_header: Vec<(ListItem<'a>, Option<&'a LogEntry>)> = vec![(self.format_url_header(), None)];
        url_items_with_header.extend(url_items.into_iter().map(|(item, entry)| (item, Some(entry))));

        // Корректируем выделение для IP списка, учитывая заголовок
        let ip_selected = ip_list_state.selected().map(|idx| idx + 1);
        let mut adjusted_ip_state = ListState::default();
        if let Some(idx) = ip_selected {
            adjusted_ip_state.select(Some(idx));
        }

        // Корректируем выделение для URL списка, учитывая заголовок
        let url_selected = url_list_state.selected().map(|idx| idx + 1);
        let mut adjusted_url_state = ListState::default();
        if let Some(idx) = url_selected {
            adjusted_url_state.select(Some(idx));
        }

        // Draw IP list
        frame.render_stateful_widget(
            List::new(ip_items_with_header.clone())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("IP List")
                    .title_style(Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)))
                .highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[0],
            &mut adjusted_ip_state
        );

        // Draw URL list
        frame.render_stateful_widget(
            List::new(url_items_with_header.iter().map(|(item, _)| item.clone()).collect::<Vec<ListItem>>())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("URL List")
                    .title_style(Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)))
                .highlight_style(SELECTED_ITEM_STYLE),
            main_chunks[1],
            &mut adjusted_url_state
        );

        // Draw scrollbars
        self.draw_scrollbar(ip_items_with_header.len(), adjusted_ip_state.selected().unwrap_or(0), frame, main_chunks[0]);
        self.draw_scrollbar(url_items_with_header.len(), adjusted_url_state.selected().unwrap_or(0), frame, main_chunks[1]);

        // Отображаем полный URL в нижней панели, если URL выбран
        if let Some(idx) = adjusted_url_state.selected() {
            if idx > 0 { // Пропускаем заголовок
                if let Some((_, entry_opt)) = url_items_with_header.get(idx) {
                    if let Some(entry) = entry_opt {
                        frame.render_widget(
                            Paragraph::new(entry.full_url.as_str())
                                .block(Block::default()
                                    .borders(Borders::ALL)
                                    .border_type(ratatui::widgets::BorderType::Rounded)
                                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                                    .title("Full URL"))
                                .style(Style::new().fg(Color::White)),
                            chunks[1]
                        );
                    }
                }
            }
        }

        // Обновляем оригинальные состояния
        if let Some(idx) = adjusted_ip_state.selected() {
            ip_list_state.select(Some(idx - 1));
        }
        if let Some(idx) = adjusted_url_state.selected() {
            url_list_state.select(Some(idx - 1));
        }
    }

    /// Renders the last requests panel with search and pagination
    pub fn draw_last_requests<'a>(
        &self,
        frame: &mut Frame,
        area: Rect,
        items: Vec<ListItem<'a>>,
        input: &str,
        current_page: usize,
        total_pages: usize,
        list_state: &mut ListState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0)
            ].as_ref())
            .split(area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .split(chunks[0]);

        // Search field
        frame.render_widget(
            self.draw_input(input)
                .style(Style::new().fg(Color::White))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Search")
                    .title_style(PANEL_TITLE_STYLE)),
            header_chunks[0]
        );

        // Pagination
        let pages: Vec<String> = (1..=total_pages).map(|i| format!("{}", i)).collect();
        frame.render_widget(
            self.draw_pagination(pages, current_page)
                .style(HEADER_STYLE)
                .highlight_style(Style::new().fg(Color::White).bg(Color::Rgb(0, 95, 135))),
            header_chunks[1]
        );

        // Request list
        frame.render_stateful_widget(
            List::new(items.clone())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Requests"))
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            list_state
        );

        self.draw_scrollbar(items.len(), list_state.selected().unwrap_or(0), frame, chunks[1]);
    }

    /// Renders the detailed requests panel
    pub fn draw_detailed_requests<'a>(
        &self,
        frame: &mut Frame,
        area: Rect,
        ip_items: Vec<ListItem<'a>>,
        request_items: Vec<ListItem<'a>>,
        selected_ip: Option<String>,
        ip_list_state: &mut ListState,
        request_list_state: &mut ListState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ].as_ref())
            .split(area);

        // Добавляем заголовки в начало списков
        let mut ip_items_with_header = vec![self.format_ip_header()];
        ip_items_with_header.extend(ip_items);

        let mut request_items_with_header = vec![];
        let has_ip_header = selected_ip.is_some();
        if let Some(ref ip) = selected_ip {
            request_items_with_header.push(ListItem::new(format!("Requests for IP: {}", ip))
                .style(PANEL_TITLE_STYLE));
        }
        request_items_with_header.extend(request_items);

        // Корректируем выделение для IP списка, учитывая заголовок
        let ip_selected = ip_list_state.selected().map(|idx| idx + 1);
        let mut adjusted_ip_state = ListState::default();
        if let Some(idx) = ip_selected {
            adjusted_ip_state.select(Some(idx));
        }

        // Корректируем выделение для списка запросов, учитывая заголовок
        let request_selected = request_list_state.selected().map(|idx| idx + if has_ip_header { 1 } else { 0 });
        let mut adjusted_request_state = ListState::default();
        if let Some(idx) = request_selected {
            adjusted_request_state.select(Some(idx));
        }

        // Draw IP list
        frame.render_stateful_widget(
            List::new(ip_items_with_header.clone())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("IP List")
                    .title_style(Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)))
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[0],
            &mut adjusted_ip_state
        );

        // Draw request list
        frame.render_stateful_widget(
            List::new(request_items_with_header.clone())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                    .title("Request Details")
                    .title_style(Style::new().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)))
                .highlight_style(SELECTED_ITEM_STYLE),
            chunks[1],
            &mut adjusted_request_state
        );

        // Draw scrollbars
        self.draw_scrollbar(ip_items_with_header.len(), adjusted_ip_state.selected().unwrap_or(0), frame, chunks[0]);
        self.draw_scrollbar(request_items_with_header.len(), adjusted_request_state.selected().unwrap_or(0), frame, chunks[1]);

        // Обновляем оригинальные состояния
        if let Some(idx) = adjusted_ip_state.selected() {
            ip_list_state.select(Some(idx - 1));
        }
        if let Some(idx) = adjusted_request_state.selected() {
            request_list_state.select(Some(idx - if selected_ip.is_some() { 1 } else { 0 }));
        }
    }

    /// Formats an IP list item
    pub fn format_ip_item<'a>(&self, ip: &str, entry: &LogEntry, is_active: bool) -> ListItem<'a> {
        let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
        let style = if is_active { ACTIVE_PANEL_STYLE } else { INACTIVE_PANEL_STYLE };
        ListItem::new(format!("{:<15} │ {:<12} │ {}", ip, entry.count, last_update_str))
            .style(style)
    }

    /// Formats a URL list item
    pub fn format_url_item<'a>(&self, url: &str, entry: &LogEntry, is_active: bool) -> ListItem<'a> {
        let last_update = entry.last_update.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let last_update_str = format!("{}", Local.timestamp_opt(last_update as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
        let style = if is_active { ACTIVE_PANEL_STYLE } else { INACTIVE_PANEL_STYLE };
        
        // Обрезаем URL если он слишком длинный
        let truncated_url = self.truncate_url(url, 25);
        
        ListItem::new(format!("{:<25} │ {:<20} │ {:<10} │ {:<12} │ {}", 
            truncated_url, 
            entry.request_type, 
            entry.request_domain, 
            entry.count, 
            last_update_str))
            .style(style)
    }

    /// Formats the IP list header
    pub fn format_ip_header<'a>(&self) -> ListItem<'a> {
        ListItem::new(format!("{:<15} │ {:<12} │ {}", "IP", "Requests", "Last Update"))
            .style(Style::new().fg(Color::Rgb(0, 191, 255)).add_modifier(Modifier::BOLD))
    }

    /// Formats the URL list header
    pub fn format_url_header<'a>(&self) -> ListItem<'a> {
        ListItem::new(format!("{:<25} │ {:<20} │ {:<10} │ {:<12} │ {}",
            "URL", "Type", "Domain", "Requests", "Last Update"))
            .style(Style::new().fg(Color::Rgb(0, 191, 255)).add_modifier(Modifier::BOLD))
    }

    /// Генерирует цвет для тепловой карты на основе нормализованного значения
    fn get_heatmap_color(&self, normalized_value: f64) -> (u8, u8, u8) {
        if normalized_value < 0.5 {
            // От синего к зеленому
            let t = normalized_value * 2.0;
            (
                0,
                (t * 255.0) as u8,
                ((1.0 - t) * 255.0) as u8,
            )
        } else {
            // От зеленого к красному
            let t = (normalized_value - 0.5) * 2.0;
            (
                (t * 255.0) as u8,
                ((1.0 - t) * 255.0) as u8,
                0,
            )
        }
    }

    pub fn draw_pagination<'a>(&self, pages: Vec<String>, selected: usize) -> Tabs<'a> {
        Tabs::new(pages)
            .select(selected)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                .title("Pages"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|")
    }

    pub fn draw_heatmap<'a>(
        &'a self,
        cells: Vec<Rectangle>,
        x_labels: Vec<(f64, String)>,
        y_labels: Vec<(f64, String)>,
    ) -> Canvas<'a, impl Fn(&mut ratatui::widgets::canvas::Context) + 'a> {
        Canvas::default()
            .marker(ratatui::symbols::Marker::HalfBlock)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::new().fg(Color::Rgb(144, 238, 144)))
                .title("Heatmap (Requests by Hour)")
                .title_alignment(ratatui::layout::Alignment::Center))
            .x_bounds([0.0, 25.5])  // 24 hours + space for labels
            .y_bounds([0.0, y_labels.len() as f64 + 1.0])  // Number of unique dates + space for labels
            .paint(move |ctx| {
                // Рисуем сетку
                for hour in 0..24 {
                    let x = hour as f64 + 1.3;
                    ctx.draw(&Rectangle {
                        x,
                        y: 0.0,
                        width: 0.8,
                        height: y_labels.len() as f64 + 1.0,
                        color: Color::Rgb(40, 40, 40),
                    });
                }

                // Рисуем подписи часов
                for label in &x_labels {
                    ctx.print(label.0, 0.0, ratatui::text::Line::from(label.1.clone())
                        .style(Style::new()
                            .fg(Color::Rgb(144, 238, 144))
                            .add_modifier(Modifier::BOLD)));
                }

                // Рисуем подписи дат
                for label in &y_labels {
                    ctx.print(0.0, label.0, ratatui::text::Line::from(label.1.clone())
                        .style(Style::new()
                            .fg(Color::Rgb(144, 238, 144))
                            .add_modifier(Modifier::BOLD)));
                }

                // Рисуем ячейки тепловой карты
                for cell in &cells {
                    ctx.draw(cell);
                }

                // Добавляем легенду
                let legend_y = y_labels.len() as f64 + 1.5;
                let legend_text = "Legend: Low → High";
                ctx.print(1.0, legend_y, ratatui::text::Line::from(legend_text)
                    .style(Style::new()
                        .fg(Color::Rgb(144, 238, 144))
                        .add_modifier(Modifier::BOLD)));

                // Рисуем градиент легенды
                for i in 0..20 {
                    let x = 20.0 + i as f64 * 0.3;
                    let normalized_value = i as f64 / 19.0;
                    let (r, g, b) = self.get_heatmap_color(normalized_value);
                    ctx.draw(&Rectangle {
                        x,
                        y: legend_y,
                        width: 0.3,
                        height: 0.5,
                        color: Color::Rgb(r, g, b),
                    });
                }
            })
    }
}
