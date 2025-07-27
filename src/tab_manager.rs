/// Universal tab manager that handles tab navigation and state
#[derive(Default)]
pub struct TabManager {
    tab_names: Vec<String>,
    current_tab: usize,
}

impl TabManager {
    /// Create a new tab manager with the given tabs
    pub fn new(tab_names: Vec<String>) -> Self {
        Self {
            tab_names,
            current_tab: 0,
        }
    }

    /// Get the current tab index
    pub fn current_tab(&self) -> usize {
        self.current_tab
    }

    /// Get the current tab name
    pub fn current_tab_name(&self) -> Option<&str> {
        self.tab_names
            .get(self.current_tab)
            .map(|name| name.as_str())
    }

    /// Get all tab names
    pub fn tab_names(&self) -> &[String] {
        &self.tab_names
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        if !self.tab_names.is_empty() {
            self.current_tab = (self.current_tab + 1) % self.tab_names.len();
        }
    }

    /// Switch to the previous tab
    pub fn previous_tab(&mut self) {
        if !self.tab_names.is_empty() {
            self.current_tab = if self.current_tab == 0 {
                self.tab_names.len() - 1
            } else {
                self.current_tab - 1
            };
        }
    }
}
