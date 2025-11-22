//! Document system for unbounded content with viewport-based scrolling
//!
//! This module provides a document abstraction that allows content of any height
//! to be rendered with viewport-based scrolling. Key features:
//!
//! - **Unbounded content**: Documents render at their natural height
//! - **Viewport scrolling**: Only the visible portion is displayed
//! - **Tab/Shift-Tab navigation**: Focus cycles through focusable elements
//! - **Autoscrolling**: Viewport automatically follows focused element
//! - **Focus highlighting**: Currently focused element is highlighted

pub mod builder;
pub mod elements;
pub mod focus;
pub mod link;
pub mod viewport;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::sync::Arc;

use crate::config::DisplayConfig;

pub use builder::DocumentBuilder;
pub use elements::DocumentElement;
pub use focus::{FocusManager, FocusableElement};
pub use link::{DocumentLink, DocumentType, LinkParams, LinkTarget};
pub use viewport::Viewport;

/// A document represents unbounded content that can be scrolled through a viewport
pub trait Document: Send + Sync {
    /// Build the document's element tree
    fn build(&self) -> Vec<DocumentElement>;

    /// Get the document's title for navigation/history
    fn title(&self) -> String;

    /// Get the document's unique ID
    fn id(&self) -> String;

    /// Calculate the total height needed to render all elements
    fn calculate_height(&self) -> u16 {
        self.build().iter().map(|elem| elem.height()).sum()
    }

    /// Render the document to a buffer at full height
    /// Returns the buffer and the actual height used
    fn render_full(&self, width: u16, config: &DisplayConfig) -> (Buffer, u16) {
        let elements = self.build();
        let height = elements.iter().map(|e| e.height()).sum();

        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        let mut y_offset = 0;

        for element in elements {
            let element_height = element.height();
            let area = Rect::new(0, y_offset, width, element_height);
            element.render(area, &mut buffer, config);
            y_offset += element_height;
        }

        (buffer, height)
    }
}

/// Container that holds a document and manages viewport/scrolling/focus
pub struct DocumentView {
    document: Arc<dyn Document>,
    viewport: Viewport,
    focus_manager: FocusManager,
    /// Pre-rendered full document buffer
    full_buffer: Option<Buffer>,
    /// Cached document height
    cached_height: u16,
}

impl DocumentView {
    /// Create a new document view
    ///
    /// # Arguments
    /// - `document`: The document to display
    /// - `viewport_height`: Height of the visible viewport
    pub fn new(document: Arc<dyn Document>, viewport_height: u16) -> Self {
        let doc_height = document.calculate_height();
        let viewport = Viewport::new(0, viewport_height, doc_height);

        // Build focus manager from document elements
        let elements = document.build();
        let focus_manager = FocusManager::from_elements(&elements);

        Self {
            document,
            viewport,
            focus_manager,
            full_buffer: None,
            cached_height: doc_height,
        }
    }

    /// Update viewport height (e.g., on terminal resize)
    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport.set_height(height);
    }

    /// Get the current viewport offset
    pub fn viewport_offset(&self) -> u16 {
        self.viewport.offset()
    }

    /// Get the viewport
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get the focus manager
    pub fn focus_manager(&self) -> &FocusManager {
        &self.focus_manager
    }

    // === Manual Scroll Operations ===

    /// Scroll the viewport up by a number of lines
    pub fn scroll_up(&mut self, lines: u16) {
        self.viewport.scroll_up(lines);
    }

    /// Scroll the viewport down by a number of lines
    pub fn scroll_down(&mut self, lines: u16) {
        self.viewport.scroll_down(lines);
    }

    /// Scroll to the top of the document
    pub fn scroll_to_top(&mut self) {
        self.viewport.scroll_to_top();
    }

    /// Scroll to the bottom of the document
    pub fn scroll_to_bottom(&mut self) {
        self.viewport.scroll_to_bottom();
    }

    /// Page up (scroll by viewport height - overlap)
    pub fn page_up(&mut self) {
        let page_size = self.viewport.height().saturating_sub(2);
        self.viewport.scroll_up(page_size);
    }

    /// Page down (scroll by viewport height - overlap)
    pub fn page_down(&mut self) {
        let page_size = self.viewport.height().saturating_sub(2);
        self.viewport.scroll_down(page_size);
    }

    // === Focus Navigation with Autoscrolling ===

    /// Navigate focus forward (Tab) with autoscrolling
    ///
    /// When focus moves to a new element, the viewport automatically scrolls
    /// to keep the focused element visible. If focus wraps from last to first
    /// element, the viewport scrolls to the top.
    pub fn focus_next(&mut self) -> bool {
        let prev_focus = self.focus_manager.current_index();

        if self.focus_manager.focus_next() {
            // Check if we wrapped around (from last to first)
            let wrapped = self.focus_manager.did_wrap_forward(prev_focus);

            if wrapped {
                // Wrapped to first element, scroll to top
                self.viewport.scroll_to_top();
            } else {
                // Normal navigation, ensure new focus is visible with padding
                self.autoscroll_to_focused();
            }
            true
        } else {
            false
        }
    }

    /// Navigate focus backward (Shift-Tab) with autoscrolling
    ///
    /// When focus moves to a new element, the viewport automatically scrolls
    /// to keep the focused element visible. If focus wraps from first to last
    /// element, the viewport scrolls to show the last element.
    pub fn focus_prev(&mut self) -> bool {
        let prev_focus = self.focus_manager.current_index();

        if self.focus_manager.focus_prev() {
            // Check if we wrapped around (from first to last)
            let wrapped = self.focus_manager.did_wrap_backward(prev_focus);

            if wrapped {
                // Wrapped to last element, scroll to show it
                if let Some(rect) = self.focus_manager.get_focused_rect() {
                    let element_bottom = rect.y + rect.height;
                    let desired_offset = element_bottom.saturating_sub(self.viewport.height());
                    self.viewport.set_offset(desired_offset);
                }
            } else {
                // Normal navigation, ensure new focus is visible with padding
                self.autoscroll_to_focused();
            }
            true
        } else {
            false
        }
    }

    /// Autoscroll to make the focused element visible with smart padding
    fn autoscroll_to_focused(&mut self) {
        if let Some(rect) = self.focus_manager.get_focused_rect() {
            let padding = self.viewport.smart_padding();
            self.viewport
                .ensure_visible_with_padding(rect.y, rect.height, padding);
        }
    }

    /// Focus a specific element by ID with autoscrolling
    pub fn focus_element_by_id(&mut self, id: &str) -> bool {
        if self.focus_manager.focus_by_id(id) {
            self.autoscroll_to_focused();
            true
        } else {
            false
        }
    }

    /// Clear focus (no element focused)
    pub fn clear_focus(&mut self) {
        self.focus_manager.clear_focus();
    }

    /// Focus a specific element by index
    pub fn focus_by_index(&mut self, index: usize) {
        self.focus_manager.focus_by_index(index);
    }

    /// Set the scroll offset directly
    pub fn set_scroll_offset(&mut self, offset: u16) {
        self.viewport.set_offset(offset);
    }

    // === Link Activation ===

    /// Activate the currently focused element (Enter)
    ///
    /// Returns a LinkTarget if a link was activated.
    pub fn activate_focused(&self) -> Option<LinkTarget> {
        self.focus_manager.activate_current()
    }

    /// Get the currently focused element's link target (if any)
    pub fn get_focused_link(&self) -> Option<&LinkTarget> {
        self.focus_manager.get_current_link()
    }

    // === Rendering ===

    /// Render the visible portion of the document
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Render full document if not cached or dimensions changed
        if self.full_buffer.is_none()
            || self.full_buffer.as_ref().unwrap().area.width != area.width
        {
            let (full_buf, height) = self.document.render_full(area.width, config);
            self.full_buffer = Some(full_buf);
            self.cached_height = height;
            self.viewport.set_content_height(height);
        }

        // Copy visible portion from full buffer to output buffer
        if let Some(full_buffer) = &self.full_buffer {
            let visible_range = self.viewport.visible_range();

            for y in visible_range.clone() {
                if y >= self.cached_height {
                    break;
                }

                let src_y = y;
                let dst_y = area.y + (y - visible_range.start);

                if dst_y >= area.y + area.height {
                    break;
                }

                for x in 0..area.width {
                    let src_idx = (src_y * area.width + x) as usize;
                    let dst_idx = (dst_y * buf.area.width + (area.x + x)) as usize;

                    if src_idx < full_buffer.content.len() && dst_idx < buf.content.len() {
                        buf.content[dst_idx] = full_buffer.content[src_idx].clone();
                    }
                }
            }

            // Highlight focused element if visible
            self.render_focus_highlight(area, buf, config);
        }
    }

    /// Render focus highlight for the currently focused element
    fn render_focus_highlight(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        use ratatui::style::Style;

        if let Some(focused_rect) = self.focus_manager.get_focused_rect() {
            if self.viewport.is_rect_visible(&focused_rect) {
                // Adjust rect to viewport coordinates
                let viewport_offset = self.viewport.offset();
                let adjusted_rect = Rect::new(
                    area.x + focused_rect.x,
                    area.y + focused_rect.y.saturating_sub(viewport_offset),
                    focused_rect.width,
                    focused_rect.height,
                );

                // Create selection style from config
                let selection_style = Style::default().fg(config.selection_fg);

                // Apply focus highlighting
                for y in adjusted_rect.top()..adjusted_rect.bottom() {
                    for x in adjusted_rect.left()..adjusted_rect.right() {
                        if x < area.x + area.width && y < area.y + area.height {
                            let idx = (y * buf.area.width + x) as usize;
                            if idx < buf.content.len() {
                                buf.content[idx].set_style(selection_style);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Invalidate the render cache (call when document content changes)
    pub fn invalidate_cache(&mut self) {
        self.full_buffer = None;
    }

    /// Rebuild the document (rebuilds element tree and focus manager)
    pub fn rebuild(&mut self) {
        let elements = self.document.build();
        self.focus_manager = FocusManager::from_elements(&elements);
        self.cached_height = elements.iter().map(|e| e.height()).sum();
        self.viewport.set_content_height(self.cached_height);
        self.invalidate_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test document for testing
    struct TestDocument {
        lines: usize,
        links: usize,
    }

    impl TestDocument {
        fn new(lines: usize, links: usize) -> Self {
            Self { lines, links }
        }
    }

    impl Document for TestDocument {
        fn build(&self) -> Vec<DocumentElement> {
            let mut elements = Vec::new();

            elements.push(DocumentElement::heading(1, "Test Document"));

            for i in 0..self.lines {
                elements.push(DocumentElement::text(format!("Line {}", i)));
            }

            for i in 0..self.links {
                elements.push(DocumentElement::link(
                    format!("link_{}", i),
                    format!("Link {}", i),
                    LinkTarget::Action(format!("action_{}", i)),
                ));
            }

            elements
        }

        fn title(&self) -> String {
            "Test Document".to_string()
        }

        fn id(&self) -> String {
            "test_doc".to_string()
        }
    }

    #[test]
    fn test_document_view_new() {
        let doc = Arc::new(TestDocument::new(10, 3));
        let view = DocumentView::new(doc, 20);

        assert_eq!(view.viewport_offset(), 0);
        assert_eq!(view.focus_manager().len(), 3); // 3 links
    }

    #[test]
    fn test_focus_next_basic() {
        let doc = Arc::new(TestDocument::new(5, 3));
        let mut view = DocumentView::new(doc, 20);

        // First tab should focus first link
        assert!(view.focus_next());
        assert_eq!(view.focus_manager().current_index(), Some(0));

        // Second tab should focus second link
        assert!(view.focus_next());
        assert_eq!(view.focus_manager().current_index(), Some(1));

        // Third tab should focus third link
        assert!(view.focus_next());
        assert_eq!(view.focus_manager().current_index(), Some(2));

        // Fourth tab should wrap to first
        assert!(view.focus_next());
        assert_eq!(view.focus_manager().current_index(), Some(0));
    }

    #[test]
    fn test_focus_prev_basic() {
        let doc = Arc::new(TestDocument::new(5, 3));
        let mut view = DocumentView::new(doc, 20);

        // First shift-tab should focus last link
        assert!(view.focus_prev());
        assert_eq!(view.focus_manager().current_index(), Some(2));

        // Second shift-tab should focus second link
        assert!(view.focus_prev());
        assert_eq!(view.focus_manager().current_index(), Some(1));
    }

    #[test]
    fn test_focus_next_wraps_and_scrolls_to_top() {
        let doc = Arc::new(TestDocument::new(50, 3));
        let mut view = DocumentView::new(doc, 10);

        // Navigate to last element
        view.focus_next(); // 0
        view.focus_next(); // 1
        view.focus_next(); // 2 (last)

        // Scroll down so we're not at top
        view.scroll_down(30);
        assert!(view.viewport_offset() > 0);

        // Wrap to first should scroll to top
        view.focus_next(); // wraps to 0
        assert_eq!(view.viewport_offset(), 0);
    }

    #[test]
    fn test_autoscroll_when_focusing_off_screen() {
        // Create a document with many lines and links spread throughout
        let doc = Arc::new(TestDocument::new(100, 50));
        let mut view = DocumentView::new(doc, 10);

        // Tab through many elements
        for _ in 0..30 {
            view.focus_next();
        }

        // The focused element should be visible
        if let Some(rect) = view.focus_manager().get_focused_rect() {
            let visible = view.viewport().visible_range();
            assert!(
                rect.y >= visible.start && rect.y < visible.end,
                "Focused element at y={} not visible in range {:?}",
                rect.y,
                visible
            );
        }
    }

    #[test]
    fn test_scroll_operations() {
        let doc = Arc::new(TestDocument::new(100, 0));
        let mut view = DocumentView::new(doc, 10);

        assert_eq!(view.viewport_offset(), 0);

        view.scroll_down(5);
        assert_eq!(view.viewport_offset(), 5);

        view.scroll_up(3);
        assert_eq!(view.viewport_offset(), 2);

        view.scroll_to_bottom();
        assert!(view.viewport().is_at_bottom());

        view.scroll_to_top();
        assert_eq!(view.viewport_offset(), 0);
    }

    #[test]
    fn test_page_operations() {
        let doc = Arc::new(TestDocument::new(100, 0));
        let mut view = DocumentView::new(doc, 20);

        view.page_down();
        assert_eq!(view.viewport_offset(), 18); // 20 - 2 overlap

        view.page_up();
        assert_eq!(view.viewport_offset(), 0);
    }

    #[test]
    fn test_focus_element_by_id() {
        let doc = Arc::new(TestDocument::new(5, 5));
        let mut view = DocumentView::new(doc, 10);

        assert!(view.focus_element_by_id("link_2"));
        assert_eq!(view.focus_manager().current_index(), Some(2));

        assert!(!view.focus_element_by_id("nonexistent"));
    }

    #[test]
    fn test_activate_focused() {
        let doc = Arc::new(TestDocument::new(5, 3));
        let mut view = DocumentView::new(doc, 10);

        // No focus yet
        assert!(view.activate_focused().is_none());

        // Focus first link
        view.focus_next();
        let target = view.activate_focused();
        assert!(matches!(target, Some(LinkTarget::Action(_))));
    }

    #[test]
    fn test_clear_focus() {
        let doc = Arc::new(TestDocument::new(5, 3));
        let mut view = DocumentView::new(doc, 10);

        view.focus_next();
        assert!(view.focus_manager().current_index().is_some());

        view.clear_focus();
        assert!(view.focus_manager().current_index().is_none());
    }

    #[test]
    fn test_set_viewport_height() {
        let doc = Arc::new(TestDocument::new(10, 0));
        let mut view = DocumentView::new(doc, 10);

        view.set_viewport_height(20);
        assert_eq!(view.viewport().height(), 20);
    }

    #[test]
    fn test_empty_document() {
        struct EmptyDoc;
        impl Document for EmptyDoc {
            fn build(&self) -> Vec<DocumentElement> {
                Vec::new()
            }
            fn title(&self) -> String {
                "Empty".to_string()
            }
            fn id(&self) -> String {
                "empty".to_string()
            }
        }

        let doc = Arc::new(EmptyDoc);
        let mut view = DocumentView::new(doc, 10);

        // Focus operations should return false
        assert!(!view.focus_next());
        assert!(!view.focus_prev());
    }

    #[test]
    fn test_document_with_no_focusable_elements() {
        let doc = Arc::new(TestDocument::new(50, 0)); // No links
        let mut view = DocumentView::new(doc, 10);

        assert!(!view.focus_next());
        assert_eq!(view.focus_manager().len(), 0);
    }
}
