pub mod state;
pub mod game_details;
pub mod panel;
mod view;
mod handler;

pub use state::State;
pub use view::{render_subtabs, render_content, format_boxscore_text};
pub use handler::handle_key;
pub use panel::ScoresPanel;
