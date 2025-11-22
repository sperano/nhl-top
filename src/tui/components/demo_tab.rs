//! Demo tab component showcasing the document system
//!
//! This tab demonstrates the document system capabilities including:
//! - Document elements (headings, text, links, separators)
//! - Viewport-based scrolling
//! - Tab/Shift-Tab focus navigation through focusable elements
//! - Autoscrolling to keep focused elements visible

use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::config::DisplayConfig;
use crate::tui::component::{Component, Effect, Element, ElementWidget};
use crate::tui::document::{Document, DocumentBuilder, DocumentElement, DocumentView, LinkTarget};

/// Props for the Demo tab
#[derive(Clone)]
pub struct DemoTabProps {
    /// Whether the tab content is focused
    pub content_focused: bool,
    /// Current focus index from AppState (None = no focus)
    pub focus_index: Option<usize>,
    /// Current scroll offset from AppState
    pub scroll_offset: u16,
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
        }))
    }
}

/// Widget for rendering the Demo tab
struct DemoTabWidget {
    content_focused: bool,
    focus_index: Option<usize>,
    scroll_offset: u16,
}

impl ElementWidget for DemoTabWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Create document view with state from AppState
        let doc = Arc::new(DemoDocument);
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
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Fills available space
    }
}

/// Demo document showcasing all document element types
struct DemoDocument;

impl Document for DemoDocument {
    fn build(&self) -> Vec<DocumentElement> {
        DocumentBuilder::new()
            .heading(1, "Document System Demo")
            .spacer(1)
            .text("This tab demonstrates the new document system for the NHL TUI.")
            .text("Press Tab/Shift-Tab to navigate between focusable elements.")
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
            .heading(2, "How Autoscrolling Works")
            .text("1. When Tab moves focus to an element below the viewport,")
            .text("   the viewport scrolls down to show it with padding.")
            .spacer(1)
            .text("2. When Shift-Tab moves focus to an element above the viewport,")
            .text("   the viewport scrolls up to show it with padding.")
            .spacer(1)
            .text("3. When focus wraps from the last to first element (Tab),")
            .text("   the viewport automatically scrolls to the top.")
            .spacer(1)
            .text("4. When focus wraps from the first to last element (Shift-Tab),")
            .text("   the viewport automatically scrolls to the bottom.")
            .spacer(1)
            .separator()
            .spacer(1)
            .heading(2, "More Links for Testing")
            .spacer(1)
            .link_with_id("link_edm", "Edmonton Oilers", LinkTarget::Action("team:EDM".to_string()))
            .spacer(1)
            .link_with_id("link_vgk", "Vegas Golden Knights", LinkTarget::Action("team:VGK".to_string()))
            .spacer(1)
            .link_with_id("link_col", "Colorado Avalanche", LinkTarget::Action("team:COL".to_string()))
            .spacer(1)
            .link_with_id("link_dal", "Dallas Stars", LinkTarget::Action("team:DAL".to_string()))
            .spacer(1)
            .link_with_id("link_wpg", "Winnipeg Jets", LinkTarget::Action("team:WPG".to_string()))
            .spacer(1)
            .link_with_id("link_fla", "Florida Panthers", LinkTarget::Action("team:FLA".to_string()))
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
        let doc = DemoDocument;
        let elements = doc.build();

        // Should have multiple elements
        assert!(elements.len() > 10);
    }

    #[test]
    fn test_demo_document_height() {
        let doc = DemoDocument;
        let height = doc.calculate_height();

        // Should have significant height (all the content)
        assert!(height > 50);
    }

    #[test]
    fn test_demo_tab_renders() {
        let props = DemoTabProps {
            content_focused: false,
            focus_index: None,
            scroll_offset: 0,
        };
        let state = DemoTabState::default();
        let demo_tab = DemoTab;

        let element = demo_tab.view(&props, &state);

        // Should return a widget element
        assert!(matches!(element, Element::Widget(_)));
    }

    #[test]
    fn test_demo_document_focusable_count() {
        let doc = DemoDocument;
        let doc_arc = Arc::new(doc);
        let view = DocumentView::new(doc_arc, 20);

        // Should have 10 focusable links
        assert_eq!(view.focus_manager().len(), 10);
    }

    #[test]
    fn test_demo_tab_widget_render() {
        let widget = DemoTabWidget {
            content_focused: true,
            focus_index: None,
            scroll_offset: 0,
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
        let doc = Arc::new(DemoDocument);
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
