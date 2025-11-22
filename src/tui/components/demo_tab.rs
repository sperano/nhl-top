//! Demo tab component showcasing the document system
//!
//! This tab demonstrates the document system capabilities including:
//! - Document elements (headings, text, links, separators)
//! - Viewport-based scrolling
//! - Tab/Shift-Tab focus navigation through focusable elements
//! - Autoscrolling to keep focused elements visible
//! - Embedded tables (league standings) rendered at natural height

use std::sync::Arc;

use nhl_api::Standing;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::config::DisplayConfig;
use crate::tui::component::{Component, Effect, Element, ElementWidget};
use crate::tui::document::{Document, DocumentBuilder, DocumentElement, DocumentView, LinkTarget};

/// Build a DemoDocument and return the y-positions of its focusable elements
///
/// This allows external code (like reducers) to get accurate focusable positions
/// when the document's underlying data changes (e.g., standings load).
pub fn build_demo_focusable_positions(standings: Option<&Vec<Standing>>) -> Vec<u16> {
    let doc = DemoDocument::new(standings.cloned());
    doc.focusable_positions()
}

/// Props for the Demo tab
#[derive(Clone)]
pub struct DemoTabProps {
    /// Whether the tab content is focused
    pub content_focused: bool,
    /// Current focus index from AppState (None = no focus)
    pub focus_index: Option<usize>,
    /// Current scroll offset from AppState
    pub scroll_offset: u16,
    /// Standings data for demonstrating embedded tables
    pub standings: Arc<Option<Vec<Standing>>>,
}

/// State for the Demo tab
#[derive(Clone, Default)]
pub struct DemoTabState {
    /// Focus index within the document (0-based)
    focus_index: Option<usize>,
    /// Scroll offset
    scroll_offset: u16,
}

/// Messages that can be sent to the Demo tab
pub enum DemoTabMessage {
    /// Tab key pressed - focus next element
    FocusNext,
    /// Shift-Tab key pressed - focus previous element
    FocusPrev,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Activate focused element
    Activate,
}

/// Demo tab component
pub struct DemoTab;

impl Component for DemoTab {
    type Props = DemoTabProps;
    type State = DemoTabState;
    type Message = DemoTabMessage;

    fn init(_props: &Self::Props) -> Self::State {
        DemoTabState::default()
    }

    fn update(&mut self, msg: Self::Message, _state: &mut Self::State) -> Effect {
        // Note: In a real implementation, focus and scroll state would be managed
        // via AppState. For this demo, we just log the messages.
        match msg {
            DemoTabMessage::FocusNext => {
                tracing::debug!("Demo: focus_next requested");
            }
            DemoTabMessage::FocusPrev => {
                tracing::debug!("Demo: focus_prev requested");
            }
            DemoTabMessage::ScrollUp => {
                tracing::debug!("Demo: scroll_up requested");
            }
            DemoTabMessage::ScrollDown => {
                tracing::debug!("Demo: scroll_down requested");
            }
            DemoTabMessage::PageUp => {
                tracing::debug!("Demo: page_up requested");
            }
            DemoTabMessage::PageDown => {
                tracing::debug!("Demo: page_down requested");
            }
            DemoTabMessage::Activate => {
                tracing::debug!("Demo: activate requested");
            }
        }

        Effect::None
    }

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(DemoTabWidget {
            content_focused: props.content_focused,
            focus_index: props.focus_index,
            scroll_offset: props.scroll_offset,
            standings: props.standings.clone(),
        }))
    }
}

/// Widget for rendering the Demo tab
struct DemoTabWidget {
    content_focused: bool,
    focus_index: Option<usize>,
    scroll_offset: u16,
    standings: Arc<Option<Vec<Standing>>>,
}

impl ElementWidget for DemoTabWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Create document view with state from AppState
        let standings = (*self.standings).clone();
        let doc = Arc::new(DemoDocument::new(standings));
        let mut view = DocumentView::new(doc, area.height);

        // Apply focus state from AppState
        if let Some(idx) = self.focus_index {
            view.focus_by_index(idx);
        }

        // Apply scroll offset from AppState
        view.set_scroll_offset(self.scroll_offset);

        view.render(area, buf, config);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(DemoTabWidget {
            content_focused: self.content_focused,
            focus_index: self.focus_index,
            scroll_offset: self.scroll_offset,
            standings: self.standings.clone(),
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Fills available space
    }
}

/// Demo document showcasing all document element types
struct DemoDocument {
    standings: Option<Vec<Standing>>,
}

impl DemoDocument {
    fn new(standings: Option<Vec<Standing>>) -> Self {
        Self { standings }
    }

    /// Build standings table lines as text elements
    fn build_standings_section(&self, builder: DocumentBuilder) -> DocumentBuilder {
        let builder = builder
            .heading(2, "League Standings (Natural Height)")
            .spacer(1)
            .text("This demonstrates embedded data rendered at natural height:");

        match &self.standings {
            Some(standings) if !standings.is_empty() => {
                // Sort by points (highest first)
                let mut sorted = standings.clone();
                sorted.sort_by(|a, b| b.points.cmp(&a.points));

                // Header
                let builder = builder
                    .spacer(1)
                    .text("Rank  Team                     GP   W   L  OT  PTS")
                    .text("────  ───────────────────────  ──  ──  ──  ──  ───");

                // Add each team as a text line with a link
                let mut b = builder;
                for (i, standing) in sorted.iter().enumerate() {
                    let rank = i + 1;
                    let team_name = &standing.team_common_name.default;
                    let abbrev = &standing.team_abbrev.default;

                    // Create a focusable link for each team
                    let link_id = format!("standings_{}", abbrev);
                    let display = format!(
                        "{:>4}  {:<23}  {:>2}  {:>2}  {:>2}  {:>2}  {:>3}",
                        rank,
                        team_name,
                        standing.games_played(),
                        standing.wins,
                        standing.losses,
                        standing.ot_losses,
                        standing.points
                    );
                    b = b.link_with_id(&link_id, &display, LinkTarget::Action(format!("team:{}", abbrev)));
                }
                b
            }
            _ => {
                builder.text("(No standings data loaded - try refreshing)")
            }
        }
    }
}

impl Document for DemoDocument {
    fn build(&self) -> Vec<DocumentElement> {
        let builder = DocumentBuilder::new()
            .heading(1, "Document System Demo")
            .spacer(1)
            .text("This tab demonstrates the new document system for the NHL TUI.")
            .text("Press Tab/Shift-Tab to navigate between focusable elements.")
            .spacer(1)
            .separator()
            .spacer(1);

        // Add standings section first (showcases natural height rendering)
        let builder = self.build_standings_section(builder);

        // Then add the rest of the demo content
        builder
            .spacer(1)
            .separator()
            .spacer(1)
            .heading(2, "Features")
            .text("- Viewport-based scrolling for unlimited content height")
            .text("- Tab/Shift-Tab navigation cycles through focusable elements")
            .text("- Autoscrolling keeps the focused element visible")
            .text("- Smart padding positions elements comfortably in view")
            .text("- Focus wrapping scrolls to top/bottom automatically")
            .spacer(1)
            .heading(2, "Example Links")
            .text("These links demonstrate focusable elements:")
            .spacer(1)
            .link_with_id("link_bos", "Boston Bruins", LinkTarget::Action("team:BOS".to_string()))
            .spacer(1)
            .link_with_id("link_tor", "Toronto Maple Leafs", LinkTarget::Action("team:TOR".to_string()))
            .spacer(1)
            .link_with_id("link_nyr", "New York Rangers", LinkTarget::Action("team:NYR".to_string()))
            .spacer(1)
            .link_with_id("link_mtl", "Montreal Canadiens", LinkTarget::Action("team:MTL".to_string()))
            .spacer(1)
            .separator()
            .spacer(1)
            .heading(2, "Implementation Notes")
            .text("The document system consists of several modules:")
            .spacer(1)
            .text("- viewport.rs: Manages scroll position and visible range")
            .text("- focus.rs: Tracks focusable elements and navigation")
            .text("- elements.rs: Document element types (text, headings, links)")
            .text("- builder.rs: Fluent API for building documents")
            .text("- mod.rs: Document trait and DocumentView container")
            .spacer(1)
            .text("Each document implements the Document trait to define its")
            .text("content structure. DocumentView manages the viewport and")
            .text("focus state for rendering and interaction.")
            .spacer(1)
            .text("End of demo document.")
            .build()
    }

    fn title(&self) -> String {
        "Document System Demo".to_string()
    }

    fn id(&self) -> String {
        "demo".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::assert_buffer;

    #[test]
    fn test_demo_document_builds() {
        let doc = DemoDocument::new(None);
        let elements = doc.build();

        // Should have multiple elements
        assert!(elements.len() > 10);
    }

    #[test]
    fn test_demo_document_height() {
        let doc = DemoDocument::new(None);
        let height = doc.calculate_height();

        // Should have significant height (all the content)
        assert!(height > 30);
    }

    #[test]
    fn test_demo_tab_renders() {
        let props = DemoTabProps {
            content_focused: false,
            focus_index: None,
            scroll_offset: 0,
            standings: Arc::new(None),
        };
        let state = DemoTabState::default();
        let demo_tab = DemoTab;

        let element = demo_tab.view(&props, &state);

        // Should return a widget element
        assert!(matches!(element, Element::Widget(_)));
    }

    #[test]
    fn test_demo_document_focusable_count_no_standings() {
        let doc = DemoDocument::new(None);
        let doc_arc = Arc::new(doc);
        let view = DocumentView::new(doc_arc, 20);

        // Should have 4 focusable links (BOS, TOR, NYR, MTL)
        assert_eq!(view.focus_manager().len(), 4);
    }

    #[test]
    fn test_demo_tab_widget_render() {
        let widget = DemoTabWidget {
            content_focused: true,
            focus_index: None,
            scroll_offset: 0,
            standings: Arc::new(None),
        };

        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 5));
        let config = DisplayConfig::default();

        widget.render(buf.area, &mut buf, &config);

        // Should render the heading and first lines of content
        assert_buffer(&buf, &[
            "Document System Demo",
            "════════════════════",
            "",
            "This tab demonstrates the new document system for the NHL TU",
            "Press Tab/Shift-Tab to navigate between focusable elements.",
        ]);
    }

    #[test]
    fn test_demo_tab_focus_navigation() {
        let doc = Arc::new(DemoDocument::new(None));
        let mut view = DocumentView::new(doc, 10);

        // Initially no focus
        assert!(view.focus_manager().current_index().is_none());

        // First Tab focuses first link
        view.focus_next();
        assert_eq!(view.focus_manager().current_index(), Some(0));

        // Second Tab focuses second link
        view.focus_next();
        assert_eq!(view.focus_manager().current_index(), Some(1));

        // Shift-Tab goes back
        view.focus_prev();
        assert_eq!(view.focus_manager().current_index(), Some(0));
    }
}
