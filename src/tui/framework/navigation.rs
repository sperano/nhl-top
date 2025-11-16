//! Navigation utilities for panel stack management
//!
//! This module provides helper functions for working with the navigation panel stack.
//! It supports breadcrumb generation and panel metadata extraction.

use super::action::Panel;
use super::state::PanelState;

/// Get the display label for a panel (for breadcrumbs)
///
/// Returns a human-readable string representing the panel,
/// suitable for display in breadcrumb trails.
pub fn panel_label(panel: &Panel) -> String {
    match panel {
        Panel::Boxscore { game_id } => format!("Game {}", game_id),
        Panel::TeamDetail { abbrev } => abbrev.clone(),
        Panel::PlayerDetail { player_id } => format!("Player {}", player_id),
    }
}

/// Get a cache key for a panel (for data caching)
///
/// Returns a unique string key that can be used to cache data
/// associated with this panel.
pub fn panel_cache_key(panel: &Panel) -> String {
    match panel {
        Panel::Boxscore { game_id } => format!("boxscore:{}", game_id),
        Panel::TeamDetail { abbrev } => format!("team:{}", abbrev),
        Panel::PlayerDetail { player_id } => format!("player:{}", player_id),
    }
}

/// Generate breadcrumb trail from panel stack
///
/// Returns a vector of labels representing the navigation path.
/// Example: `["TOR", "Player 8478402"]`
pub fn breadcrumb_trail(panel_stack: &[PanelState]) -> Vec<String> {
    panel_stack.iter().map(|ps| panel_label(&ps.panel)).collect()
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
        assert_eq!(panel_label(&panel), "Game 2024020001");
    }

    #[test]
    fn test_panel_label_team_detail() {
        let panel = Panel::TeamDetail {
            abbrev: "TOR".to_string(),
        };
        assert_eq!(panel_label(&panel), "TOR");
    }

    #[test]
    fn test_panel_label_player_detail() {
        let panel = Panel::PlayerDetail { player_id: 8478402 };
        assert_eq!(panel_label(&panel), "Player 8478402");
    }

    #[test]
    fn test_panel_cache_key() {
        let boxscore = Panel::Boxscore { game_id: 2024020001 };
        assert_eq!(panel_cache_key(&boxscore), "boxscore:2024020001");

        let team = Panel::TeamDetail {
            abbrev: "TOR".to_string(),
        };
        assert_eq!(panel_cache_key(&team), "team:TOR");

        let player = Panel::PlayerDetail { player_id: 8478402 };
        assert_eq!(panel_cache_key(&player), "player:8478402");
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
