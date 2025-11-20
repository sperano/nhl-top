//! Shared layout constants used across CLI and TUI components.
//!
//! This module centralizes common layout values to ensure consistency
//! and make it easier to adjust layouts globally.

/// Width of a score box/game box in the TUI and CLI output
pub const SCORE_BOX_WIDTH: u16 = 37;

/// Margin added around game boxes for spacing
pub const GAME_BOX_MARGIN: u16 = 2;

/// Total width of a game box including margins
pub const GAME_BOX_WITH_MARGIN: u16 = SCORE_BOX_WIDTH + GAME_BOX_MARGIN;

/// Width of period score columns (e.g., "P1", "P2", "P3")
pub const PERIOD_COL_WIDTH: usize = 4;

/// Width of team abbreviation column
pub const TEAM_ABBREV_COL_WIDTH: usize = 5;

/// Size of the date window shown in scores tab
pub const DATE_WINDOW_SIZE: usize = 5;

// CLI-specific formatting constants

/// Width of schedule box content (excluding borders)
pub const SCHEDULE_BOX_CONTENT_WIDTH: usize = 60;

/// Total width of schedule box (including 2-char borders)
pub const SCHEDULE_BOX_TOTAL_WIDTH: usize = SCHEDULE_BOX_CONTENT_WIDTH + 2;

/// Width of boxscore stat labels
pub const BOXSCORE_LABEL_WIDTH: usize = 20;

/// Width of boxscore score display
pub const BOXSCORE_SCORE_WIDTH: usize = 3;

/// Width of boxscore stat bar visualization
pub const BOXSCORE_STAT_BAR_WIDTH: usize = 30;
