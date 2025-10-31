use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Constraint, Direction, Alignment},
    style::Modifier,
    text::Text,
    widgets::{Block, Borders, Paragraph, BorderType},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::theme;
use crate::SharedDataHandle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Division,
    Conference,
    League,
}

impl GroupBy {
    fn label(&self) -> &'static str {
        match self {
            GroupBy::Division => "Division",
            GroupBy::Conference => "Conference",
            GroupBy::League => "League",
        }
    }

    fn next(&self) -> Self {
        match self {
            GroupBy::Division => GroupBy::Conference,
            GroupBy::Conference => GroupBy::League,
            GroupBy::League => GroupBy::Division,
        }
    }

    fn prev(&self) -> Self {
        match self {
            GroupBy::Division => GroupBy::League,
            GroupBy::Conference => GroupBy::Division,
            GroupBy::League => GroupBy::Conference,
        }
    }
}

pub struct ViewSelectorView {
    selected: GroupBy,
    shared_data: SharedDataHandle,
}

impl ViewSelectorView {
    pub fn new(shared_data: SharedDataHandle) -> Self {
        ViewSelectorView {
            selected: GroupBy::Division,
            shared_data,
        }
    }

    /// Render horizontal pills for view selection
    fn render_pills(&self, f: &mut Frame, area: Rect) {
        let options = [GroupBy::Division, GroupBy::Conference, GroupBy::League];

        // Calculate pill width (equal distribution with gaps)
        let pill_width = (area.width - 4) / 3; // 4 for gaps between pills
        let gap = 2;

        for (i, option) in options.iter().enumerate() {
            let x = area.x + (i as u16 * (pill_width + gap));
            let pill_area = Rect::new(x, area.y, pill_width, 3);

            let is_selected = *option == self.selected;

            let (border_style, text_style, border_type) = if is_selected {
                (
                    theme::pill_selected_border(),
                    theme::pill_selected_text(),
                    BorderType::Double,
                )
            } else {
                (
                    theme::pill_normal_border(),
                    theme::pill_normal_text(),
                    BorderType::Plain,
                )
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(border_style);

            let paragraph = Paragraph::new(option.label())
                .style(text_style)
                .alignment(Alignment::Center)
                .block(block);

            f.render_widget(paragraph, pill_area);
        }
    }

    /// Render hint text below pills
    fn render_hint(&self, f: &mut Frame, area: Rect) {
        let hint = Paragraph::new("← → Switch view  •  Enter to view standings")
            .style(theme::hint_style())
            .alignment(Alignment::Center);

        f.render_widget(hint, area);
    }
}

impl View for ViewSelectorView {
    fn render(&mut self, f: &mut Frame, area: Rect, _focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Standings ");

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Layout: pills at top, hint below, rest is empty space
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Pills
                Constraint::Length(2),  // Hint
                Constraint::Min(0),     // Empty space
            ])
            .split(inner);

        self.render_pills(f, chunks[0]);
        self.render_hint(f, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Left => {
                self.selected = self.selected.prev();
                KeyResult::Handled
            }
            KeyCode::Right => {
                self.selected = self.selected.next();
                KeyResult::Handled
            }
            KeyCode::Enter => {
                // TODO: Drill down to standings table
                // For now, just handle the key
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
        "View Selection".to_string()
    }
}
