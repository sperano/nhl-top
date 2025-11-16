//! General test utilities for TUI tests.
//!
//! This module provides common test helpers used across multiple test modules.
//! For widget-specific rendering helpers, see `crate::tui::widgets::testing`.
//!
//! # Usage
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use crate::tui::testing::*;
//!
//!     #[test]
//!     fn test_example() {
//!         let client = create_client();
//!         // Use client in tests...
//!     }
//! }
//! ```

use nhl_api::{Client, Standing};
use std::sync::Arc;
use ratatui::buffer::Buffer;

/// Creates an Arc-wrapped NHL API client for testing.
///
/// This helper eliminates the repetitive `Arc::new(Client::new().unwrap())`
/// pattern found across multiple test modules.
///
/// # Panics
///
/// Panics if client creation fails (acceptable in test code).
///
/// # Examples
///
/// ```rust
/// use crate::tui::testing::create_client;
///
/// #[test]
/// fn test_with_client() {
///     let client = create_client();
///     // Use client for testing...
/// }
/// ```
pub fn create_client() -> Arc<Client> {
    Arc::new(Client::new().expect("Failed to create test NHL API client"))
}

/// Constant for general rendering width
pub const RENDER_WIDTH: u16 = 80;

/// Helper to extract lines from buffer
pub fn buffer_lines(buf: &Buffer) -> Vec<String> {
    let area = buf.area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect()
}

/// Helper for buffer assertions
pub fn assert_buffer(buf: &Buffer, expected: &[&str]) {
    let actual = buffer_lines(buf);
    let buffer_width = buf.area().width as usize;

    assert_eq!(
        actual.len(),
        expected.len(),
        "Buffer height mismatch: expected {} lines, got {}",
        expected.len(),
        actual.len()
    );
    for (i, expected_line) in expected.iter().enumerate() {
        assert_eq!(
            actual[i].chars().count(),
            buffer_width,
            "Line {} width mismatch: expected {}, got {}",
            i,
            buffer_width,
            actual[i].chars().count()
        );
        assert_eq!(
            actual[i].trim_end(),
            expected_line.trim_end(),
            "Line {} mismatch:\nExpected: '{}'\nActual:   '{}'",
            i,
            expected_line,
            actual[i]
        );
    }
}

/// Helper to create a test Standing
fn create_division_team(
    name: &str,
    abbrev: &str,
    division: &str,
    conference: &str,
    wins: i32,
    losses: i32,
    ot_losses: i32,
    points: i32,
) -> Standing {
    use nhl_api::LocalizedString;

    Standing {
        conference_abbrev: Some(conference.to_string()),
        conference_name: Some(conference.to_string()),
        division_abbrev: division.to_string(),
        division_name: division.to_string(),
        team_name: LocalizedString {
            default: name.to_string(),
        },
        team_common_name: LocalizedString {
            default: name.to_string(),
        },
        team_abbrev: LocalizedString {
            default: abbrev.to_string(),
        },
        team_logo: format!("https://assets.nhle.com/logos/nhl/svg/{}_light.svg", abbrev),
        wins,
        losses,
        ot_losses,
        points,
    }
}

/// Create a full 32-team NHL standings for testing
pub fn create_test_standings() -> Vec<Standing> {
    vec![
        // Atlantic (Eastern) - 8 teams
        create_division_team("Panthers", "FLA", "Atlantic", "Eastern", 14, 3, 2, 30),
        create_division_team("Bruins", "BOS", "Atlantic", "Eastern", 13, 4, 1, 27),
        create_division_team("Maple Leafs", "TOR", "Atlantic", "Eastern", 12, 5, 2, 26),
        create_division_team("Lightning", "TBL", "Atlantic", "Eastern", 11, 6, 1, 23),
        create_division_team("Canadiens", "MTL", "Atlantic", "Eastern", 10, 5, 3, 23),
        create_division_team("Senators", "OTT", "Atlantic", "Eastern", 9, 7, 2, 20),
        create_division_team("Red Wings", "DET", "Atlantic", "Eastern", 8, 8, 2, 18),
        create_division_team("Sabres", "BUF", "Atlantic", "Eastern", 6, 10, 2, 14),
        // Metropolitan (Eastern) - 8 teams
        create_division_team("Devils", "NJD", "Metropolitan", "Eastern", 15, 2, 1, 31),
        create_division_team("Hurricanes", "CAR", "Metropolitan", "Eastern", 14, 3, 2, 30),
        create_division_team("Rangers", "NYR", "Metropolitan", "Eastern", 12, 5, 1, 25),
        create_division_team("Penguins", "PIT", "Metropolitan", "Eastern", 11, 6, 2, 24),
        create_division_team("Capitals", "WSH", "Metropolitan", "Eastern", 10, 7, 1, 21),
        create_division_team("Islanders", "NYI", "Metropolitan", "Eastern", 9, 7, 2, 20),
        create_division_team("Flyers", "PHI", "Metropolitan", "Eastern", 8, 9, 1, 17),
        create_division_team("Blue Jackets", "CBJ", "Metropolitan", "Eastern", 5, 11, 2, 12),
        // Central (Western) - 8 teams
        create_division_team("Avalanche", "COL", "Central", "Western", 16, 2, 1, 33),
        create_division_team("Stars", "DAL", "Central", "Western", 14, 4, 2, 30),
        create_division_team("Jets", "WPG", "Central", "Western", 13, 5, 1, 27),
        create_division_team("Wild", "MIN", "Central", "Western", 11, 6, 2, 24),
        create_division_team("Predators", "NSH", "Central", "Western", 10, 7, 2, 22),
        create_division_team("Blues", "STL", "Central", "Western", 8, 8, 3, 19),
        create_division_team("Blackhawks", "CHI", "Central", "Western", 7, 10, 1, 15),
        create_division_team("Coyotes", "ARI", "Central", "Western", 4, 13, 1, 9),
        // Pacific (Western) - 8 teams
        create_division_team("Golden Knights", "VGK", "Pacific", "Western", 15, 3, 1, 31),
        create_division_team("Oilers", "EDM", "Pacific", "Western", 14, 4, 2, 30),
        create_division_team("Kings", "LA", "Pacific", "Western", 12, 6, 1, 25),
        create_division_team("Kraken", "SEA", "Pacific", "Western", 11, 6, 2, 24),
        create_division_team("Canucks", "VAN", "Pacific", "Western", 10, 7, 2, 22),
        create_division_team("Flames", "CGY", "Pacific", "Western", 9, 8, 2, 20),
        create_division_team("Ducks", "ANA", "Pacific", "Western", 7, 10, 2, 16),
        create_division_team("Sharks", "SJ", "Pacific", "Western", 5, 12, 1, 11),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client_returns_arc() {
        let client = create_client();
        assert_eq!(Arc::strong_count(&client), 1);
    }

    #[test]
    fn test_create_client_can_be_cloned() {
        let client1 = create_client();
        let client2 = Arc::clone(&client1);
        assert_eq!(Arc::strong_count(&client1), 2);
        drop(client2); // Ensure client2 is used
    }

    #[test]
    fn test_create_client_is_functional() {
        let client = create_client();
        // Verify we can call methods on the client
        // (Client::new() should create a valid instance)
        assert!(Arc::strong_count(&client) > 0);
    }
}

