use super::traits::View;

/// Main tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Scores,
    Standings,
    Stats,
    Settings,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Scores => "Scores",
            Tab::Standings => "Standings",
            Tab::Stats => "Stats",
            Tab::Settings => "Settings",
        }
    }

    pub fn number(&self) -> usize {
        match self {
            Tab::Scores => 1,
            Tab::Standings => 2,
            Tab::Stats => 3,
            Tab::Settings => 4,
        }
    }

    pub fn from_number(n: usize) -> Option<Self> {
        match n {
            1 => Some(Tab::Scores),
            2 => Some(Tab::Standings),
            3 => Some(Tab::Stats),
            4 => Some(Tab::Settings),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Tab::Scores => Tab::Standings,
            Tab::Standings => Tab::Stats,
            Tab::Stats => Tab::Settings,
            Tab::Settings => Tab::Scores,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Tab::Scores => Tab::Settings,
            Tab::Standings => Tab::Scores,
            Tab::Stats => Tab::Standings,
            Tab::Settings => Tab::Stats,
        }
    }
}

/// Application state managing navigation
pub struct AppState {
    pub current_tab: Tab,
    pub view_stack: Vec<Box<dyn View>>,
    pub breadcrumb: Vec<String>,
}

impl AppState {
    pub fn new(initial_tab: Tab, root_view: Box<dyn View>) -> Self {
        let breadcrumb = vec![initial_tab.label().to_string(), root_view.breadcrumb_label()];
        AppState {
            current_tab: initial_tab,
            view_stack: vec![root_view],
            breadcrumb,
        }
    }

    /// Get the current active view (top of stack)
    pub fn current_view(&mut self) -> &mut Box<dyn View> {
        self.view_stack.last_mut().expect("View stack should never be empty")
    }

    /// Push a new view onto the stack
    pub fn push_view(&mut self, view: Box<dyn View>) {
        self.breadcrumb.push(view.breadcrumb_label());
        self.view_stack.push(view);
    }

    /// Pop the current view from the stack
    /// Returns false if we're already at the root view
    pub fn pop_view(&mut self) -> bool {
        if self.view_stack.len() > 1 {
            self.view_stack.pop();
            self.breadcrumb.pop();
            true
        } else {
            false
        }
    }

    /// Get the current navigation depth (0 = root view)
    pub fn depth(&self) -> usize {
        self.view_stack.len() - 1
    }

    /// Check if we're at the root level of the current tab
    pub fn at_root(&self) -> bool {
        self.depth() == 0
    }

    /// Replace the entire view stack with a new root view
    /// Used when switching tabs
    pub fn replace_root(&mut self, view: Box<dyn View>) {
        self.view_stack.clear();
        self.breadcrumb.clear();
        self.breadcrumb.push(self.current_tab.label().to_string());
        self.breadcrumb.push(view.breadcrumb_label());
        self.view_stack.push(view);
    }
}
