use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Clear, Gauge, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Tabs, Wrap,
    },
    Frame,
};

/// Function to display simple progress bar in console
pub fn draw_simple_progress_bar(progress: f64) {
    let bar_length = 50;
    let filled_length = (progress * bar_length as f64) as usize;
    let empty_length = bar_length - filled_length;

    let filled = "█".repeat(filled_length);
    let empty = "░".repeat(empty_length);
    let percentage = (progress * 100.0) as usize;

    eprint!("\r[{}] {}%", filled + &empty, percentage);
}

/// Function to hide progress bar (clears the line)
pub fn hide_progress_bar() {
    eprint!("\r{}", " ".repeat(100)); // Clear the line
    eprintln!(); // New line
}

/// Function to draw progress bar in TUI interface
pub fn draw_tui_progress_bar(frame: &mut Frame, area: Rect, progress: f64, title: &str) {
    let progress_widget = Gauge::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .ratio(progress);

    frame.render_widget(progress_widget, area);
}

/// Universal tab manager that handles tab navigation and state
pub struct TuiManager;

// UI element styles
pub const HEADER_STYLE: Style = Style::new()
    .fg(Color::Rgb(144, 238, 144)) // Light green (softer)
    .add_modifier(Modifier::BOLD);

pub const SELECTED_ITEM_STYLE: Style = Style::new()
    .fg(Color::White)
    .bg(Color::Rgb(0, 95, 135)) // Dark blue background
    .add_modifier(Modifier::BOLD);

pub const PANEL_TITLE_STYLE: Style = Style::new()
    .fg(Color::Rgb(144, 238, 144)) // Light green (softer)
    .add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = Color::Rgb(158, 158, 158);

impl Default for TuiManager {
    fn default() -> Self {
        Self::new()
    }
}

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
        Paragraph::new(summary).block(Block::default().borders(Borders::ALL).title("Summary"))
    }

    /// Renders a modal window with a message
    pub fn draw_modal(&self, frame: &mut Frame, message: &str) {
        let area = frame.area();
        let popup_area = self.popup_area(area, 40, 20);

        // Clear the area under the popup
        frame.render_widget(Clear, popup_area);

        // Create vertical layout for popup content
        let chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Length(1), // Empty line for spacing
            Constraint::Length(3), // Main message
            Constraint::Length(1), // Empty line for spacing
            Constraint::Min(0),    // Additional message
        ])
        .spacing(0)
        .split(popup_area);

        // Draw the main popup block
        let block = Block::default()
            .title("Success")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Rgb(28, 28, 28)).fg(Color::White));

        frame.render_widget(block, popup_area);

        // Split message into lines
        let lines: Vec<&str> = message.split('\n').collect();

        // Draw icon and main message
        let icon = "✓";
        let message = format!("{} {}", icon, lines[0]);
        let paragraph = Paragraph::new(message)
            .style(
                Style::default()
                    .fg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            )
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

    pub fn draw_scrollbar(
        &self,
        count: usize,
        selected_index: usize,
        frame: &mut Frame,
        rect: Rect,
    ) {
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
}
