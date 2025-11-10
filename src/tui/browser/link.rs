use super::target::Target;

/// Link represents a hyperlink-like element in browser content
#[derive(Debug, Clone)]
pub struct Link {
    /// The display text for the link
    pub display: String,
    /// Where the link points to
    pub target: Target,
    /// Start position of the link in the line (character index, not byte index)
    pub start: usize,
    /// End position of the link in the line (exclusive, character index)
    pub end: usize,
}

impl Link {
    /// Create a new link
    pub fn new(display: impl Into<String>, target: Target, start: usize, end: usize) -> Self {
        Self {
            display: display.into(),
            target,
            start,
            end,
        }
    }

    /// Get the length of the link in characters
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the link is empty (should not normally happen)
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this link overlaps with another link
    pub fn overlaps_with(&self, other: &Link) -> bool {
        !(self.end <= other.start || other.end <= self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let target = Target::Player { id: 8480018 };
        let link = Link::new("Nick Suzuki", target.clone(), 0, 11);

        assert_eq!(link.display, "Nick Suzuki");
        assert_eq!(link.target, target);
        assert_eq!(link.start, 0);
        assert_eq!(link.end, 11);
    }

    #[test]
    fn test_link_len() {
        let target = Target::Team { id: "MTL".to_string() };
        let link = Link::new("Canadiens", target, 10, 19);

        assert_eq!(link.len(), 9);
    }

    #[test]
    fn test_link_is_empty() {
        let target = Target::Team { id: "MTL".to_string() };
        let valid_link = Link::new("Canadiens", target.clone(), 10, 19);
        let empty_link = Link::new("", target.clone(), 5, 5);
        let invalid_link = Link::new("", target, 10, 5);

        assert!(!valid_link.is_empty());
        assert!(empty_link.is_empty());
        assert!(invalid_link.is_empty());
    }

    #[test]
    fn test_link_no_overlap() {
        let target = Target::Player { id: 8480018 };
        let link1 = Link::new("Link1", target.clone(), 0, 5);
        let link2 = Link::new("Link2", target, 10, 15);

        assert!(!link1.overlaps_with(&link2));
        assert!(!link2.overlaps_with(&link1));
    }

    #[test]
    fn test_link_overlap() {
        let target = Target::Player { id: 8480018 };
        let link1 = Link::new("Link1", target.clone(), 0, 10);
        let link2 = Link::new("Link2", target, 5, 15);

        assert!(link1.overlaps_with(&link2));
        assert!(link2.overlaps_with(&link1));
    }

    #[test]
    fn test_link_adjacent_no_overlap() {
        let target = Target::Player { id: 8480018 };
        let link1 = Link::new("Link1", target.clone(), 0, 5);
        let link2 = Link::new("Link2", target, 5, 10);

        assert!(!link1.overlaps_with(&link2));
        assert!(!link2.overlaps_with(&link1));
    }

    #[test]
    fn test_link_contained() {
        let target = Target::Player { id: 8480018 };
        let outer = Link::new("Outer", target.clone(), 0, 20);
        let inner = Link::new("Inner", target, 5, 10);

        assert!(outer.overlaps_with(&inner));
        assert!(inner.overlaps_with(&outer));
    }

    #[test]
    fn test_link_debug() {
        let target = Target::Team { id: "MTL".to_string() };
        let link = Link::new("Canadiens", target, 10, 19);
        let debug_str = format!("{:?}", link);

        assert!(debug_str.contains("Canadiens"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("19"));
    }

    #[test]
    fn test_link_clone() {
        let target = Target::Team { id: "MTL".to_string() };
        let link = Link::new("Canadiens", target, 10, 19);
        let cloned = link.clone();

        assert_eq!(link.display, cloned.display);
        assert_eq!(link.target, cloned.target);
        assert_eq!(link.start, cloned.start);
        assert_eq!(link.end, cloned.end);
    }
}
