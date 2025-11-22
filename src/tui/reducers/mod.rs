// Module exports for the reducer sub-modules

pub mod data_loading;
pub mod document;
pub mod navigation;
pub mod panels;
pub mod scores;
pub mod standings;
pub mod standings_layout;

// Re-export the main reducer functions for convenience
pub use data_loading::reduce_data_loading;
pub use document::reduce_document;
pub use navigation::reduce_navigation;
pub use panels::reduce_panels;
pub use scores::reduce_scores;
pub use standings::reduce_standings;
