use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Paragraph, Wrap},
};

/// A wrapper that makes any content scrollable
pub struct Scrollable {
    pub scroll_offset: u16,
    pub content_height: usize,
    pub viewport_height: u16,
}

impl Scrollable {
    pub fn new() -> Self {
        Scrollable {
            scroll_offset: 0,
            content_height: 0,
            viewport_height: 0,
        }
    }

    /// Handle scroll keys (Up, Down, PageUp, PageDown, Home, End)
    /// Returns true if the key was handled
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                true
            }
            KeyCode::Down => {
                self.scroll_down(1);
                true
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                true
            }
            KeyCode::PageDown => {
                self.scroll_down(10);
                true
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                true
            }
            KeyCode::End => {
                self.scroll_to_bottom();
                true
            }
            _ => false,
        }
    }

    /// Scroll down by n lines, but don't scroll past the bottom
    fn scroll_down(&mut self, n: u16) {
        let max_scroll = self.max_scroll();
        self.scroll_offset = (self.scroll_offset + n).min(max_scroll);
    }

    /// Scroll to the bottom
    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll();
    }

    /// Calculate the maximum scroll offset
    fn max_scroll(&self) -> u16 {
        if self.content_height as u16 > self.viewport_height {
            (self.content_height as u16) - self.viewport_height
        } else {
            0
        }
    }

    /// Update the viewport height (call this during render)
    pub fn update_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
        // Ensure scroll offset is still valid
        let max = self.max_scroll();
        if self.scroll_offset > max {
            self.scroll_offset = max;
        }
    }

    /// Update the content height (call this when content changes)
    pub fn update_content_height(&mut self, height: usize) {
        self.content_height = height;
        // Ensure scroll offset is still valid
        let max = self.max_scroll();
        if self.scroll_offset > max {
            self.scroll_offset = max;
        }
    }

    /// Render scrollable content using a Paragraph widget
    pub fn render_paragraph(
        &mut self,
        f: &mut Frame,
        area: Rect,
        content: String,
        block: Option<Block>,
    ) {
        // Update viewport height (subtract borders if block is present)
        let viewport_height = if block.is_some() {
            area.height.saturating_sub(2) // Account for top and bottom borders
        } else {
            area.height
        };
        self.update_viewport_height(viewport_height);

        // Count lines in content
        let line_count = content.lines().count();
        self.update_content_height(line_count);

        let paragraph = if let Some(b) = block {
            Paragraph::new(content)
                .block(b)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll_offset, 0))
        } else {
            Paragraph::new(content)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll_offset, 0))
        };

        f.render_widget(paragraph, area);
    }

    /// Get scroll indicator text (e.g., "Line 5/20")
    pub fn scroll_indicator(&self) -> String {
        if self.content_height == 0 {
            return String::new();
        }

        let current_line = self.scroll_offset + 1;
        let total_lines = self.content_height;

        if total_lines as u16 <= self.viewport_height {
            // Content fits entirely in viewport
            String::new()
        } else {
            format!("Line {}/{}", current_line, total_lines)
        }
    }
}

impl Default for Scrollable {
    fn default() -> Self {
        Self::new()
    }
}
