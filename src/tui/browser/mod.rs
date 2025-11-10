mod target;
mod link;
mod content;
mod state;
mod view;
mod handler;

pub use state::State;
pub use view::render_content;
pub use handler::handle_key;

// Re-export types for tests
#[cfg(test)]
pub use target::Target;
#[cfg(test)]
pub use link::Link;
#[cfg(test)]
pub use content::{BrowserContent, BrowserContentBuilder};
