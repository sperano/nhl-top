//! Navigation utilities for panel stack management
//!
//! This module provides helper functions for working with the navigation panel stack.
//! It supports breadcrumb generation and panel metadata extraction.

use super::state::PanelState;
use super::types::Panel;

/// Generate breadcrumb trail from panel stack
///
/// Returns a vector of labels representing the navigation path.
/// Example: `["TOR", "Player 8478402"]`
pub fn breadcrumb_trail(panel_stack: &[PanelState]) -> Vec<String> {
    panel_stack.iter().map(|ps| ps.panel.label()).collect()
}

/// Generate breadcrumb string from panel stack
///
/// Returns a single string with labels separated by the given separator.
/// Example with separator " >> ": `"TOR >> Player 8478402"`
pub fn breadcrumb_string(panel_stack: &[PanelState], separator: &str) -> String {
    breadcrumb_trail(panel_stack).join(separator)
}

/// Check if we're at the root (no panels open)
pub fn is_at_root(panel_stack: &[PanelState]) -> bool {
    panel_stack.is_empty()
}

/// Get the current (top) panel if any
pub fn current_panel(panel_stack: &[PanelState]) -> Option<&Panel> {
    panel_stack.last().map(|ps| &ps.panel)
}

/// Get the depth of the panel stack
pub fn stack_depth(panel_stack: &[PanelState]) -> usize {
    panel_stack.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_label_boxscore() {
        let panel = Panel::Boxscore { game_id: 2024020001 };
        assert_eq!(panel.label(), "Game 2024020001");
    }

    #[test]
    fn test_panel_label_team_detail() {
        let panel = Panel::TeamDetail {
            abbrev: "TOR".to_string(),
        };
        assert_eq!(panel.label(), "TOR");
    }

    #[test]
    fn test_panel_label_player_detail() {
        let panel = Panel::PlayerDetail { player_id: 8478402 };
        assert_eq!(panel.label(), "Player 8478402");
    }

    #[test]
    fn test_breadcrumb_trail_empty() {
        let stack: Vec<PanelState> = vec![];
        let trail = breadcrumb_trail(&stack);
        assert!(trail.is_empty());
    }

    #[test]
    fn test_breadcrumb_trail_single_panel() {
        let stack = vec![PanelState {
            panel: Panel::TeamDetail {
                abbrev: "TOR".to_string(),
            },
            scroll_offset: 0,
            selected_index: None,
        }];
        let trail = breadcrumb_trail(&stack);
        assert_eq!(trail, vec!["TOR"]);
    }

    #[test]
    fn test_breadcrumb_trail_multiple_panels() {
        let stack = vec![
            PanelState {
                panel: Panel::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                scroll_offset: 0,
                selected_index: None,
            },
            PanelState {
                panel: Panel::PlayerDetail { player_id: 8478402 },
                scroll_offset: 0,
                selected_index: None,
            },
        ];
        let trail = breadcrumb_trail(&stack);
        assert_eq!(trail, vec!["TOR", "Player 8478402"]);
    }

    #[test]
    fn test_breadcrumb_string_with_separator() {
        let stack = vec![
            PanelState {
                panel: Panel::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                scroll_offset: 0,
                selected_index: None,
            },
            PanelState {
                panel: Panel::PlayerDetail { player_id: 8478402 },
                scroll_offset: 0,
                selected_index: None,
            },
        ];
        assert_eq!(breadcrumb_string(&stack, " >> "), "TOR >> Player 8478402");
        assert_eq!(breadcrumb_string(&stack, " / "), "TOR / Player 8478402");
    }

    #[test]
    fn test_is_at_root() {
        let empty: Vec<PanelState> = vec![];
        assert!(is_at_root(&empty));

        let with_panel = vec![PanelState {
            panel: Panel::TeamDetail {
                abbrev: "TOR".to_string(),
            },
            scroll_offset: 0,
            selected_index: None,
        }];
        assert!(!is_at_root(&with_panel));
    }

    #[test]
    fn test_current_panel() {
        let empty: Vec<PanelState> = vec![];
        assert!(current_panel(&empty).is_none());

        let stack = vec![
            PanelState {
                panel: Panel::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                scroll_offset: 0,
                selected_index: None,
            },
            PanelState {
                panel: Panel::PlayerDetail { player_id: 8478402 },
                scroll_offset: 0,
                selected_index: None,
            },
        ];

        let current = current_panel(&stack).unwrap();
        assert!(matches!(current, Panel::PlayerDetail { player_id: 8478402 }));
    }

    #[test]
    fn test_stack_depth() {
        let empty: Vec<PanelState> = vec![];
        assert_eq!(stack_depth(&empty), 0);

        let stack = vec![
            PanelState {
                panel: Panel::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                scroll_offset: 0,
                selected_index: None,
            },
            PanelState {
                panel: Panel::PlayerDetail { player_id: 8478402 },
                scroll_offset: 0,
                selected_index: None,
            },
        ];
        assert_eq!(stack_depth(&stack), 2);
    }
}
