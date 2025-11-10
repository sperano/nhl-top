// ! Panel definitions for scores navigation

use crate::tui::navigation::Panel;

/// Panels that can be navigated to in scores view
#[derive(Clone, Debug, PartialEq)]
pub enum ScoresPanel {
    /// Player detail view showing player stats and career from a game
    PlayerDetail {
        player_id: i64,
        player_name: String,
        from_game_id: i64,
    },
}

impl Panel for ScoresPanel {
    fn breadcrumb_label(&self) -> String {
        match self {
            ScoresPanel::PlayerDetail { player_name, .. } => player_name.clone(),
        }
    }

    fn cache_key(&self) -> String {
        match self {
            ScoresPanel::PlayerDetail { player_id, .. } => format!("player:{}", player_id),
        }
    }
}
