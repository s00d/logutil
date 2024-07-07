use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Sparkline, Tabs};
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::layout::{Rect};
use ratatui::Frame;
use ratatui::symbols::Marker;
use ratatui::text::Line;

pub struct TuiManager;

// pub const NORMAL_ROW_BG: Color = Color::Rgb(18, 18, 20);
pub const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(0, 31, 63)).add_modifier(Modifier::BOLD);
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

    pub fn draw_list<'a>(&self, items: Vec<ListItem<'a>>, title: String) -> List<'a> {
        List::new(items)
            .block(Block::default().borders(Borders::ALL).style(Style::default()).title(title))
            .highlight_style(SELECTED_STYLE)
            // .highlight_symbol(">")
    }

    pub fn draw_progress_bar(&self, progress: f64) -> Gauge {
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Loading Progress"))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::ITALIC))
            .ratio(progress)
    }

    pub fn draw_sparkline<'a>(&self, data: &'a [u64], title: &'a str) -> Sparkline<'a> {
        Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .data(data)
            .direction(ratatui::widgets::RenderDirection::RightToLeft)
            .style(Style::default().fg(Color::Cyan))
    }

    pub fn draw_pagination<'a>(&self, pages: Vec<String>, selected: usize) -> Tabs<'a> {
        Tabs::new(pages)
            .select(selected)
            .block(Block::default().borders(Borders::ALL).title("Pages"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|")
    }

    pub fn draw_heatmap<'a>(&self, cells: Vec<Rectangle>, x_labels: Vec<(f64, String)>, y_labels: Vec<(f64, String)>) -> Canvas<'a, impl Fn(&mut ratatui::widgets::canvas::Context) + 'a> {
        Canvas::default()
            .marker(Marker::HalfBlock)
            .block(Block::default().borders(Borders::ALL).title("Heatmap (hourly by date, UTC)"))
            .x_bounds([0.0, 25.5])  // 24 hours + space for labels
            .y_bounds([0.0, y_labels.len() as f64 + 1.0])  // Number of unique dates + space for labels
            .paint(move |ctx| {
                for label in &x_labels {
                    ctx.print(label.0, 0.0, Line::from(label.1.clone()));
                }

                for label in &y_labels {
                    ctx.print(0.0, label.0, Line::from(label.1.clone()));
                }

                for cell in &cells {
                    ctx.draw(cell);
                }
            })
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
}
