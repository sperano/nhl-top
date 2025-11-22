//! Document elements that can be rendered in a document
//!
//! Provides various element types (text, headings, tables, links, etc.)
//! that can be composed to build documents.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::config::DisplayConfig;

use super::focus::FocusableElement;
use super::link::LinkTarget;

/// Elements that can be part of a document
#[derive(Clone)]
pub enum DocumentElement {
    /// Plain text paragraph
    Text {
        content: String,
        style: Option<Style>,
    },

    /// Heading (different levels)
    Heading {
        level: u8, // 1-6
        content: String,
    },

    /// A link that can be focused and activated
    Link {
        display: String,
        target: LinkTarget,
        id: String,
    },

    /// Horizontal separator
    Separator,

    /// Vertical spacing
    Spacer { height: u16 },

    /// Container for grouping elements
    Group {
        children: Vec<DocumentElement>,
        style: Option<Style>,
    },

    /// Raw content with pre-calculated focusable elements
    /// Used for complex widgets like tables that manage their own focus
    Custom {
        /// Render function that draws to a buffer
        render_fn: fn(Rect, &mut Buffer, &DisplayConfig),
        /// Height of the element
        height: u16,
        /// Focusable elements within this custom element
        focusable: Vec<FocusableElement>,
    },
}

impl std::fmt::Debug for DocumentElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text { content, style } => f
                .debug_struct("Text")
                .field("content", content)
                .field("style", style)
                .finish(),
            Self::Heading { level, content } => f
                .debug_struct("Heading")
                .field("level", level)
                .field("content", content)
                .finish(),
            Self::Link { display, target, id } => f
                .debug_struct("Link")
                .field("display", display)
                .field("target", target)
                .field("id", id)
                .finish(),
            Self::Separator => write!(f, "Separator"),
            Self::Spacer { height } => f.debug_struct("Spacer").field("height", height).finish(),
            Self::Group { children, style } => f
                .debug_struct("Group")
                .field("children", children)
                .field("style", style)
                .finish(),
            Self::Custom { height, focusable, .. } => f
                .debug_struct("Custom")
                .field("height", height)
                .field("focusable_count", &focusable.len())
                .finish(),
        }
    }
}

impl DocumentElement {
    /// Calculate the height this element needs
    pub fn height(&self) -> u16 {
        match self {
            Self::Text { content, .. } => {
                // Count lines in text (minimum 1)
                content.lines().count().max(1) as u16
            }
            Self::Heading { level, .. } => {
                // Level 1 headings have underline
                if *level == 1 {
                    2
                } else {
                    1
                }
            }
            Self::Link { .. } => 1,
            Self::Separator => 1,
            Self::Spacer { height } => *height,
            Self::Group { children, .. } => children.iter().map(|c| c.height()).sum(),
            Self::Custom { height, .. } => *height,
        }
    }

    /// Collect focusable elements from this element
    ///
    /// # Arguments
    /// - `out`: Vector to append focusable elements to
    /// - `y_offset`: Current y offset in the document
    pub fn collect_focusable(&self, out: &mut Vec<FocusableElement>, y_offset: u16) {
        match self {
            Self::Link { display, target, id } => {
                out.push(FocusableElement {
                    id: id.clone(),
                    y: y_offset,
                    height: 1,
                    rect: Rect::new(0, y_offset, display.chars().count() as u16, 1),
                    link_target: Some(target.clone()),
                    tab_order: 0,
                });
            }
            Self::Group { children, .. } => {
                let mut child_offset = y_offset;
                for child in children {
                    child.collect_focusable(out, child_offset);
                    child_offset += child.height();
                }
            }
            Self::Custom { focusable, .. } => {
                // Add focusable elements with adjusted y positions
                for elem in focusable {
                    let mut adjusted = elem.clone();
                    adjusted.y += y_offset;
                    adjusted.rect.y += y_offset;
                    out.push(adjusted);
                }
            }
            _ => {}
        }
    }

    /// Render this element to a buffer
    pub fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        match self {
            Self::Text { content, style } => {
                render_text(content, *style, area, buf);
            }
            Self::Heading { level, content } => {
                render_heading(*level, content, area, buf, config);
            }
            Self::Link { display, .. } => {
                render_link(display, area, buf);
            }
            Self::Separator => {
                render_separator(area, buf, config);
            }
            Self::Spacer { .. } => {
                // Just empty space, nothing to render
            }
            Self::Group { children, style } => {
                render_group(children, *style, area, buf, config);
            }
            Self::Custom { render_fn, .. } => {
                render_fn(area, buf, config);
            }
        }
    }

    /// Create a text element
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            style: None,
        }
    }

    /// Create a styled text element
    pub fn styled_text(content: impl Into<String>, style: Style) -> Self {
        Self::Text {
            content: content.into(),
            style: Some(style),
        }
    }

    /// Create a heading element
    pub fn heading(level: u8, content: impl Into<String>) -> Self {
        Self::Heading {
            level: level.clamp(1, 6),
            content: content.into(),
        }
    }

    /// Create a link element
    pub fn link(id: impl Into<String>, display: impl Into<String>, target: LinkTarget) -> Self {
        Self::Link {
            id: id.into(),
            display: display.into(),
            target,
        }
    }

    /// Create a separator element
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Create a spacer element
    pub fn spacer(height: u16) -> Self {
        Self::Spacer { height }
    }

    /// Create a group element
    pub fn group(children: Vec<DocumentElement>) -> Self {
        Self::Group {
            children,
            style: None,
        }
    }

    /// Create a styled group element
    pub fn styled_group(children: Vec<DocumentElement>, style: Style) -> Self {
        Self::Group {
            children,
            style: Some(style),
        }
    }
}

fn render_text(content: &str, style: Option<Style>, area: Rect, buf: &mut Buffer) {
    for (i, line) in content.lines().enumerate() {
        if i as u16 >= area.height {
            break;
        }
        let y = area.y + i as u16;
        for (x, ch) in line.chars().enumerate() {
            if x as u16 >= area.width {
                break;
            }
            let cell = buf.cell_mut((area.x + x as u16, y));
            if let Some(cell) = cell {
                cell.set_char(ch);
                if let Some(s) = style {
                    cell.set_style(s);
                }
            }
        }
    }
}

fn render_heading(level: u8, content: &str, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
    let style = match level {
        1 => Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED),
        2 => Style::default().add_modifier(Modifier::BOLD),
        _ => Style::default().add_modifier(Modifier::UNDERLINED),
    };

    // Render heading text
    for (x, ch) in content.chars().enumerate() {
        if x as u16 >= area.width {
            break;
        }
        let cell = buf.cell_mut((area.x + x as u16, area.y));
        if let Some(cell) = cell {
            cell.set_char(ch);
            cell.set_style(style);
        }
    }

    // Render underline for level 1
    if level == 1 && area.height > 1 {
        for x in 0..area.width.min(content.chars().count() as u16) {
            let cell = buf.cell_mut((area.x + x, area.y + 1));
            if let Some(cell) = cell {
                cell.set_char('═');
            }
        }
    }
}

fn render_link(display: &str, area: Rect, buf: &mut Buffer) {
    let style = Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED);

    for (x, ch) in display.chars().enumerate() {
        if x as u16 >= area.width {
            break;
        }
        let cell = buf.cell_mut((area.x + x as u16, area.y));
        if let Some(cell) = cell {
            cell.set_char(ch);
            cell.set_style(style);
        }
    }
}

fn render_separator(area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
    let sep_str = &config.box_chars.horizontal;
    let sep_char = sep_str.chars().next().unwrap_or('-');
    for x in 0..area.width {
        let cell = buf.cell_mut((area.x + x, area.y));
        if let Some(cell) = cell {
            cell.set_char(sep_char);
        }
    }
}

fn render_group(
    children: &[DocumentElement],
    style: Option<Style>,
    area: Rect,
    buf: &mut Buffer,
    config: &DisplayConfig,
) {
    let mut y_offset = 0;
    for child in children {
        let child_height = child.height();
        if y_offset >= area.height {
            break;
        }
        let child_area = Rect::new(
            area.x,
            area.y + y_offset,
            area.width,
            child_height.min(area.height - y_offset),
        );
        child.render(child_area, buf, config);
        y_offset += child_height;
    }

    // Apply group style if any
    if let Some(s) = style {
        for y in area.y..area.y + area.height.min(y_offset) {
            for x in area.x..area.x + area.width {
                let cell = buf.cell_mut((x, y));
                if let Some(cell) = cell {
                    let existing = cell.style();
                    cell.set_style(existing.patch(s));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::document::link::DocumentLink;

    #[test]
    fn test_text_element_height() {
        let elem = DocumentElement::text("Hello");
        assert_eq!(elem.height(), 1);

        let elem = DocumentElement::text("Line 1\nLine 2\nLine 3");
        assert_eq!(elem.height(), 3);

        let elem = DocumentElement::text("");
        assert_eq!(elem.height(), 1);
    }

    #[test]
    fn test_heading_element_height() {
        let elem = DocumentElement::heading(1, "Title");
        assert_eq!(elem.height(), 2); // Heading + underline

        let elem = DocumentElement::heading(2, "Subtitle");
        assert_eq!(elem.height(), 1);

        let elem = DocumentElement::heading(3, "Section");
        assert_eq!(elem.height(), 1);
    }

    #[test]
    fn test_link_element_height() {
        let elem = DocumentElement::link(
            "link1",
            "Click me",
            LinkTarget::Action("test".to_string()),
        );
        assert_eq!(elem.height(), 1);
    }

    #[test]
    fn test_separator_element_height() {
        let elem = DocumentElement::separator();
        assert_eq!(elem.height(), 1);
    }

    #[test]
    fn test_spacer_element_height() {
        let elem = DocumentElement::spacer(5);
        assert_eq!(elem.height(), 5);
    }

    #[test]
    fn test_group_element_height() {
        let elem = DocumentElement::group(vec![
            DocumentElement::text("Line 1"),
            DocumentElement::spacer(2),
            DocumentElement::text("Line 2"),
        ]);
        assert_eq!(elem.height(), 4); // 1 + 2 + 1
    }

    #[test]
    fn test_collect_focusable_link() {
        let elem = DocumentElement::link(
            "my_link",
            "Click here",
            LinkTarget::Document(DocumentLink::team("BOS")),
        );

        let mut focusable = Vec::new();
        elem.collect_focusable(&mut focusable, 10);

        assert_eq!(focusable.len(), 1);
        assert_eq!(focusable[0].id, "my_link");
        assert_eq!(focusable[0].y, 10);
        assert_eq!(focusable[0].rect.width, 10); // "Click here" = 10 chars
    }

    #[test]
    fn test_collect_focusable_group() {
        let elem = DocumentElement::group(vec![
            DocumentElement::text("Not focusable"),
            DocumentElement::link("link1", "First", LinkTarget::Action("a".to_string())),
            DocumentElement::spacer(2),
            DocumentElement::link("link2", "Second", LinkTarget::Action("b".to_string())),
        ]);

        let mut focusable = Vec::new();
        elem.collect_focusable(&mut focusable, 0);

        assert_eq!(focusable.len(), 2);
        assert_eq!(focusable[0].id, "link1");
        assert_eq!(focusable[0].y, 1); // After "Not focusable"
        assert_eq!(focusable[1].id, "link2");
        assert_eq!(focusable[1].y, 4); // 1 (text) + 1 (link) + 2 (spacer)
    }

    #[test]
    fn test_collect_focusable_nested_groups() {
        let elem = DocumentElement::group(vec![
            DocumentElement::group(vec![
                DocumentElement::link("inner1", "Inner", LinkTarget::Action("x".to_string())),
            ]),
            DocumentElement::link("outer1", "Outer", LinkTarget::Action("y".to_string())),
        ]);

        let mut focusable = Vec::new();
        elem.collect_focusable(&mut focusable, 5);

        assert_eq!(focusable.len(), 2);
        assert_eq!(focusable[0].id, "inner1");
        assert_eq!(focusable[0].y, 5);
        assert_eq!(focusable[1].id, "outer1");
        assert_eq!(focusable[1].y, 6);
    }

    #[test]
    fn test_render_text() {
        let elem = DocumentElement::text("Hello");
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 20, 5), &mut buf, &config);

        // Check that "Hello" was rendered
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "H");
        assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "e");
        assert_eq!(buf.cell((2, 0)).unwrap().symbol(), "l");
        assert_eq!(buf.cell((3, 0)).unwrap().symbol(), "l");
        assert_eq!(buf.cell((4, 0)).unwrap().symbol(), "o");
    }

    #[test]
    fn test_render_heading_level_1() {
        let elem = DocumentElement::heading(1, "Title");
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 20, 5), &mut buf, &config);

        // Check heading text
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "T");
        // Check underline
        assert_eq!(buf.cell((0, 1)).unwrap().symbol(), "═");
    }

    #[test]
    fn test_render_link() {
        let elem = DocumentElement::link(
            "test_link",
            "Click",
            LinkTarget::Action("test".to_string()),
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 20, 5), &mut buf, &config);

        // Check that "Click" was rendered
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "C");
        // Check style (blue, underlined)
        let style = buf.cell((0, 0)).unwrap().style();
        assert_eq!(style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_render_separator() {
        let elem = DocumentElement::separator();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 10, 1), &mut buf, &config);

        // All cells should be horizontal line
        for x in 0..10 {
            let symbol = buf.cell((x, 0)).unwrap().symbol();
            assert!(symbol == "─" || symbol == "-"); // Unicode or ASCII
        }
    }

    #[test]
    fn test_render_spacer() {
        let elem = DocumentElement::spacer(3);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));
        // Fill buffer with 'X' first
        for y in 0..5 {
            for x in 0..10 {
                buf.cell_mut((x, y)).unwrap().set_char('X');
            }
        }
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 10, 3), &mut buf, &config);

        // Spacer doesn't change buffer - cells should still be 'X'
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "X");
    }

    #[test]
    fn test_styled_text() {
        let style = Style::default().fg(Color::Red);
        let elem = DocumentElement::styled_text("Red text", style);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        let config = DisplayConfig::default();

        elem.render(Rect::new(0, 0, 20, 1), &mut buf, &config);

        assert_eq!(buf.cell((0, 0)).unwrap().style().fg, Some(Color::Red));
    }

    #[test]
    fn test_heading_level_clamping() {
        let elem = DocumentElement::heading(0, "Zero");
        match elem {
            DocumentElement::Heading { level, .. } => assert_eq!(level, 1),
            _ => panic!("Expected Heading"),
        }

        let elem = DocumentElement::heading(10, "Ten");
        match elem {
            DocumentElement::Heading { level, .. } => assert_eq!(level, 6),
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_styled_group() {
        let style = Style::default().bg(Color::Blue);
        let elem = DocumentElement::styled_group(
            vec![DocumentElement::text("Content")],
            style,
        );

        match elem {
            DocumentElement::Group { style: s, .. } => assert_eq!(s, Some(style)),
            _ => panic!("Expected Group"),
        }
    }

    #[test]
    fn test_document_element_debug() {
        let elem = DocumentElement::text("Hello");
        let debug_str = format!("{:?}", elem);
        assert!(debug_str.contains("Text"));
        assert!(debug_str.contains("Hello"));

        let elem = DocumentElement::separator();
        let debug_str = format!("{:?}", elem);
        assert!(debug_str.contains("Separator"));
    }
}
