use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Constraint, Direction, Alignment},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::theme;

#[derive(Debug, Clone, Copy, PartialEq)]
enum StatCategory {
    Points,
    Goals,
    Assists,
    PlusMinus,
    Goalies,
}

impl StatCategory {
    fn label(&self) -> &'static str {
        match self {
            StatCategory::Points => "Top 30 Points",
            StatCategory::Goals => "Top 30 Goals",
            StatCategory::Assists => "Top 30 Assists",
            StatCategory::PlusMinus => "Top 30 Plus/Minus",
            StatCategory::Goalies => "Goalie Stats",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            StatCategory::Points => "Leading scorers across NHL",
            StatCategory::Goals => "Top goal scorers",
            StatCategory::Assists => "Players with most assists",
            StatCategory::PlusMinus => "Best plus/minus ratings",
            StatCategory::Goalies => "Goaltender statistics",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            StatCategory::Points,
            StatCategory::Goals,
            StatCategory::Assists,
            StatCategory::PlusMinus,
            StatCategory::Goalies,
        ]
    }
}

pub struct CategoryListView {
    selected_index: usize,
    categories: Vec<StatCategory>,
}

impl CategoryListView {
    pub fn new() -> Self {
        CategoryListView {
            selected_index: 0,
            categories: StatCategory::all(),
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new("Player Statistics")
            .style(theme::list_header_style())
            .alignment(Alignment::Left);
        f.render_widget(header, area);
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.categories
            .iter()
            .map(|cat| {
                ListItem::new(cat.label())
                    .style(theme::list_normal_style())
            })
            .collect();

        let list = List::new(items)
            .highlight_style(theme::list_selected_style())
            .highlight_symbol(theme::LIST_HIGHLIGHT_SYMBOL);

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_description(&self, f: &mut Frame, area: Rect) {
        if let Some(category) = self.categories.get(self.selected_index) {
            let desc = Paragraph::new(category.description())
                .style(theme::hint_style())
                .alignment(Alignment::Left);
            f.render_widget(desc, area);
        }
    }

    fn render_hint(&self, f: &mut Frame, area: Rect) {
        let hint = Paragraph::new("↑↓ Navigate  •  Enter to view")
            .style(theme::hint_style())
            .alignment(Alignment::Center);
        f.render_widget(hint, area);
    }
}

impl View for CategoryListView {
    fn render(&mut self, f: &mut Frame, area: Rect, _focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Stats ");

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Layout: header, list, description, hint
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Header
                Constraint::Min(5),     // List
                Constraint::Length(2),  // Description
                Constraint::Length(2),  // Hint
            ])
            .split(inner);

        self.render_header(f, chunks[0]);
        self.render_list(f, chunks[1]);
        self.render_description(f, chunks[2]);
        self.render_hint(f, chunks[3]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                KeyResult::Handled
            }
            KeyCode::Down => {
                if self.selected_index < self.categories.len() - 1 {
                    self.selected_index += 1;
                }
                KeyResult::Handled
            }
            KeyCode::Enter => {
                // TODO: Drill down to top list
                KeyResult::Handled
            }
            KeyCode::Esc => KeyResult::GoBack,
            KeyCode::Char('q') => KeyResult::Quit,
            _ => KeyResult::NotHandled,
        }
    }

    fn can_drill_down(&self) -> bool {
        true
    }

    fn breadcrumb_label(&self) -> String {
        "Categories".to_string()
    }
}
