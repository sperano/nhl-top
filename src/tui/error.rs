use thiserror::Error;
use tokio::sync::mpsc;

/// TUI-specific errors
#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Game has not started yet")]
    GameNotStarted,

    #[error("Failed to send refresh signal")]
    RefreshSignalFailed(#[from] mpsc::error::SendError<()>),

    #[error("No schedule data available")]
    NoScheduleData,

    #[error("Game not found at index {0}")]
    GameNotFound(usize),

    #[error("Invalid grid position: row={0}, col={1}")]
    InvalidGridPosition(usize, usize),
}

/// Result type for TUI operations
pub type TuiResult<T> = Result<T, TuiError>;
