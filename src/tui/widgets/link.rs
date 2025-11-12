/// Clickable link widget for navigation
///
/// This widget provides a focusable, clickable text element that triggers
/// navigation actions. It's used for player names, team names, and other
/// interactive elements throughout the TUI.

use super::focus::*;
use crate::config::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

/// A clickable link widget
///
/// Links can navigate to teams, players, games, or trigger custom actions.
/// They change appearance when focused to provide visual feedback.
pub struct Link {
    id: WidgetId,
    text: String,
    focused: bool,
    /// Callback when link is activated
    on_activate: Option<Box<dyn FnMut() -> NavigationAction + Send>>,
    /// Optional styling
    style: LinkStyle,
}

/// Visual styling for links
#[derive(Debug, Clone)]
pub struct LinkStyle {
    /// Style when not focused
    pub normal: Style,
    /// Style when focused
    pub focused: Style,
    /// Prefix shown when focused
    pub focus_indicator: String,
}

impl Default for LinkStyle {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            focused: Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED),
            focus_indicator: "▶ ".to_string(),
        }
    }
}

impl Link {
    /// Create a new link with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: WidgetId::new(),
            text: text.into(),
            focused: false,
            on_activate: None,
            style: LinkStyle::default(),
        }
    }

    /// Set the action callback
    pub fn with_action<F>(mut self, action: F) -> Self
    where
        F: FnMut() -> NavigationAction + Send + 'static,
    {
        self.on_activate = Some(Box::new(action));
        self
    }

    /// Set custom styling
    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.style = style;
        self
    }

    /// Create a player link
    ///
    /// When activated, navigates to the player's profile page.
    pub fn player(name: impl Into<String>, player_id: i64) -> Self {
        Self::new(name).with_action(move || NavigationAction::NavigateToPlayer(player_id))
    }

    /// Create a team link
    ///
    /// When activated, navigates to the team's page.
    pub fn team(name: impl Into<String>, team_abbrev: impl Into<String>) -> Self {
        let abbrev = team_abbrev.into();
        Self::new(name).with_action(move || NavigationAction::NavigateToTeam(abbrev.clone()))
    }

    /// Create a game link
    ///
    /// When activated, navigates to the game details page.
    pub fn game(text: impl Into<String>, game_id: i64) -> Self {
        Self::new(text).with_action(move || NavigationAction::NavigateToGame(game_id))
    }
}

impl Focusable for Link {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        self.on_activate.is_some()
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused {
            return InputResult::NotHandled;
        }

        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(ref mut action) = self.on_activate {
                    let navigation = action();
                    InputResult::Navigate(navigation)
                } else {
                    InputResult::Handled
                }
            }
            _ => InputResult::NotHandled,
        }
    }
}

impl super::RenderableWidget for Link {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let (style, text) = if self.focused {
            let focused_style = self.style.focused.patch(
                Style::default().fg(config.selection_fg)
            );
            let text = format!("{}{}", self.style.focus_indicator, self.text);
            (focused_style, text)
        } else {
            (self.style.normal, self.text.clone())
        };

        // Render text, truncating if necessary
        let max_width = area.width as usize;
        let display_text = if text.len() > max_width {
            format!("{}...", &text[..max_width.saturating_sub(3)])
        } else {
            text
        };

        buf.set_string(area.x, area.y, &display_text, style);
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(1)
    }

    fn preferred_width(&self) -> Option<u16> {
        let width = if self.focused {
            self.text.len() + self.style.focus_indicator.len()
        } else {
            self.text.len()
        };
        Some(width as u16)
    }
}

/// Builder for creating multiple links with consistent styling
pub struct LinkBuilder {
    style: LinkStyle,
}

impl LinkBuilder {
    /// Create a new link builder
    pub fn new() -> Self {
        Self {
            style: LinkStyle::default(),
        }
    }

    /// Set the style for all links created by this builder
    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.style = style;
        self
    }

    /// Create a player link
    pub fn player(&self, name: impl Into<String>, player_id: i64) -> Link {
        Link::player(name, player_id).with_style(self.style.clone())
    }

    /// Create a team link
    pub fn team(&self, name: impl Into<String>, team_abbrev: impl Into<String>) -> Link {
        Link::team(name, team_abbrev).with_style(self.style.clone())
    }

    /// Create a game link
    pub fn game(&self, text: impl Into<String>, game_id: i64) -> Link {
        Link::game(text, game_id).with_style(self.style.clone())
    }
}

impl Default for LinkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DisplayConfig;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{buffer::Buffer, layout::Rect};

    fn test_config() -> DisplayConfig {
        DisplayConfig::default()
    }

    #[test]
    fn test_link_creation() {
        let link = Link::new("Test Link");
        assert!(!link.is_focused());
        assert!(!link.can_focus()); // No action set yet
    }

    #[test]
    fn test_link_player() {
        let link = Link::player("Connor McDavid", 8478402);
        assert!(link.can_focus());
        assert!(!link.is_focused());
    }

    #[test]
    fn test_link_team() {
        let link = Link::team("Edmonton Oilers", "EDM");
        assert!(link.can_focus());
    }

    #[test]
    fn test_link_focus_state() {
        let mut link = Link::new("Test");
        assert!(!link.is_focused());

        link.set_focused(true);
        assert!(link.is_focused());

        link.set_focused(false);
        assert!(!link.is_focused());
    }

    #[test]
    fn test_link_activates_on_enter() {
        let mut link = Link::player("Test Player", 123);
        link.set_focused(true);

        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = link.handle_input(event);

        match result {
            InputResult::Navigate(NavigationAction::NavigateToPlayer(id)) => {
                assert_eq!(id, 123);
            }
            _ => panic!("Expected Navigate action"),
        }
    }

    #[test]
    fn test_link_not_handled_when_unfocused() {
        let mut link = Link::player("Test", 123);

        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = link.handle_input(event);

        assert_eq!(result, InputResult::NotHandled);
    }

    #[test]
    fn test_link_widget_id_unique() {
        let link1 = Link::new("Link 1");
        let link2 = Link::new("Link 2");

        assert_ne!(link1.widget_id(), link2.widget_id());
    }
}
