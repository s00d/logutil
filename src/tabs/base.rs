use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Trait for TUI tabs
pub trait Tab: Send + Sync + 'static {
    /// Draw the tab content
    fn draw(&mut self, frame: &mut Frame, area: Rect, log_data: &crate::log_data::LogData);

    /// Handle input for the tab
    fn handle_input(&mut self, key: KeyEvent, log_data: &crate::log_data::LogData) -> bool;

    /// Get mutable reference as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
