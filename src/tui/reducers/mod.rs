// Module exports for the reducer sub-modules

pub mod data_loading;
pub mod document_stack;
pub mod navigation;
pub mod scores;
pub mod standings;

// Re-export the main reducer functions for convenience
pub use data_loading::reduce_data_loading;
pub use document_stack::reduce_document_stack;
pub use navigation::reduce_navigation;
pub use scores::reduce_scores;
pub use standings::reduce_standings;
