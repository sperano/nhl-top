//! # Navigation Framework
//!
//! This module provides a generic navigation system for drill-down views.
//! It supports:
//! - Stack-based navigation (like browser history)
//! - Breadcrumb trails
//! - Type-safe panel definitions
//! - Data caching per panel
//!
//! ## Usage Example
//!
//! ```ignore
//! // Define your panel types
//! #[derive(Clone, Debug, PartialEq)]
//! enum MyPanel {
//!     TeamDetail { team_id: i64, team_name: String },
//!     PlayerDetail { player_id: i64, player_name: String },
//! }
//!
//! impl Panel for MyPanel {
//!     fn breadcrumb_label(&self) -> String {
//!         match self {
//!             MyPanel::TeamDetail { team_name, .. } => team_name.clone(),
//!             MyPanel::PlayerDetail { player_name, .. } => player_name.clone(),
//!         }
//!     }
//! }
//!
//! // Use the navigation stack
//! let mut nav = NavigationStack::new();
//! nav.push(MyPanel::TeamDetail { team_id: 1, team_name: "Canadiens".into() });
//! nav.push(MyPanel::PlayerDetail { player_id: 42, player_name: "Anderson".into() });
//!
//! // Get breadcrumb: "Canadiens >> Anderson"
//! let breadcrumb = nav.breadcrumb_string(" >> ");
//! ```

use std::collections::HashMap;
use std::hash::Hash;

/// Trait for panel types that can be navigated to
pub trait Panel: Clone + PartialEq {
    /// Get the label to display in breadcrumb trail
    fn breadcrumb_label(&self) -> String;

    /// Optional: Get a unique key for data caching
    /// Default implementation uses the breadcrumb label
    fn cache_key(&self) -> String {
        self.breadcrumb_label()
    }
}

/// A stack-based navigation system with breadcrumbs
#[derive(Clone)]
pub struct NavigationStack<P: Panel> {
    stack: Vec<P>,
}

impl<P: Panel> NavigationStack<P> {
    /// Create a new empty navigation stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push a new panel onto the stack
    pub fn push(&mut self, panel: P) {
        self.stack.push(panel);
    }

    /// Pop the current panel and return to the previous one
    /// Returns the popped panel if successful, None if stack is empty
    pub fn pop(&mut self) -> Option<P> {
        self.stack.pop()
    }

    /// Get the current (top) panel
    pub fn current(&self) -> Option<&P> {
        self.stack.last()
    }

    /// Get the depth of the navigation stack
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Clear the entire stack
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Get breadcrumb trail as a vector of labels
    pub fn breadcrumb_trail(&self) -> Vec<String> {
        self.stack.iter().map(|p| p.breadcrumb_label()).collect()
    }

    /// Get breadcrumb trail as a single string with separator
    pub fn breadcrumb_string(&self, separator: &str) -> String {
        self.breadcrumb_trail().join(separator)
    }

    /// Get a reference to the entire stack
    pub fn stack(&self) -> &[P] {
        &self.stack
    }

    /// Replace the entire stack
    pub fn replace_stack(&mut self, new_stack: Vec<P>) {
        self.stack = new_stack;
    }

    /// Go back to a specific depth in the stack
    /// Returns true if successful, false if depth is invalid
    pub fn go_to_depth(&mut self, depth: usize) -> bool {
        if depth == 0 || depth > self.stack.len() {
            return false;
        }
        self.stack.truncate(depth);
        true
    }
}

impl<P: Panel> Default for NavigationStack<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Data cache associated with navigation panels
pub struct NavigationDataCache<K, V>
where
    K: Eq + Hash,
{
    cache: HashMap<K, V>,
}

impl<K, V> NavigationDataCache<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Store data for a given key
    pub fn insert(&mut self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// Get data for a given key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    /// Remove data for a given key
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key)
    }

    /// Check if data exists for a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached items
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl<K, V> Default for NavigationDataCache<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Combined navigation context with stack and data cache
pub struct NavigationContext<P, K, V>
where
    P: Panel,
    K: Eq + Hash,
{
    pub stack: NavigationStack<P>,
    pub data: NavigationDataCache<K, V>,
}

impl<P, K, V> NavigationContext<P, K, V>
where
    P: Panel,
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            stack: NavigationStack::new(),
            data: NavigationDataCache::new(),
        }
    }

    /// Navigate to a new panel (push)
    pub fn navigate_to(&mut self, panel: P) {
        self.stack.push(panel);
    }

    /// Go back (pop)
    pub fn go_back(&mut self) -> Option<P> {
        self.stack.pop()
    }

    /// Clear navigation and optionally clear data cache
    pub fn reset(&mut self, clear_cache: bool) {
        self.stack.clear();
        if clear_cache {
            self.data.clear();
        }
    }

    /// Check if we're at the root (no navigation stack)
    pub fn is_at_root(&self) -> bool {
        self.stack.is_empty()
    }
}

impl<P, K, V> Default for NavigationContext<P, K, V>
where
    P: Panel,
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestPanel {
        Team { id: i64, name: String },
        Player { id: i64, name: String },
    }

    impl Panel for TestPanel {
        fn breadcrumb_label(&self) -> String {
            match self {
                TestPanel::Team { name, .. } => name.clone(),
                TestPanel::Player { name, .. } => name.clone(),
            }
        }

        fn cache_key(&self) -> String {
            match self {
                TestPanel::Team { id, .. } => format!("team:{}", id),
                TestPanel::Player { id, .. } => format!("player:{}", id),
            }
        }
    }

    #[test]
    fn test_navigation_stack_push_pop() {
        let mut nav = NavigationStack::new();
        assert!(nav.is_empty());
        assert_eq!(nav.depth(), 0);

        nav.push(TestPanel::Team {
            id: 1,
            name: "Canadiens".into(),
        });
        assert_eq!(nav.depth(), 1);
        assert_eq!(
            nav.current().unwrap().breadcrumb_label(),
            "Canadiens"
        );

        nav.push(TestPanel::Player {
            id: 42,
            name: "Anderson".into(),
        });
        assert_eq!(nav.depth(), 2);
        assert_eq!(nav.current().unwrap().breadcrumb_label(), "Anderson");

        let popped = nav.pop();
        assert!(popped.is_some());
        assert_eq!(nav.depth(), 1);
        assert_eq!(
            nav.current().unwrap().breadcrumb_label(),
            "Canadiens"
        );

        // Pop the last item - should succeed and return to empty stack
        let popped = nav.pop();
        assert!(popped.is_some());
        assert_eq!(nav.depth(), 0);
        assert!(nav.is_empty());
    }

    #[test]
    fn test_breadcrumb_trail() {
        let mut nav = NavigationStack::new();
        nav.push(TestPanel::Team {
            id: 1,
            name: "Canadiens".into(),
        });
        nav.push(TestPanel::Player {
            id: 42,
            name: "Anderson".into(),
        });
        nav.push(TestPanel::Team {
            id: 2,
            name: "Columbus".into(),
        });

        let trail = nav.breadcrumb_trail();
        assert_eq!(trail, vec!["Canadiens", "Anderson", "Columbus"]);

        let breadcrumb = nav.breadcrumb_string(" >> ");
        assert_eq!(breadcrumb, "Canadiens >> Anderson >> Columbus");
    }

    #[test]
    fn test_go_to_depth() {
        let mut nav = NavigationStack::new();
        nav.push(TestPanel::Team {
            id: 1,
            name: "Canadiens".into(),
        });
        nav.push(TestPanel::Player {
            id: 42,
            name: "Anderson".into(),
        });
        nav.push(TestPanel::Team {
            id: 2,
            name: "Columbus".into(),
        });

        assert_eq!(nav.depth(), 3);

        assert!(nav.go_to_depth(2));
        assert_eq!(nav.depth(), 2);
        assert_eq!(nav.current().unwrap().breadcrumb_label(), "Anderson");

        assert!(!nav.go_to_depth(0));
        assert!(!nav.go_to_depth(5));
    }

    #[test]
    fn test_data_cache() {
        let mut cache: NavigationDataCache<String, Vec<String>> =
            NavigationDataCache::new();

        cache.insert("team:1".into(), vec!["Player1".into(), "Player2".into()]);
        cache.insert("player:42".into(), vec!["Stat1".into(), "Stat2".into()]);

        assert_eq!(cache.len(), 2);
        assert!(cache.contains_key(&"team:1".into()));

        let data = cache.get(&"team:1".into());
        assert!(data.is_some());
        assert_eq!(data.unwrap().len(), 2);

        cache.remove(&"team:1".into());
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains_key(&"team:1".into()));
    }

    #[test]
    fn test_navigation_context() {
        let mut ctx: NavigationContext<TestPanel, String, Vec<String>> =
            NavigationContext::new();

        assert!(ctx.is_at_root());

        ctx.navigate_to(TestPanel::Team {
            id: 1,
            name: "Canadiens".into(),
        });
        ctx.data.insert("team:1".into(), vec!["Player1".into()]);

        assert!(!ctx.is_at_root());
        assert_eq!(ctx.stack.depth(), 1);
        assert_eq!(ctx.data.len(), 1);

        ctx.reset(true);
        assert!(ctx.is_at_root());
        assert_eq!(ctx.data.len(), 0);
    }

    #[test]
    fn test_cache_key_override() {
        let panel = TestPanel::Team {
            id: 1,
            name: "Canadiens".into(),
        };

        assert_eq!(panel.cache_key(), "team:1");
        assert_eq!(panel.breadcrumb_label(), "Canadiens");
    }
}
