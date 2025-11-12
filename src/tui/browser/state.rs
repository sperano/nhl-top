use crate::tui::widgets::Container;

// Commented out for refactoring - will reactivate later
// use super::content::BrowserContent;
// use super::link::Link;
// use super::target::Target;

/// BrowserState manages the state of the browser tab
pub struct State {
    pub container: Option<Container>,
    pub subtab_focused: bool,

    // Commented out for refactoring - will reactivate later
    // /// The content being displayed
    // pub content: BrowserContent,
    // /// Index of the currently selected link (None if no links)
    // pub selected_link_index: Option<usize>,
    // /// Scroll offset for future scrolling support
    // pub scroll_offset: u16,
}

impl State {
    pub fn new() -> Self {
        Self {
            container: None,
            subtab_focused: false,
        }
    }

    // Commented out for refactoring - will reactivate later
    // /// Create a new BrowserState with demo content
    // pub fn new() -> Self {
    //     let content = Self::create_demo_content();
    //     let selected = if content.links.is_empty() {
    //         None
    //     } else {
    //         Some(0)
    //     };
    //
    //     Self {
    //         content,
    //         selected_link_index: selected,
    //         scroll_offset: 0,
    //         subtab_focused: false,
    //     }
    // }
    //
    // /// Create the demo content with NHL-related links
    // fn create_demo_content() -> BrowserContent {
    //     BrowserContent::builder()
    //         .link("Nick Suzuki", Target::Player { id: 8480018 })
    //         .text(" plays for the ")
    //         .link("Canadiens", Target::Team { id: "MTL".to_string() })
    //         .text(", and was drafted by the ")
    //         .link("Golden Knights", Target::Team { id: "VGK".to_string() })
    //         .text(".")
    //         .build()
    // }

    // Commented out for refactoring - will reactivate later
    // /// Move to the next link (wraps around to first)
    // pub fn select_next_link(&mut self) {
    //     if self.content.links.is_empty() {
    //         self.selected_link_index = None;
    //         return;
    //     }
    //
    //     self.selected_link_index = Some(match self.selected_link_index {
    //         Some(idx) => {
    //             if idx + 1 >= self.content.links.len() {
    //                 0 // Wrap to first
    //             } else {
    //                 idx + 1
    //             }
    //         }
    //         None => 0,
    //     });
    // }
    //
    // /// Move to the previous link (wraps around to last)
    // pub fn select_previous_link(&mut self) {
    //     if self.content.links.is_empty() {
    //         self.selected_link_index = None;
    //         return;
    //     }
    //
    //     self.selected_link_index = Some(match self.selected_link_index {
    //         Some(idx) => {
    //             if idx == 0 {
    //                 self.content.links.len() - 1 // Wrap to last
    //             } else {
    //                 idx - 1
    //             }
    //         }
    //         None => 0,
    //     });
    // }
    //
    // /// Get the currently selected link
    // pub fn get_selected_link(&self) -> Option<&Link> {
    //     self.selected_link_index
    //         .and_then(|idx| self.content.links.get(idx))
    // }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

// Commented out for refactoring - will reactivate later
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_state_new() {
//         let state = State::new();
//
//         assert!(!state.content.links.is_empty());
//         assert_eq!(state.selected_link_index, Some(0));
//         assert_eq!(state.scroll_offset, 0);
//     }
//
//     #[test]
//     fn test_demo_content_structure() {
//         let state = State::new();
//
//         assert_eq!(state.content.lines.len(), 1);
//         assert_eq!(state.content.links.len(), 3);
//
//         // Verify the three links
//         assert_eq!(state.content.links[0].display, "Nick Suzuki");
//         assert_eq!(state.content.links[0].target, Target::Player { id: 8480018 });
//
//         assert_eq!(state.content.links[1].display, "Canadiens");
//         assert_eq!(state.content.links[1].target, Target::Team { id: "MTL".to_string() });
//
//         assert_eq!(state.content.links[2].display, "Golden Knights");
//         assert_eq!(state.content.links[2].target, Target::Team { id: "VGK".to_string() });
//     }
//
//     #[test]
//     fn test_demo_content_text() {
//         let state = State::new();
//
//         let expected = "Nick Suzuki plays for the Canadiens, and was drafted by the Golden Knights.";
//         assert_eq!(state.content.lines[0], expected);
//     }
//
//     #[test]
//     fn test_select_next_link() {
//         let mut state = State::new();
//
//         assert_eq!(state.selected_link_index, Some(0));
//
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(1));
//
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(2));
//
//         // Should wrap around to 0
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(0));
//     }
//
//     #[test]
//     fn test_select_previous_link() {
//         let mut state = State::new();
//
//         assert_eq!(state.selected_link_index, Some(0));
//
//         // Should wrap around to last link (index 2)
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(2));
//
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(1));
//
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(0));
//     }
//
//     #[test]
//     fn test_select_next_then_previous() {
//         let mut state = State::new();
//
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(1));
//
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(0));
//     }
//
//     #[test]
//     fn test_get_selected_link() {
//         let state = State::new();
//
//         let link = state.get_selected_link();
//         assert!(link.is_some());
//         assert_eq!(link.unwrap().display, "Nick Suzuki");
//     }
//
//     #[test]
//     fn test_get_selected_link_after_navigation() {
//         let mut state = State::new();
//
//         state.select_next_link();
//         let link = state.get_selected_link();
//         assert!(link.is_some());
//         assert_eq!(link.unwrap().display, "Canadiens");
//
//         state.select_next_link();
//         let link = state.get_selected_link();
//         assert!(link.is_some());
//         assert_eq!(link.unwrap().display, "Golden Knights");
//     }
//
//     #[test]
//     fn test_empty_content_navigation() {
//         let mut state = State {
//             content: BrowserContent::builder().build(),
//             selected_link_index: None,
//             scroll_offset: 0,
//             subtab_focused: false,
//         };
//
//         assert_eq!(state.selected_link_index, None);
//
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, None);
//
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, None);
//
//         assert!(state.get_selected_link().is_none());
//     }
//
//     #[test]
//     fn test_single_link_navigation() {
//         let mut state = State {
//             content: BrowserContent::builder()
//                 .link("OnlyLink", Target::Team { id: "MTL".to_string() })
//                 .build(),
//             selected_link_index: Some(0),
//             scroll_offset: 0,
//             subtab_focused: false,
//         };
//
//         assert_eq!(state.selected_link_index, Some(0));
//
//         // Navigating next on a single link should stay at 0
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(0));
//
//         // Navigating previous on a single link should stay at 0
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(0));
//     }
//
//     #[test]
//     fn test_wrap_around_multiple_times() {
//         let mut state = State::new();
//
//         // Wrap forward multiple times
//         for _ in 0..10 {
//             state.select_next_link();
//         }
//         // 10 % 3 = 1, so should be at index 1
//         assert_eq!(state.selected_link_index, Some(1));
//
//         // Wrap backward multiple times
//         for _ in 0..10 {
//             state.select_previous_link();
//         }
//         // Started at 1, went back 10 times
//         // (1 - 10) % 3 = -9 % 3, which wraps to 0
//         assert_eq!(state.selected_link_index, Some(0));
//     }
//
//     #[test]
//     fn test_state_from_none_index() {
//         let mut state = State {
//             content: BrowserContent::builder()
//                 .link("Link1", Target::Team { id: "MTL".to_string() })
//                 .link("Link2", Target::Team { id: "TOR".to_string() })
//                 .build(),
//             selected_link_index: None,
//             scroll_offset: 0,
//             subtab_focused: false,
//         };
//
//         assert_eq!(state.selected_link_index, None);
//
//         state.select_next_link();
//         assert_eq!(state.selected_link_index, Some(0));
//
//         state.selected_link_index = None;
//
//         state.select_previous_link();
//         assert_eq!(state.selected_link_index, Some(0));
//     }
// }
