use super::link::Link;
use super::target::Target;

/// BrowserContent represents the content displayed in the browser tab
#[derive(Debug)]
pub struct BrowserContent {
    /// The raw text lines
    pub lines: Vec<String>,
    /// All links in the content
    pub links: Vec<Link>,
}

impl BrowserContent {
    /// Create a new builder for constructing browser content
    pub fn builder() -> BrowserContentBuilder {
        BrowserContentBuilder::new()
    }

    /// Get all links on a specific line
    pub fn links_on_line(&self, line_index: usize) -> Vec<&Link> {
        if line_index >= self.lines.len() {
            return vec![];
        }

        let mut char_count = 0;
        for i in 0..line_index {
            char_count += self.lines[i].chars().count() + 1; // +1 for newline
        }

        let line_start = char_count;
        let line_end = line_start + self.lines[line_index].chars().count();

        self.links
            .iter()
            .filter(|link| {
                link.start >= line_start && link.start < line_end
            })
            .collect()
    }
}

/// Builder for creating BrowserContent with embedded links
pub struct BrowserContentBuilder {
    current_line: String,
    current_position: usize,
    links: Vec<Link>,
    lines: Vec<String>,
}

impl BrowserContentBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            current_line: String::new(),
            current_position: 0,
            links: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Add plain text to the current line
    pub fn text(mut self, text: &str) -> Self {
        self.current_line.push_str(text);
        self.current_position += text.chars().count();
        self
    }

    /// Add a link to the current line
    pub fn link(mut self, display: &str, target: Target) -> Self {
        let start = self.current_position;
        let end = start + display.chars().count();

        self.links.push(Link::new(display, target, start, end));
        self.current_line.push_str(display);
        self.current_position = end;

        self
    }

    /// End the current line and start a new one
    pub fn newline(mut self) -> Self {
        self.lines.push(self.current_line.clone());
        self.current_line.clear();
        self.current_position += 1; // Account for newline character
        self
    }

    /// Build the final BrowserContent
    pub fn build(mut self) -> BrowserContent {
        // Add the last line if it's not empty
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line);
        }

        BrowserContent {
            lines: self.lines,
            links: self.links,
        }
    }
}

impl Default for BrowserContentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_text_only() {
        let content = BrowserContent::builder()
            .text("Hello, world!")
            .build();

        assert_eq!(content.lines.len(), 1);
        assert_eq!(content.lines[0], "Hello, world!");
        assert_eq!(content.links.len(), 0);
    }

    #[test]
    fn test_builder_with_single_link() {
        let target = Target::Player { id: 8480018 };
        let content = BrowserContent::builder()
            .link("Nick Suzuki", target.clone())
            .text(" plays hockey.")
            .build();

        assert_eq!(content.lines.len(), 1);
        assert_eq!(content.lines[0], "Nick Suzuki plays hockey.");
        assert_eq!(content.links.len(), 1);
        assert_eq!(content.links[0].display, "Nick Suzuki");
        assert_eq!(content.links[0].start, 0);
        assert_eq!(content.links[0].end, 11);
    }

    #[test]
    fn test_builder_multiple_links() {
        let player_target = Target::Player { id: 8480018 };
        let team_target = Target::Team { id: "MTL".to_string() };

        let content = BrowserContent::builder()
            .link("Nick Suzuki", player_target)
            .text(" plays for the ")
            .link("Canadiens", team_target)
            .text(".")
            .build();

        assert_eq!(content.lines.len(), 1);
        assert_eq!(content.lines[0], "Nick Suzuki plays for the Canadiens.");
        assert_eq!(content.links.len(), 2);

        assert_eq!(content.links[0].display, "Nick Suzuki");
        assert_eq!(content.links[0].start, 0);
        assert_eq!(content.links[0].end, 11);

        assert_eq!(content.links[1].display, "Canadiens");
        assert_eq!(content.links[1].start, 26);
        assert_eq!(content.links[1].end, 35);
    }

    #[test]
    fn test_builder_multiline() {
        let target = Target::Player { id: 8480018 };

        let content = BrowserContent::builder()
            .text("Line 1: ")
            .link("Link", target.clone())
            .newline()
            .text("Line 2: ")
            .link("Another", target)
            .build();

        assert_eq!(content.lines.len(), 2);
        assert_eq!(content.lines[0], "Line 1: Link");
        assert_eq!(content.lines[1], "Line 2: Another");
        assert_eq!(content.links.len(), 2);
    }

    #[test]
    fn test_builder_empty() {
        let content = BrowserContent::builder().build();

        assert_eq!(content.lines.len(), 0);
        assert_eq!(content.links.len(), 0);
    }

    #[test]
    fn test_builder_link_positions_with_newlines() {
        let target = Target::Player { id: 8480018 };

        let content = BrowserContent::builder()
            .text("First ")
            .link("link", target.clone())
            .newline()
            .text("Second ")
            .link("link", target)
            .build();

        assert_eq!(content.links.len(), 2);
        // First link on line 0
        assert_eq!(content.links[0].start, 6);
        assert_eq!(content.links[0].end, 10);
        // Second link on line 1 (position after first line + newline)
        assert_eq!(content.links[1].start, 18); // "First link\n" = 11 chars, then "Second " = 7 chars
        assert_eq!(content.links[1].end, 22);
    }

    #[test]
    fn test_builder_only_links() {
        let target1 = Target::Team { id: "MTL".to_string() };
        let target2 = Target::Team { id: "TOR".to_string() };

        let content = BrowserContent::builder()
            .link("Link1", target1)
            .link("Link2", target2)
            .build();

        assert_eq!(content.lines.len(), 1);
        assert_eq!(content.lines[0], "Link1Link2");
        assert_eq!(content.links.len(), 2);
        assert_eq!(content.links[0].start, 0);
        assert_eq!(content.links[0].end, 5);
        assert_eq!(content.links[1].start, 5);
        assert_eq!(content.links[1].end, 10);
    }

    #[test]
    fn test_links_on_line_single_line() {
        let player_target = Target::Player { id: 8480018 };
        let team_target = Target::Team { id: "MTL".to_string() };

        let content = BrowserContent::builder()
            .link("Nick Suzuki", player_target)
            .text(" plays for ")
            .link("Canadiens", team_target)
            .build();

        let links_line_0 = content.links_on_line(0);
        assert_eq!(links_line_0.len(), 2);
        assert_eq!(links_line_0[0].display, "Nick Suzuki");
        assert_eq!(links_line_0[1].display, "Canadiens");

        let links_line_1 = content.links_on_line(1);
        assert_eq!(links_line_1.len(), 0);
    }

    #[test]
    fn test_links_on_line_multiline() {
        let target = Target::Player { id: 8480018 };

        let content = BrowserContent::builder()
            .text("Line 1: ")
            .link("Link1", target.clone())
            .newline()
            .text("Line 2: ")
            .link("Link2", target)
            .build();

        let links_line_0 = content.links_on_line(0);
        assert_eq!(links_line_0.len(), 1);
        assert_eq!(links_line_0[0].display, "Link1");

        let links_line_1 = content.links_on_line(1);
        assert_eq!(links_line_1.len(), 1);
        assert_eq!(links_line_1[0].display, "Link2");
    }

    #[test]
    fn test_links_on_line_no_links() {
        let content = BrowserContent::builder()
            .text("No links here")
            .build();

        let links_line_0 = content.links_on_line(0);
        assert_eq!(links_line_0.len(), 0);
    }

    #[test]
    fn test_links_on_line_out_of_bounds() {
        let target = Target::Player { id: 8480018 };
        let content = BrowserContent::builder()
            .link("Link", target)
            .build();

        let links = content.links_on_line(10);
        assert_eq!(links.len(), 0);
    }
}
