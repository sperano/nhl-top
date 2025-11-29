//! Navigation utilities for document stack management
//!
//! This module provides helper functions for working with the navigation document stack.
//! It supports breadcrumb generation and document metadata extraction.

use super::state::DocumentStackEntry;
use super::types::StackedDocument;

/// Generate breadcrumb trail from document stack
///
/// Returns a vector of labels representing the navigation path.
/// Example: `["TOR", "Player 8478402"]`
pub fn breadcrumb_trail(document_stack: &[DocumentStackEntry]) -> Vec<String> {
    document_stack.iter().map(|ds| ds.document.label()).collect()
}

/// Generate breadcrumb string from document stack
///
/// Returns a single string with labels separated by the given separator.
/// Example with separator " >> ": `"TOR >> Player 8478402"`
pub fn breadcrumb_string(document_stack: &[DocumentStackEntry], separator: &str) -> String {
    breadcrumb_trail(document_stack).join(separator)
}

/// Check if we're at the root (no documents open)
pub fn is_at_root(document_stack: &[DocumentStackEntry]) -> bool {
    document_stack.is_empty()
}

/// Get the current (top) document if any
pub fn current_document(document_stack: &[DocumentStackEntry]) -> Option<&StackedDocument> {
    document_stack.last().map(|ds| &ds.document)
}

/// Get the depth of the document stack
pub fn stack_depth(document_stack: &[DocumentStackEntry]) -> usize {
    document_stack.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_label_boxscore() {
        let doc = StackedDocument::Boxscore {
            game_id: 2024020001,
        };
        assert_eq!(doc.label(), "Game 2024020001");
    }

    #[test]
    fn test_document_label_team_detail() {
        let doc = StackedDocument::TeamDetail {
            abbrev: "TOR".to_string(),
        };
        assert_eq!(doc.label(), "TOR");
    }

    #[test]
    fn test_document_label_player_detail() {
        let doc = StackedDocument::PlayerDetail { player_id: 8478402 };
        assert_eq!(doc.label(), "Player 8478402");
    }

    #[test]
    fn test_breadcrumb_trail_empty() {
        let stack: Vec<DocumentStackEntry> = vec![];
        let trail = breadcrumb_trail(&stack);
        assert!(trail.is_empty());
    }

    #[test]
    fn test_breadcrumb_trail_single_document() {
        let stack = vec![DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "TOR".to_string(),
            },
            selected_index: None,
            scroll_offset: 0,
        }];
        let trail = breadcrumb_trail(&stack);
        assert_eq!(trail, vec!["TOR"]);
    }

    #[test]
    fn test_breadcrumb_trail_multiple_documents() {
        let stack = vec![
            DocumentStackEntry {
                document: StackedDocument::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                selected_index: None,
                scroll_offset: 0,
            },
            DocumentStackEntry {
                document: StackedDocument::PlayerDetail { player_id: 8478402 },
                selected_index: None,
                scroll_offset: 0,
            },
        ];
        let trail = breadcrumb_trail(&stack);
        assert_eq!(trail, vec!["TOR", "Player 8478402"]);
    }

    #[test]
    fn test_breadcrumb_string_with_separator() {
        let stack = vec![
            DocumentStackEntry {
                document: StackedDocument::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                selected_index: None,
                scroll_offset: 0,
            },
            DocumentStackEntry {
                document: StackedDocument::PlayerDetail { player_id: 8478402 },
                selected_index: None,
                scroll_offset: 0,
            },
        ];
        assert_eq!(breadcrumb_string(&stack, " >> "), "TOR >> Player 8478402");
        assert_eq!(breadcrumb_string(&stack, " / "), "TOR / Player 8478402");
    }

    #[test]
    fn test_is_at_root() {
        let empty: Vec<DocumentStackEntry> = vec![];
        assert!(is_at_root(&empty));

        let with_document = vec![DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "TOR".to_string(),
            },
            selected_index: None,
            scroll_offset: 0,
        }];
        assert!(!is_at_root(&with_document));
    }

    #[test]
    fn test_current_document() {
        let empty: Vec<DocumentStackEntry> = vec![];
        assert!(current_document(&empty).is_none());

        let stack = vec![
            DocumentStackEntry {
                document: StackedDocument::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                selected_index: None,
                scroll_offset: 0,
            },
            DocumentStackEntry {
                document: StackedDocument::PlayerDetail { player_id: 8478402 },
                selected_index: None,
                scroll_offset: 0,
            },
        ];

        let current = current_document(&stack).unwrap();
        assert!(matches!(
            current,
            StackedDocument::PlayerDetail { player_id: 8478402 }
        ));
    }

    #[test]
    fn test_stack_depth() {
        let empty: Vec<DocumentStackEntry> = vec![];
        assert_eq!(stack_depth(&empty), 0);

        let stack = vec![
            DocumentStackEntry {
                document: StackedDocument::TeamDetail {
                    abbrev: "TOR".to_string(),
                },
                selected_index: None,
                scroll_offset: 0,
            },
            DocumentStackEntry {
                document: StackedDocument::PlayerDetail { player_id: 8478402 },
                selected_index: None,
                scroll_offset: 0,
            },
        ];
        assert_eq!(stack_depth(&stack), 2);
    }
}
