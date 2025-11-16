// use crate::tui::widgets::Container;
//
// // === COMMENTED OUT FOR REFACTORING - WILL REACTIVATE LATER ===
// // This code represents the old state-based scores implementation
// // Keep for reference when rebuilding scores functionality with Container widgets
// //
// // use crate::tui::common::scrollable::Scrollable;
// // use crate::tui::scores::game_details::GameDetailsState;
// // use crate::tui::scores::panel::ScoresPanel;
// // use crate::tui::navigation::NavigationContext;
// //
// // /// Date window configuration
// // pub const DATE_WINDOW_SIZE: usize = 5;
// // pub const DATE_WINDOW_CENTER: usize = 2;
// // pub const DATE_WINDOW_MIN_INDEX: usize = 0;
// // pub const DATE_WINDOW_MAX_INDEX: usize = 4;
//
// /// Date window configuration
// pub const DATE_WINDOW_SIZE: usize = 5;
// pub const DATE_WINDOW_CENTER: usize = 2;
// pub const DATE_WINDOW_MIN_INDEX: usize = 0;
// pub const DATE_WINDOW_MAX_INDEX: usize = 4;
//
// pub struct State {
//     //pub container: Option<Container>,
//     pub subtab_focused: bool,
//
//     // Date window navigation
//     pub selected_index: usize,
//
//     // Box/game selection
//     pub box_selection_active: bool,
//     pub selected_box: (usize, usize), // (col, row)
//     pub grid_dimensions: (usize, usize), // (num_cols, num_rows)
//
//     // Boxscore view (for future implementation)
//     pub boxscore_view_active: bool,
//
//     // === OLD FIELDS - KEPT FOR REFERENCE ===
//     // pub boxscore_scrollable: Scrollable,
//     // pub grid_scrollable: Scrollable,
//     // pub game_details: GameDetailsState,
//     // pub navigation: Option<NavigationContext<ScoresPanel, String, ()>>,
//     // pub panel_scrollable: Scrollable,
// }
//
// impl State {
//     pub fn new() -> Self {
//         Self {
//             //container: None,
//             subtab_focused: false,
//             selected_index: DATE_WINDOW_CENTER,
//             box_selection_active: false,
//             selected_box: (0, 0),
//             grid_dimensions: (0, 0),
//             boxscore_view_active: false,
//         }
//     }
//
//     /// Calculate the linear game index from grid position
//     pub fn get_selected_game_index(&self) -> Option<usize> {
//         if !self.box_selection_active {
//             return None;
//         }
//
//         let (col, row) = self.selected_box;
//         let (num_cols, _) = self.grid_dimensions;
//
//         if num_cols == 0 {
//             return None;
//         }
//
//         Some(row * num_cols + col)
//     }
//
//     /// Update grid dimensions based on terminal width and number of games
//     pub fn update_grid_dimensions(&mut self, terminal_width: u16, num_games: usize) {
//         let num_cols = if terminal_width >= 115 {
//             3
//         } else if terminal_width >= 76 {
//             2
//         } else {
//             1
//         };
//
//         let num_rows = if num_cols > 0 {
//             (num_games + num_cols - 1) / num_cols
//         } else {
//             0
//         };
//
//         self.grid_dimensions = (num_cols, num_rows);
//
//         // Clamp selected box to valid range
//         let (col, row) = self.selected_box;
//         if col >= num_cols && num_cols > 0 {
//             self.selected_box.0 = num_cols - 1;
//         }
//         if row >= num_rows && num_rows > 0 {
//             self.selected_box.1 = num_rows - 1;
//         }
//     }
// }
//
// // === OLD IMPLEMENTATION - COMMENTED FOR REFERENCE ===
// // pub fn new() -> Self {
// //     State {
// //         selected_index: DATE_WINDOW_CENTER,
// //         subtab_focused: false,
// //         box_selection_active: false,
// //         selected_box: (0, 0),
// //         grid_dimensions: (0, 0),
// //         boxscore_view_active: false,
// //         boxscore_scrollable: Scrollable::new(),
// //         grid_scrollable: Scrollable::new(),
// //         game_details: GameDetailsState::new(),
// //         navigation: None,
// //         panel_scrollable: Scrollable::new(),
// //     }
// // }
//
// impl Default for State {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// // impl crate::tui::context::NavigationContextProvider for State {
//     // fn get_available_actions(&self) -> Vec<crate::tui::widgets::Action> {
//     //     let mut actions = vec![];
//     //
//     //     if self.subtab_focused {
//     //         actions.push(crate::tui::widgets::Action {
//     //             key: "←→".to_string(),
//     //             label: "Change Date".to_string(),
//     //             enabled: true,
//     //         });
//     //     }
//     //
//     //     actions
//     // }
//
//     fn get_keyboard_hints(&self) -> Vec<crate::tui::widgets::KeyHint> {
//         use crate::tui::widgets::{KeyHint, KeyHintStyle};
//         let mut hints = vec![];
//
//         if self.subtab_focused {
//             hints.push(KeyHint {
//                 key: "←→".to_string(),
//                 action: "Change Date".to_string(),
//                 style: KeyHintStyle::Important,
//             });
//             hints.push(KeyHint {
//                 key: "↑".to_string(),
//                 action: "Back".to_string(),
//                 style: KeyHintStyle::Normal,
//             });
//         } else {
//             hints.push(KeyHint {
//                 key: "↓".to_string(),
//                 action: "Select Date".to_string(),
//                 style: KeyHintStyle::Important,
//             });
//         }
//
//         hints.push(KeyHint {
//             key: "ESC".to_string(),
//             action: "Exit".to_string(),
//             style: KeyHintStyle::Subtle,
//         });
//
//         hints
//     }
// }
