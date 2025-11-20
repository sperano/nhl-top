/// Shared types used across the application
///
/// This module contains type definitions that are shared between
/// the library (commands, tui) and the binary (main.rs).

/// Global constants
pub const NHL_LEAGUE_ABBREV: &str = "NHL";

enum TeamNameFormat {
    Abbreviated,
    City,
    Common,
    Full,
}