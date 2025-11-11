# NHL TUI Widget-Based Architecture Refactoring Plan

## Executive Summary

This document outlines the complete refactoring of the NHL TUI application from a state-based navigation system to a proper widget-based architecture with hierarchical focus management. The new architecture will be similar to modern UI frameworks like React or Flutter, where widgets form a tree structure and focus flows naturally through the hierarchy.

### Goals
1. Replace manual focus tracking with automatic focus management
2. Make all player and team names clickable links throughout the application
3. Enable deep navigation chains (Team → Player → Team → Player → ...)
4. Add breadcrumb navigation between TabBar and Content
5. Fix broken game details navigation
6. Create a maintainable, testable, and extensible widget system

### Timeline Estimate
- Phase 1 (Core Infrastructure): 2-3 days
- Phase 2 (Container Widgets): 2 days
- Phase 3 (Breadcrumbs): 1 day
- Phase 4 (Standings Navigation): 3-4 days
- Phase 5 (Game Details): 2 days
- Phase 6 (Testing): 2-3 days
- Phase 7 (Migration): 1-2 days
- **Total: 13-18 days**

---

## Current Architecture Analysis

### What Currently Exists

1. **RenderableWidget Trait** (`src/tui/widgets/mod.rs`)
   - Basic rendering interface for widgets
   - No focus management or input handling

2. **NavigationContext** (`src/tui/navigation/mod.rs`)
   - Generic navigation framework with stack-based navigation
   - Used in standings for Team → Player navigation
   - Complex but functional

3. **Manual Focus Tracking**
   - Each tab maintains its own focus state
   - Inconsistent implementation across tabs
   - Difficult to maintain and extend

4. **Broken Game Details Navigation**
   - Complex `PlayerSection` enum tracking
   - Unreliable focus state management
   - Poor user experience

### What's Missing

1. **Widget Hierarchy**: No parent-child relationships between widgets
2. **Focus System**: No automatic focus management or delegation
3. **Input Routing**: No systematic way to route input to focused widget
4. **Consistent Links**: Player/team names not consistently clickable
5. **Breadcrumbs**: No visual navigation history

---

## Phase 1: Core Widget Infrastructure

### Step 1.1: Define Core Focus Types

**File:** `src/tui/widgets/focus.rs`

```rust
use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use std::any::Any;

/// A unique identifier for widgets in the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub usize);

impl WidgetId {
    /// Generate a new unique widget ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        WidgetId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Result of handling an input event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    /// Input was handled, stop propagation
    Handled,
    /// Input was not handled, continue propagation
    NotHandled,
    /// Request focus to move in a direction
    MoveFocus(FocusDirection),
    /// Request navigation to a new panel/page
    Navigate(NavigationAction),
}

/// Focus movement direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Next,      // Tab or Down
    Previous,  // Shift+Tab or Up
    Left,      // Left arrow
    Right,     // Right arrow
    In,        // Enter - focus into container
    Out,       // Esc - focus out of container
}

/// Navigation action to perform
#[derive(Debug, Clone)]
pub enum NavigationAction {
    PushPanel(Box<dyn Any>),
    PopPanel,
    NavigateToTeam(String),
    NavigateToPlayer(i64),
    NavigateToGame(i64),
}

/// Trait for widgets that can receive focus
pub trait Focusable: RenderableWidget {
    /// Get the unique ID of this widget
    fn widget_id(&self) -> WidgetId;

    /// Returns true if this widget can receive focus
    fn can_focus(&self) -> bool;

    /// Returns true if this widget currently has focus
    fn is_focused(&self) -> bool;

    /// Set whether this widget has focus
    fn set_focused(&mut self, focused: bool);

    /// Handle keyboard input when focused
    fn handle_input(&mut self, event: KeyEvent) -> InputResult;

    /// Get focusable children for navigation
    fn focusable_children(&self) -> Vec<WidgetId> {
        vec![]
    }

    /// Find a child widget by ID
    fn find_child(&self, id: WidgetId) -> Option<&dyn Focusable> {
        None
    }

    /// Find a mutable child widget by ID
    fn find_child_mut(&mut self, id: WidgetId) -> Option<&mut dyn Focusable> {
        None
    }
}
```

**Implementation Tasks:**
1. Create the `focus.rs` file with these types
2. Add derive macros for common traits
3. Document each type and method thoroughly
4. Create builder patterns for common configurations

**Tests to Write:**
- `test_widget_id_uniqueness()` - Verify IDs are unique
- `test_focus_direction_mapping()` - Map key events to directions
- `test_input_result_propagation()` - Test result handling

---

### Step 1.2: Implement Widget Tree Manager

**File:** `src/tui/widgets/tree.rs`

```rust
use super::focus::*;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

/// Manages the widget tree and focus state
pub struct WidgetTree {
    /// Root widget of the tree
    root: Option<Box<dyn Focusable>>,
    /// Currently focused widget ID
    focused_id: Option<WidgetId>,
    /// Focus path from root to focused widget
    focus_path: Vec<WidgetId>,
    /// Cache of widget references for quick lookup
    widget_cache: HashMap<WidgetId, *const dyn Focusable>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            root: None,
            focused_id: None,
            focus_path: vec![],
            widget_cache: HashMap::new(),
        }
    }

    /// Set the root widget
    pub fn set_root(&mut self, root: Box<dyn Focusable>) {
        let root_id = root.widget_id();
        self.root = Some(root);
        self.rebuild_cache();

        // Focus first focusable widget
        if self.focused_id.is_none() {
            self.focus_first();
        }
    }

    /// Route input to the focused widget
    pub fn handle_input(&mut self, event: KeyEvent) -> bool {
        if let Some(focused_id) = self.focused_id {
            if let Some(widget) = self.find_widget_mut(focused_id) {
                match widget.handle_input(event) {
                    InputResult::Handled => return true,
                    InputResult::NotHandled => {
                        // Try default navigation
                        return self.handle_default_navigation(event);
                    }
                    InputResult::MoveFocus(direction) => {
                        return self.move_focus(direction);
                    }
                    InputResult::Navigate(action) => {
                        // Emit navigation event (handled by parent)
                        return self.emit_navigation(action);
                    }
                }
            }
        }
        false
    }

    /// Move focus in the specified direction
    pub fn move_focus(&mut self, direction: FocusDirection) -> bool {
        match direction {
            FocusDirection::Next => self.focus_next(),
            FocusDirection::Previous => self.focus_previous(),
            FocusDirection::In => self.focus_into(),
            FocusDirection::Out => self.focus_out(),
            FocusDirection::Left => self.focus_left(),
            FocusDirection::Right => self.focus_right(),
        }
    }

    /// Focus the next focusable widget
    fn focus_next(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if focusable_widgets.is_empty() {
            return false;
        }

        let current_index = self.focused_id
            .and_then(|id| focusable_widgets.iter().position(|&w| w == id))
            .unwrap_or(0);

        let next_index = (current_index + 1) % focusable_widgets.len();
        self.set_focus(focusable_widgets[next_index]);
        true
    }

    /// Focus the previous focusable widget
    fn focus_previous(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if focusable_widgets.is_empty() {
            return false;
        }

        let current_index = self.focused_id
            .and_then(|id| focusable_widgets.iter().position(|&w| w == id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            focusable_widgets.len() - 1
        } else {
            current_index - 1
        };
        self.set_focus(focusable_widgets[prev_index]);
        true
    }

    /// Set focus to a specific widget
    fn set_focus(&mut self, widget_id: WidgetId) {
        // Clear old focus
        if let Some(old_id) = self.focused_id {
            if let Some(widget) = self.find_widget_mut(old_id) {
                widget.set_focused(false);
            }
        }

        // Set new focus
        if let Some(widget) = self.find_widget_mut(widget_id) {
            widget.set_focused(true);
            self.focused_id = Some(widget_id);
            self.update_focus_path(widget_id);
        }
    }

    /// Collect all focusable widgets in tree order
    fn collect_focusable_widgets(&self) -> Vec<WidgetId> {
        let mut widgets = vec![];
        if let Some(root) = &self.root {
            self.collect_focusable_recursive(root.as_ref(), &mut widgets);
        }
        widgets
    }

    fn collect_focusable_recursive(&self, widget: &dyn Focusable, widgets: &mut Vec<WidgetId>) {
        if widget.can_focus() {
            widgets.push(widget.widget_id());
        }
        for child_id in widget.focusable_children() {
            if let Some(child) = widget.find_child(child_id) {
                self.collect_focusable_recursive(child, widgets);
            }
        }
    }

    // ... Additional helper methods
}

/// Default keyboard navigation mappings
impl WidgetTree {
    fn handle_default_navigation(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Tab => self.move_focus(FocusDirection::Next),
            KeyCode::BackTab => self.move_focus(FocusDirection::Previous),
            KeyCode::Down => self.move_focus(FocusDirection::Next),
            KeyCode::Up => self.move_focus(FocusDirection::Previous),
            KeyCode::Left => self.move_focus(FocusDirection::Left),
            KeyCode::Right => self.move_focus(FocusDirection::Right),
            KeyCode::Enter => self.move_focus(FocusDirection::In),
            KeyCode::Esc => self.move_focus(FocusDirection::Out),
            _ => false,
        }
    }
}
```

**Implementation Tasks:**
1. Implement focus traversal algorithms (depth-first for Tab navigation)
2. Add focus path tracking for breadcrumb support
3. Implement widget cache for performance
4. Add debug visualization of widget tree

**Tests to Write:**
- `test_focus_next_wraps_around()` - Tab wraps to first widget
- `test_focus_previous_wraps_to_end()` - Shift+Tab wraps to last
- `test_focus_path_tracking()` - Path updates correctly
- `test_empty_tree_handling()` - Graceful handling of empty tree

---

### Step 1.3: Create Link Widget

**File:** `src/tui/widgets/link.rs`

```rust
use super::focus::*;
use crate::formatting::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

/// A clickable link widget
pub struct Link {
    id: WidgetId,
    text: String,
    focused: bool,
    /// Callback when link is activated
    on_activate: Option<Box<dyn FnMut() -> NavigationAction>>,
    /// Optional styling
    style: LinkStyle,
}

#[derive(Debug, Clone)]
pub struct LinkStyle {
    pub normal: Style,
    pub focused: Style,
    pub active: Style,
}

impl Default for LinkStyle {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            focused: Style::default().fg(Color::Yellow),
            active: Style::default().fg(Color::Green),
        }
    }
}

impl Link {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: WidgetId::new(),
            text: text.into(),
            focused: false,
            on_activate: None,
            style: LinkStyle::default(),
        }
    }

    pub fn with_action<F>(mut self, action: F) -> Self
    where
        F: FnMut() -> NavigationAction + 'static,
    {
        self.on_activate = Some(Box::new(action));
        self
    }

    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.style = style;
        self
    }

    /// Create a player link
    pub fn player(name: impl Into<String>, player_id: i64) -> Self {
        Self::new(name).with_action(move || NavigationAction::NavigateToPlayer(player_id))
    }

    /// Create a team link
    pub fn team(name: impl Into<String>, team_abbrev: impl Into<String>) -> Self {
        let abbrev = team_abbrev.into();
        Self::new(name).with_action(move || NavigationAction::NavigateToTeam(abbrev.clone()))
    }
}

impl Focusable for Link {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused {
            return InputResult::NotHandled;
        }

        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(ref mut action) = self.on_activate {
                    let navigation = action();
                    InputResult::Navigate(navigation)
                } else {
                    InputResult::Handled
                }
            }
            _ => InputResult::NotHandled,
        }
    }
}

impl RenderableWidget for Link {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let style = if self.focused {
            self.style.focused.patch(Style::default().fg(config.colors.selection_fg))
        } else {
            self.style.normal
        };

        // Render text with underline if focused
        let text = if self.focused {
            format!("▶ {}", self.text)
        } else {
            self.text.clone()
        };

        buf.set_string(area.x, area.y, &text, style);
    }

    fn height_hint(&self, _width: u16) -> u16 {
        1
    }
}

/// Builder for creating multiple links
pub struct LinkBuilder {
    style: LinkStyle,
}

impl LinkBuilder {
    pub fn new() -> Self {
        Self {
            style: LinkStyle::default(),
        }
    }

    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.style = style;
        self
    }

    pub fn player(&self, name: impl Into<String>, player_id: i64) -> Link {
        Link::player(name, player_id).with_style(self.style.clone())
    }

    pub fn team(&self, name: impl Into<String>, team_abbrev: impl Into<String>) -> Link {
        Link::team(name, team_abbrev).with_style(self.style.clone())
    }
}
```

**Implementation Tasks:**
1. Add hover state for mouse support (future)
2. Add keyboard shortcuts (e.g., underlined letters)
3. Support for disabled state
4. Add icon support (▶, →, etc.)

**Tests to Write:**
- `test_link_renders_normal()` - Normal rendering
- `test_link_renders_focused()` - Focused rendering with indicator
- `test_link_activates_on_enter()` - Enter key activates
- `test_link_activates_on_space()` - Space key activates
- `test_link_builder_pattern()` - Builder creates correct links

---

## Phase 2: Container Widgets

### Step 2.1: List Widget (Vertical Container)

**File:** `src/tui/widgets/list.rs`

```rust
use super::focus::*;
use crate::formatting::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

/// A vertical list of focusable widgets
pub struct List {
    id: WidgetId,
    items: Vec<Box<dyn Focusable>>,
    selected_index: usize,
    focused: bool,
    /// Visual style
    style: ListStyle,
    /// Scroll state
    scroll_offset: usize,
    visible_items: usize,
}

#[derive(Debug, Clone)]
pub struct ListStyle {
    pub border: bool,
    pub highlight_symbol: String,
    pub spacing: u16,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            border: false,
            highlight_symbol: "▶ ".to_string(),
            spacing: 0,
        }
    }
}

impl List {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            items: vec![],
            selected_index: 0,
            focused: false,
            style: ListStyle::default(),
            scroll_offset: 0,
            visible_items: 10,
        }
    }

    pub fn with_items(mut self, items: Vec<Box<dyn Focusable>>) -> Self {
        self.items = items;
        self
    }

    pub fn add_item(&mut self, item: Box<dyn Focusable>) {
        self.items.push(item);
    }

    pub fn with_style(mut self, style: ListStyle) -> Self {
        self.style = style;
        self
    }

    /// Ensure selected item is visible
    fn ensure_visible(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_items {
            self.scroll_offset = self.selected_index.saturating_sub(self.visible_items - 1);
        }
    }

    fn select_next(&mut self) -> bool {
        if self.selected_index + 1 < self.items.len() {
            self.items[self.selected_index].set_focused(false);
            self.selected_index += 1;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    fn select_previous(&mut self) -> bool {
        if self.selected_index > 0 {
            self.items[self.selected_index].set_focused(false);
            self.selected_index -= 1;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }
}

impl Focusable for List {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        !self.items.is_empty()
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;

        // Update child focus
        if focused && !self.items.is_empty() {
            self.items[self.selected_index].set_focused(true);
        } else {
            for item in &mut self.items {
                item.set_focused(false);
            }
        }
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused || self.items.is_empty() {
            return InputResult::NotHandled;
        }

        // First try to delegate to selected item
        let result = self.items[self.selected_index].handle_input(event);
        if result != InputResult::NotHandled {
            return result;
        }

        // Handle list navigation
        match event.code {
            KeyCode::Down => {
                if self.select_next() {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Next)
                }
            }
            KeyCode::Up => {
                if self.select_previous() {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Previous)
                }
            }
            KeyCode::Home => {
                if self.selected_index > 0 {
                    self.items[self.selected_index].set_focused(false);
                    self.selected_index = 0;
                    self.items[self.selected_index].set_focused(true);
                    self.ensure_visible();
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            KeyCode::End => {
                if self.selected_index < self.items.len() - 1 {
                    self.items[self.selected_index].set_focused(false);
                    self.selected_index = self.items.len() - 1;
                    self.items[self.selected_index].set_focused(true);
                    self.ensure_visible();
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            _ => InputResult::NotHandled,
        }
    }

    fn focusable_children(&self) -> Vec<WidgetId> {
        self.items.iter().map(|item| item.widget_id()).collect()
    }

    fn find_child(&self, id: WidgetId) -> Option<&dyn Focusable> {
        self.items.iter()
            .find(|item| item.widget_id() == id)
            .map(|item| item.as_ref())
    }

    fn find_child_mut(&mut self, id: WidgetId) -> Option<&mut dyn Focusable> {
        self.items.iter_mut()
            .find(|item| item.widget_id() == id)
            .map(|item| item.as_mut())
    }
}

impl RenderableWidget for List {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Calculate visible range
        let visible_height = area.height as usize;
        self.visible_items = visible_height;

        let end_index = (self.scroll_offset + visible_height).min(self.items.len());

        // Render visible items
        let mut y = area.y;
        for (i, item) in self.items[self.scroll_offset..end_index].iter().enumerate() {
            let item_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            item.render(item_area, buf, config);

            y += 1 + self.style.spacing;
            if y >= area.y + area.height {
                break;
            }
        }

        // Render scroll indicators if needed
        if self.scroll_offset > 0 {
            buf.set_string(area.right() - 1, area.y, "▲", Style::default());
        }
        if end_index < self.items.len() {
            buf.set_string(area.right() - 1, area.bottom() - 1, "▼", Style::default());
        }
    }

    fn height_hint(&self, _width: u16) -> u16 {
        let item_height = 1 + self.style.spacing;
        (self.items.len() as u16) * item_height
    }
}
```

**Implementation Tasks:**
1. Add smooth scrolling animation (optional)
2. Support for separators between items
3. Multi-column list support
4. Lazy loading for large lists

**Tests to Write:**
- `test_list_navigation_up_down()` - Basic navigation
- `test_list_scrolling()` - Scroll when navigating beyond visible
- `test_list_home_end()` - Jump to first/last
- `test_list_empty_handling()` - Graceful empty list
- `test_list_focus_delegation()` - Focus passes to children

---

### Step 2.2: Table Widget (2D Container)

**File:** `src/tui/widgets/table_focusable.rs`

```rust
use super::focus::*;
use super::list::List;
use crate::formatting::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

/// A table where cells can contain focusable widgets
pub struct FocusableTable {
    id: WidgetId,
    /// Headers for columns
    headers: Vec<String>,
    /// Rows of cells (Some = focusable widget, None = text)
    rows: Vec<TableRow>,
    /// Currently selected cell position
    selected_row: usize,
    selected_col: usize,
    focused: bool,
    /// Visual style
    style: TableStyle,
}

pub struct TableRow {
    cells: Vec<TableCell>,
}

pub enum TableCell {
    Text(String),
    Widget(Box<dyn Focusable>),
}

#[derive(Debug, Clone)]
pub struct TableStyle {
    pub show_headers: bool,
    pub borders: bool,
    pub row_spacing: u16,
    pub column_spacing: u16,
    pub highlight_row: bool,
    pub highlight_cell: bool,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            show_headers: true,
            borders: true,
            row_spacing: 0,
            column_spacing: 2,
            highlight_row: false,
            highlight_cell: true,
        }
    }
}

impl FocusableTable {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            headers: vec![],
            rows: vec![],
            selected_row: 0,
            selected_col: 0,
            focused: false,
            style: TableStyle::default(),
        }
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn add_row(&mut self, cells: Vec<TableCell>) {
        self.rows.push(TableRow { cells });
    }

    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Find next focusable cell in direction
    fn find_next_focusable(&self, row: usize, col: usize, direction: FocusDirection) -> Option<(usize, usize)> {
        match direction {
            FocusDirection::Right => {
                // Search right in current row
                for c in (col + 1)..self.rows.get(row)?.cells.len() {
                    if matches!(self.rows[row].cells[c], TableCell::Widget(_)) {
                        return Some((row, c));
                    }
                }
                // Wrap to next row
                for r in (row + 1)..self.rows.len() {
                    for c in 0..self.rows[r].cells.len() {
                        if matches!(self.rows[r].cells[c], TableCell::Widget(_)) {
                            return Some((r, c));
                        }
                    }
                }
                None
            }
            FocusDirection::Left => {
                // Search left in current row
                for c in (0..col).rev() {
                    if matches!(self.rows[row].cells[c], TableCell::Widget(_)) {
                        return Some((row, c));
                    }
                }
                // Wrap to previous row
                for r in (0..row).rev() {
                    for c in (0..self.rows[r].cells.len()).rev() {
                        if matches!(self.rows[r].cells[c], TableCell::Widget(_)) {
                            return Some((r, c));
                        }
                    }
                }
                None
            }
            FocusDirection::Next => {
                // Search down in current column
                for r in (row + 1)..self.rows.len() {
                    if col < self.rows[r].cells.len() {
                        if matches!(self.rows[r].cells[col], TableCell::Widget(_)) {
                            return Some((r, col));
                        }
                    }
                }
                None
            }
            FocusDirection::Previous => {
                // Search up in current column
                for r in (0..row).rev() {
                    if col < self.rows[r].cells.len() {
                        if matches!(self.rows[r].cells[col], TableCell::Widget(_)) {
                            return Some((r, col));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn move_selection(&mut self, direction: FocusDirection) -> bool {
        if let Some((new_row, new_col)) = self.find_next_focusable(self.selected_row, self.selected_col, direction) {
            // Clear old focus
            if let Some(TableCell::Widget(ref mut widget)) = self.rows.get_mut(self.selected_row)
                .and_then(|r| r.cells.get_mut(self.selected_col)) {
                widget.set_focused(false);
            }

            // Set new focus
            self.selected_row = new_row;
            self.selected_col = new_col;

            if let Some(TableCell::Widget(ref mut widget)) = self.rows.get_mut(new_row)
                .and_then(|r| r.cells.get_mut(new_col)) {
                widget.set_focused(true);
            }

            true
        } else {
            false
        }
    }
}

impl Focusable for FocusableTable {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        // Check if any cell is focusable
        self.rows.iter().any(|row| {
            row.cells.iter().any(|cell| matches!(cell, TableCell::Widget(_)))
        })
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;

        // Update cell focus
        if let Some(TableCell::Widget(ref mut widget)) = self.rows.get_mut(self.selected_row)
            .and_then(|r| r.cells.get_mut(self.selected_col)) {
            widget.set_focused(focused);
        }
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused {
            return InputResult::NotHandled;
        }

        // First try to delegate to selected cell
        if let Some(TableCell::Widget(ref mut widget)) = self.rows.get_mut(self.selected_row)
            .and_then(|r| r.cells.get_mut(self.selected_col)) {
            let result = widget.handle_input(event);
            if result != InputResult::NotHandled {
                return result;
            }
        }

        // Handle table navigation
        match event.code {
            KeyCode::Right => {
                if self.move_selection(FocusDirection::Right) {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Right)
                }
            }
            KeyCode::Left => {
                if self.move_selection(FocusDirection::Left) {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Left)
                }
            }
            KeyCode::Down => {
                if self.move_selection(FocusDirection::Next) {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Next)
                }
            }
            KeyCode::Up => {
                if self.move_selection(FocusDirection::Previous) {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Previous)
                }
            }
            _ => InputResult::NotHandled,
        }
    }

    // ... find_child methods similar to List
}

impl RenderableWidget for FocusableTable {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Calculate column widths
        let num_cols = self.headers.len().max(
            self.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0)
        );
        let col_width = area.width / num_cols as u16;

        let mut y = area.y;

        // Render headers
        if self.style.show_headers && !self.headers.is_empty() {
            let mut x = area.x;
            for header in &self.headers {
                buf.set_string(x, y, header, Style::default().add_modifier(Modifier::BOLD));
                x += col_width;
            }
            y += 1;

            // Header separator
            if self.style.borders {
                for x in area.x..area.right() {
                    buf.set_string(x, y, "─", Style::default());
                }
                y += 1;
            }
        }

        // Render rows
        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut x = area.x;

            for (col_idx, cell) in row.cells.iter().enumerate() {
                let cell_area = Rect {
                    x,
                    y,
                    width: col_width.saturating_sub(self.style.column_spacing),
                    height: 1,
                };

                match cell {
                    TableCell::Text(text) => {
                        let style = if self.style.highlight_row && row_idx == self.selected_row {
                            Style::default().bg(Color::DarkGray)
                        } else {
                            Style::default()
                        };
                        buf.set_string(x, y, text, style);
                    }
                    TableCell::Widget(widget) => {
                        widget.render(cell_area, buf, config);
                    }
                }

                x += col_width;
            }

            y += 1 + self.style.row_spacing;
            if y >= area.bottom() {
                break;
            }
        }
    }

    fn height_hint(&self, _width: u16) -> u16 {
        let header_height = if self.style.show_headers { 2 } else { 0 };
        let row_height = 1 + self.style.row_spacing;
        header_height + (self.rows.len() as u16 * row_height)
    }
}

/// Builder for creating tables with mixed content
pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<TableCell>>,
    style: TableStyle,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            headers: vec![],
            rows: vec![],
            style: TableStyle::default(),
        }
    }

    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn add_row(mut self, cells: Vec<TableCell>) -> Self {
        self.rows.push(cells);
        self
    }

    pub fn add_text_row(mut self, texts: Vec<String>) -> Self {
        let cells = texts.into_iter().map(TableCell::Text).collect();
        self.rows.push(cells);
        self
    }

    pub fn build(self) -> FocusableTable {
        let mut table = FocusableTable::new()
            .with_headers(self.headers)
            .with_style(self.style);

        for row in self.rows {
            table.add_row(row);
        }

        table
    }
}
```

**Implementation Tasks:**
1. Column width calculation strategies
2. Cell alignment options
3. Sort indicators in headers
4. Row selection vs cell selection modes
5. Virtual scrolling for large tables

**Tests to Write:**
- `test_table_2d_navigation()` - Navigate in all directions
- `test_table_skip_non_focusable()` - Skip text cells
- `test_table_wrap_navigation()` - Right wraps to next row
- `test_table_mixed_content()` - Mix of text and widgets
- `test_table_builder_pattern()` - Builder creates correct table

---

## Phase 3: Breadcrumb Integration

### Step 3.1: Create Breadcrumb Widget with Focus

**File:** `src/tui/widgets/breadcrumb_focusable.rs`

```rust
use super::focus::*;
use super::link::Link;
use crate::formatting::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

/// A breadcrumb navigation widget with clickable items
pub struct BreadcrumbWidget {
    id: WidgetId,
    items: Vec<BreadcrumbItem>,
    selected_index: Option<usize>,
    focused: bool,
    separator: String,
}

pub struct BreadcrumbItem {
    pub label: String,
    pub action: Option<Box<dyn FnMut() -> NavigationAction>>,
    pub is_current: bool,
}

impl BreadcrumbWidget {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            items: vec![],
            selected_index: None,
            focused: false,
            separator: " › ".to_string(),
        }
    }

    pub fn with_items(mut self, items: Vec<BreadcrumbItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_separator(mut self, separator: String) -> Self {
        self.separator = separator;
        self
    }

    /// Add an item to the breadcrumb trail
    pub fn push(&mut self, label: String, action: Option<Box<dyn FnMut() -> NavigationAction>>) {
        // Mark previous items as not current
        for item in &mut self.items {
            item.is_current = false;
        }

        self.items.push(BreadcrumbItem {
            label,
            action,
            is_current: true,
        });
    }

    /// Remove items after the specified index
    pub fn truncate(&mut self, index: usize) {
        self.items.truncate(index + 1);
        if let Some(last) = self.items.last_mut() {
            last.is_current = true;
        }
    }
}

impl Focusable for BreadcrumbWidget {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        self.items.iter().any(|item| item.action.is_some())
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if focused && self.selected_index.is_none() {
            // Find first clickable item
            self.selected_index = self.items.iter()
                .position(|item| item.action.is_some());
        } else if !focused {
            self.selected_index = None;
        }
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused || self.selected_index.is_none() {
            return InputResult::NotHandled;
        }

        let current_index = self.selected_index.unwrap();

        match event.code {
            KeyCode::Enter => {
                if let Some(ref mut action) = self.items[current_index].action {
                    let nav_action = action();
                    InputResult::Navigate(nav_action)
                } else {
                    InputResult::Handled
                }
            }
            KeyCode::Right => {
                // Find next clickable item
                for i in (current_index + 1)..self.items.len() {
                    if self.items[i].action.is_some() {
                        self.selected_index = Some(i);
                        return InputResult::Handled;
                    }
                }
                InputResult::MoveFocus(FocusDirection::Next)
            }
            KeyCode::Left => {
                // Find previous clickable item
                for i in (0..current_index).rev() {
                    if self.items[i].action.is_some() {
                        self.selected_index = Some(i);
                        return InputResult::Handled;
                    }
                }
                InputResult::MoveFocus(FocusDirection::Previous)
            }
            _ => InputResult::NotHandled,
        }
    }
}

impl RenderableWidget for BreadcrumbWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut x = area.x;
        let y = area.y;

        for (i, item) in self.items.iter().enumerate() {
            // Render item
            let style = if item.is_current {
                Style::default().add_modifier(Modifier::BOLD)
            } else if self.focused && self.selected_index == Some(i) {
                Style::default()
                    .fg(config.colors.selection_fg)
                    .add_modifier(Modifier::UNDERLINED)
            } else if item.action.is_some() {
                Style::default().fg(config.colors.link_fg)
            } else {
                Style::default()
            };

            let text = if self.focused && self.selected_index == Some(i) {
                format!("[{}]", item.label)
            } else {
                item.label.clone()
            };

            buf.set_string(x, y, &text, style);
            x += text.len() as u16;

            // Render separator
            if i < self.items.len() - 1 {
                buf.set_string(x, y, &self.separator, Style::default());
                x += self.separator.len() as u16;
            }

            if x >= area.right() {
                break;
            }
        }
    }

    fn height_hint(&self, _width: u16) -> u16 {
        1
    }
}
```

**Implementation Tasks:**
1. Ellipsis handling for long breadcrumbs
2. Dropdown menu for overflow items
3. Animation when navigation changes
4. Home icon for root

**Tests to Write:**
- `test_breadcrumb_navigation()` - Left/Right navigation
- `test_breadcrumb_activation()` - Enter activates item
- `test_breadcrumb_truncation()` - Truncate removes items
- `test_breadcrumb_rendering()` - Visual rendering test

---

## Phase 4: Standings Deep Navigation

### Step 4.1: Create Standings Widget Tree

**File:** `src/tui/standings/widgets.rs`

```rust
use crate::tui::widgets::{
    focus::*,
    link::Link,
    table_focusable::{FocusableTable, TableCell, TableBuilder},
    list::List,
};
use crate::model::{Standing, ClubStats, PlayerLanding};

/// Create the standings table with team links
pub fn create_standings_table(
    standings: &[Standing],
    view: GroupBy,
) -> FocusableTable {
    let mut builder = TableBuilder::new()
        .headers(vec![
            "Team".to_string(),
            "GP".to_string(),
            "W".to_string(),
            "L".to_string(),
            "OTL".to_string(),
            "PTS".to_string(),
            "GF".to_string(),
            "GA".to_string(),
            "DIFF".to_string(),
        ]);

    for team in standings {
        let team_link = Link::team(
            &team.team_common_name,
            &team.team_abbrev,
        );

        builder = builder.add_row(vec![
            TableCell::Widget(Box::new(team_link)),
            TableCell::Text(team.games_played.to_string()),
            TableCell::Text(team.wins.to_string()),
            TableCell::Text(team.losses.to_string()),
            TableCell::Text(team.ot_losses.unwrap_or(0).to_string()),
            TableCell::Text(team.points.to_string()),
            TableCell::Text(team.goals_for.to_string()),
            TableCell::Text(team.goals_against.to_string()),
            TableCell::Text(team.goal_diff.to_string()),
        ]);
    }

    builder.build()
}

/// Create the team roster with player links
pub fn create_team_roster(
    team_abbrev: &str,
    club_stats: &ClubStats,
) -> List {
    let mut list = List::new();

    // Add section header
    list.add_item(Box::new(SectionHeader::new("Forwards")));

    // Add forwards
    for skater in club_stats.skaters.iter()
        .filter(|s| s.position == "C" || s.position == "L" || s.position == "R") {
        let player_link = Link::player(
            format!("{} {} ({})",
                skater.first_name,
                skater.last_name,
                skater.position
            ),
            skater.player_id,
        );
        list.add_item(Box::new(player_link));
    }

    // Add section header
    list.add_item(Box::new(SectionHeader::new("Defense")));

    // Add defensemen
    for skater in club_stats.skaters.iter()
        .filter(|s| s.position == "D") {
        let player_link = Link::player(
            format!("{} {}", skater.first_name, skater.last_name),
            skater.player_id,
        );
        list.add_item(Box::new(player_link));
    }

    // Add section header
    list.add_item(Box::new(SectionHeader::new("Goalies")));

    // Add goalies
    for goalie in &club_stats.goalies {
        let player_link = Link::player(
            format!("{} {}", goalie.first_name, goalie.last_name),
            goalie.player_id,
        );
        list.add_item(Box::new(player_link));
    }

    list
}

/// Create player career stats with team links
pub fn create_player_career_table(
    player_info: &PlayerLanding,
) -> FocusableTable {
    let mut builder = TableBuilder::new()
        .headers(vec![
            "Season".to_string(),
            "Team".to_string(),
            "GP".to_string(),
            "G".to_string(),
            "A".to_string(),
            "PTS".to_string(),
            "+/-".to_string(),
            "PIM".to_string(),
        ]);

    for season in &player_info.season_totals {
        let team_link = Link::team(
            &season.team_name,
            &season.team_abbrev,
        );

        builder = builder.add_row(vec![
            TableCell::Text(format!("{}", season.season_id)),
            TableCell::Widget(Box::new(team_link)),
            TableCell::Text(season.games_played.to_string()),
            TableCell::Text(season.goals.to_string()),
            TableCell::Text(season.assists.to_string()),
            TableCell::Text(season.points.to_string()),
            TableCell::Text(season.plus_minus.to_string()),
            TableCell::Text(season.pim.to_string()),
        ]);
    }

    builder.build()
}
```

**Implementation Tasks:**
1. Add stats formatting helpers
2. Create section headers for organization
3. Add team logos (Unicode or ASCII art)
4. Color coding for playoff positions

**Tests to Write:**
- `test_standings_table_creation()` - Creates correct structure
- `test_team_roster_creation()` - Groups players by position
- `test_career_stats_creation()` - Shows all seasons

---

### Step 4.2: Update Standings State and Handler

**File:** `src/tui/standings/state.rs`

```rust
use crate::tui::widgets::tree::WidgetTree;
use crate::tui::navigation::NavigationStack;

pub struct State {
    /// The widget tree for the current view
    pub widget_tree: Option<WidgetTree>,
    /// Navigation stack for deep navigation
    pub navigation_stack: NavigationStack<StandingsPanel>,
    /// Current standings view
    pub view: GroupBy,
    /// Whether we're in subtab mode
    pub subtab_focused: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            widget_tree: None,
            navigation_stack: NavigationStack::new(StandingsPanel::Main),
            view: GroupBy::Division,
            subtab_focused: false,
        }
    }

    /// Navigate to a team
    pub fn navigate_to_team(&mut self, team_abbrev: String) {
        self.navigation_stack.push(StandingsPanel::Team { team_abbrev });
        // Widget tree will be rebuilt by the view
    }

    /// Navigate to a player
    pub fn navigate_to_player(&mut self, player_id: i64) {
        self.navigation_stack.push(StandingsPanel::Player { player_id });
        // Widget tree will be rebuilt by the view
    }

    /// Handle navigation actions from widgets
    pub fn handle_navigation_action(&mut self, action: NavigationAction) {
        match action {
            NavigationAction::NavigateToTeam(team_abbrev) => {
                self.navigate_to_team(team_abbrev);
            }
            NavigationAction::NavigateToPlayer(player_id) => {
                self.navigate_to_player(player_id);
            }
            NavigationAction::PopPanel => {
                self.navigation_stack.pop();
            }
            _ => {}
        }
    }
}
```

**File:** `src/tui/standings/handler.rs`

```rust
pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // If we have a widget tree, delegate to it
    if let Some(ref mut tree) = state.widget_tree {
        if tree.handle_input(key) {
            // Check for navigation actions
            // This would need to be implemented through a channel or callback
            return true;
        }
    }

    // Fallback to legacy handling for migration period
    handle_legacy_key(key, state, shared_data, refresh_tx).await
}
```

**Implementation Tasks:**
1. Wire up navigation actions from widgets
2. Update breadcrumbs on navigation
3. Cache loaded data (team rosters, player info)
4. Handle loading states for API calls

**Tests to Write:**
- `test_deep_navigation_flow()` - Full navigation chain
- `test_breadcrumb_sync()` - Breadcrumbs match stack
- `test_navigation_persistence()` - State preserved on tab switch

---

## Phase 5: Game Details Player Links

### Step 5.1: Remove Old Navigation Code

**Files to modify:**
- `src/tui/scores/game_details/state.rs` - Remove PlayerSection enum
- `src/tui/scores/game_details/handler.rs` - Remove complex navigation

### Step 5.2: Create Game Details Widgets

**File:** `src/tui/scores/game_details/widgets.rs`

```rust
use crate::tui::widgets::{
    focus::*,
    link::Link,
    list::List,
    table_focusable::{FocusableTable, TableCell, TableBuilder},
};
use crate::model::{GameDetails, ScoringSummary, PlayerStats};

/// Create the scoring summary with player links
pub fn create_scoring_summary(summary: &ScoringSummary) -> List {
    let mut list = List::new();

    for (period_num, period) in summary.periods.iter().enumerate() {
        // Add period header
        list.add_item(Box::new(PeriodHeader::new(period_num + 1)));

        for goal in &period.goals {
            let mut goal_row = GoalRow::new();

            // Add scorer link
            goal_row.add_scorer(Link::player(
                &goal.scorer_name,
                goal.scorer_id,
            ));

            // Add assist links
            for assist in &goal.assists {
                goal_row.add_assist(Link::player(
                    &assist.name,
                    assist.player_id,
                ));
            }

            list.add_item(Box::new(goal_row));
        }
    }

    list
}

/// Create skater stats table with player links
pub fn create_skater_stats_table(
    skaters: &[PlayerStats],
    team_name: &str,
) -> FocusableTable {
    let mut builder = TableBuilder::new()
        .headers(vec![
            "Player".to_string(),
            "Pos".to_string(),
            "G".to_string(),
            "A".to_string(),
            "P".to_string(),
            "+/-".to_string(),
            "SOG".to_string(),
            "Hits".to_string(),
            "BS".to_string(),
            "TOI".to_string(),
        ]);

    for skater in skaters {
        let player_link = Link::player(
            format!("{} {}", skater.first_name, skater.last_name),
            skater.player_id,
        );

        builder = builder.add_row(vec![
            TableCell::Widget(Box::new(player_link)),
            TableCell::Text(skater.position.clone()),
            TableCell::Text(skater.goals.to_string()),
            TableCell::Text(skater.assists.to_string()),
            TableCell::Text(skater.points.to_string()),
            TableCell::Text(format!("{:+}", skater.plus_minus)),
            TableCell::Text(skater.shots.to_string()),
            TableCell::Text(skater.hits.to_string()),
            TableCell::Text(skater.blocked_shots.to_string()),
            TableCell::Text(format_time(skater.toi_seconds)),
        ]);
    }

    builder.build()
}

/// Create goalie stats table with player links
pub fn create_goalie_stats_table(
    goalies: &[GoalieStats],
    team_name: &str,
) -> FocusableTable {
    let mut builder = TableBuilder::new()
        .headers(vec![
            "Goalie".to_string(),
            "SA".to_string(),
            "SV".to_string(),
            "GA".to_string(),
            "SV%".to_string(),
            "TOI".to_string(),
        ]);

    for goalie in goalies {
        let player_link = Link::player(
            format!("{} {}", goalie.first_name, goalie.last_name),
            goalie.player_id,
        );

        builder = builder.add_row(vec![
            TableCell::Widget(Box::new(player_link)),
            TableCell::Text(goalie.shots_against.to_string()),
            TableCell::Text(goalie.saves.to_string()),
            TableCell::Text(goalie.goals_against.to_string()),
            TableCell::Text(format!("{:.3}", goalie.save_pct)),
            TableCell::Text(format_time(goalie.toi_seconds)),
        ]);
    }

    builder.build()
}

/// Create penalty summary with player links
pub fn create_penalty_summary(penalties: &[Penalty]) -> List {
    let mut list = List::new();

    for penalty in penalties {
        let player_link = Link::player(
            &penalty.player_name,
            penalty.player_id,
        );

        let penalty_row = PenaltyRow::new(
            player_link,
            &penalty.infraction,
            penalty.minutes,
            &penalty.time,
        );

        list.add_item(Box::new(penalty_row));
    }

    list
}

/// Composite widget for complete game details
pub struct GameDetailsWidget {
    id: WidgetId,
    sections: Vec<Box<dyn Focusable>>,
    focused: bool,
}

impl GameDetailsWidget {
    pub fn from_game_details(details: &GameDetails) -> Self {
        let mut sections = vec![];

        // Add scoring summary
        sections.push(Box::new(create_scoring_summary(&details.scoring)) as Box<dyn Focusable>);

        // Add home team stats
        sections.push(Box::new(create_skater_stats_table(
            &details.home_skaters,
            &details.home_team,
        )) as Box<dyn Focusable>);

        // Add away team stats
        sections.push(Box::new(create_skater_stats_table(
            &details.away_skaters,
            &details.away_team,
        )) as Box<dyn Focusable>);

        // Add goalie stats
        sections.push(Box::new(create_goalie_stats_table(
            &details.home_goalies,
            &details.home_team,
        )) as Box<dyn Focusable>);

        sections.push(Box::new(create_goalie_stats_table(
            &details.away_goalies,
            &details.away_team,
        )) as Box<dyn Focusable>);

        // Add penalties
        sections.push(Box::new(create_penalty_summary(&details.penalties)) as Box<dyn Focusable>);

        Self {
            id: WidgetId::new(),
            sections,
            focused: false,
        }
    }
}

impl Focusable for GameDetailsWidget {
    // Implement delegation to sections
    // Tab navigates through all player links across all sections
}
```

**Implementation Tasks:**
1. Create custom row widgets for goals and penalties
2. Add period/section headers
3. Format statistics properly
4. Add team colors/styling

**Tests to Write:**
- `test_all_players_are_links()` - Every player name is clickable
- `test_tab_navigation_across_sections()` - Tab moves through all links
- `test_player_navigation()` - Clicking navigates to player

---

## Phase 6: Testing Strategy

### Step 6.1: Widget Testing Framework

**File:** `tests/widgets/test_helpers.rs`

```rust
use ratatui::prelude::*;
use nhl::tui::widgets::focus::*;

/// Create a test buffer with expected size
pub fn test_buffer(width: u16, height: u16) -> Buffer {
    Buffer::empty(Rect::new(0, 0, width, height))
}

/// Assert buffer contents match expected
pub fn assert_buffer_eq(buffer: &Buffer, expected: &[&str]) {
    let mut actual = vec![];
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            line.push_str(cell.symbol());
        }
        actual.push(line);
    }

    for (i, (actual_line, expected_line)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual_line.trim_end(),
            expected_line.trim_end(),
            "Line {} differs:\nActual:   '{}'\nExpected: '{}'",
            i, actual_line, expected_line
        );
    }
}

/// Create a mock focusable widget for testing
pub struct MockWidget {
    id: WidgetId,
    focused: bool,
    handle_count: Arc<AtomicUsize>,
}

impl MockWidget {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            focused: false,
            handle_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn handle_count(&self) -> usize {
        self.handle_count.load(Ordering::Relaxed)
    }
}

impl Focusable for MockWidget {
    // ... implementation
}
```

### Step 6.2: Unit Tests for Each Widget

**File:** `tests/widgets/link_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_renders_normal() {
        let config = test_config();
        let mut buf = test_buffer(20, 1);
        let area = Rect::new(0, 0, 20, 1);

        let link = Link::new("Test Link");
        link.render(area, &mut buf, &config);

        let expected = ["Test Link           "];
        assert_buffer_eq(&buf, &expected);
    }

    #[test]
    fn test_link_renders_focused() {
        let config = test_config();
        let mut buf = test_buffer(20, 1);
        let area = Rect::new(0, 0, 20, 1);

        let mut link = Link::new("Test Link");
        link.set_focused(true);
        link.render(area, &mut buf, &config);

        let expected = ["▶ Test Link         "];
        assert_buffer_eq(&buf, &expected);

        // Check styling
        let cell = buf.get(0, 0);
        assert_eq!(cell.fg, config.colors.selection_fg);
    }

    #[test]
    fn test_link_activates_on_enter() {
        let activated = Arc::new(AtomicBool::new(false));
        let activated_clone = activated.clone();

        let mut link = Link::new("Test")
            .with_action(move || {
                activated_clone.store(true, Ordering::Relaxed);
                NavigationAction::NavigateToPlayer(123)
            });

        link.set_focused(true);

        let result = link.handle_input(KeyEvent::from(KeyCode::Enter));

        assert!(matches!(result, InputResult::Navigate(_)));
        assert!(activated.load(Ordering::Relaxed));
    }
}
```

### Step 6.3: Integration Tests

**File:** `tests/integration/navigation_test.rs`

```rust
#[tokio::test]
async fn test_complete_navigation_flow() {
    let mut app = setup_test_app_with_data().await;

    // Start at standings
    assert_eq!(app.current_tab(), CurrentTab::Standings);

    // Navigate to team
    app.send_key(KeyCode::Down).await; // Enter subtab
    app.send_key(KeyCode::Down).await; // Focus first team
    app.send_key(KeyCode::Enter).await; // Select team

    // Verify we're on team panel
    let breadcrumbs = app.get_breadcrumbs();
    assert_eq!(breadcrumbs, vec!["NHL", "Standings", "Canadiens"]);

    // Navigate to player
    app.send_key(KeyCode::Down).await; // Focus first player
    app.send_key(KeyCode::Enter).await; // Select player

    // Verify we're on player panel
    let breadcrumbs = app.get_breadcrumbs();
    assert_eq!(breadcrumbs, vec!["NHL", "Standings", "Canadiens", "Nick Suzuki"]);

    // Navigate to another team via career stats
    app.send_key(KeyCode::Tab).await; // Tab to career table
    app.send_key(KeyCode::Down).await; // Select a season
    app.send_key(KeyCode::Enter).await; // Click team link

    // Verify we're on new team
    let breadcrumbs = app.get_breadcrumbs();
    assert_eq!(breadcrumbs, vec![
        "NHL", "Standings", "Canadiens", "Nick Suzuki", "Vegas"
    ]);

    // Use breadcrumb to go back
    app.focus_breadcrumb(1).await; // Focus "Standings"
    app.send_key(KeyCode::Enter).await;

    // Verify we're back at standings
    let breadcrumbs = app.get_breadcrumbs();
    assert_eq!(breadcrumbs, vec!["NHL", "Standings"]);
}

#[tokio::test]
async fn test_game_details_player_links() {
    let mut app = setup_test_app_with_game().await;

    // Navigate to game details
    app.go_to_game_details(2024020001).await;

    // Count all focusable player links
    let focusable_count = app.count_focusable_widgets();
    assert!(focusable_count > 20); // Should have many player links

    // Tab through first few links
    for _ in 0..5 {
        app.send_key(KeyCode::Tab).await;

        let focused = app.get_focused_widget_text();
        assert!(focused.contains(" ")); // Player names have spaces
    }

    // Click a player
    app.send_key(KeyCode::Enter).await;

    // Verify navigation to player
    let breadcrumbs = app.get_breadcrumbs();
    assert!(breadcrumbs.last().unwrap().contains(" ")); // Player name
}
```

---

## Phase 7: Migration Strategy

### Step 7.1: Feature Flag Implementation

**File:** `src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // Existing fields...

    /// Enable experimental widget-based navigation
    #[serde(default = "default_widget_nav")]
    pub use_widget_navigation: bool,

    /// Tabs to enable widget navigation for
    #[serde(default = "default_widget_tabs")]
    pub widget_enabled_tabs: Vec<String>,
}

fn default_widget_nav() -> bool {
    false
}

fn default_widget_tabs() -> Vec<String> {
    vec![] // Start with no tabs enabled
}

impl Config {
    pub fn is_widget_nav_enabled_for(&self, tab: &str) -> bool {
        self.use_widget_navigation && self.widget_enabled_tabs.contains(&tab.to_string())
    }
}
```

### Step 7.2: Parallel Implementation Pattern

**File:** `src/tui/standings/handler.rs`

```rust
pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    config: &Config,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    if config.is_widget_nav_enabled_for("standings") {
        handle_key_widget_mode(key, state, shared_data, refresh_tx).await
    } else {
        handle_key_legacy_mode(key, state, shared_data, refresh_tx).await
    }
}

async fn handle_key_widget_mode(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // New widget-based implementation
    if let Some(ref mut tree) = state.widget_tree {
        tree.handle_input(key)
    } else {
        false
    }
}

async fn handle_key_legacy_mode(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // Existing implementation
    // ... current code ...
}
```

### Step 7.3: Rollout Plan

```toml
# Phase 1: Internal testing
[config]
use_widget_navigation = true
widget_enabled_tabs = ["standings"]

# Phase 2: Add game details
[config]
use_widget_navigation = true
widget_enabled_tabs = ["standings", "scores"]

# Phase 3: Full rollout
[config]
use_widget_navigation = true
widget_enabled_tabs = ["standings", "scores", "settings"]

# Phase 4: Remove legacy code
# Delete legacy handlers and remove feature flags
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure ✅
- [ ] Create `src/tui/widgets/focus.rs` with core traits
- [ ] Implement `WidgetTree` for focus management
- [ ] Create `Link` widget
- [ ] Write unit tests for focus system
- [ ] Test focus navigation

### Phase 2: Container Widgets ✅
- [ ] Implement `List` widget
- [ ] Implement `FocusableTable` widget
- [ ] Test 2D navigation
- [ ] Test focus delegation

### Phase 3: Breadcrumbs ✅
- [ ] Create `BreadcrumbWidget` with clickable items
- [ ] Integrate with navigation stack
- [ ] Test breadcrumb navigation

### Phase 4: Standings Navigation ✅
- [ ] Create standings widget builders
- [ ] Implement team links
- [ ] Implement player links
- [ ] Wire up navigation actions
- [ ] Test deep navigation chains

### Phase 5: Game Details ✅
- [ ] Remove old navigation code
- [ ] Create game details widgets
- [ ] Make all player names clickable
- [ ] Test tab navigation through all links

### Phase 6: Testing ✅
- [ ] Create testing helpers
- [ ] Write widget unit tests
- [ ] Write integration tests
- [ ] Visual regression tests

### Phase 7: Migration ✅
- [ ] Add feature flags
- [ ] Implement parallel handlers
- [ ] Test with flags enabled/disabled
- [ ] Document migration process

---

## Risk Mitigation

### Performance Risks
- **Large widget trees**: Implement widget caching and lazy evaluation
- **Frequent re-renders**: Use dirty flags and selective updates
- **Memory usage**: Implement widget pooling for common types

### Compatibility Risks
- **Breaking changes**: Use feature flags for gradual rollout
- **State migration**: Keep both systems until fully tested
- **User disruption**: Maintain exact same UX during transition

### Technical Risks
- **Focus loops**: Add cycle detection in focus navigation
- **Input conflicts**: Clear event propagation rules
- **Memory leaks**: Careful lifecycle management for callbacks

---

## Success Metrics

1. **Code Quality**
   - 90%+ test coverage for new code
   - No performance regression
   - Reduced cyclomatic complexity

2. **User Experience**
   - All player/team names clickable
   - Consistent navigation patterns
   - Visual focus indicators everywhere

3. **Developer Experience**
   - Adding new focusable elements is trivial
   - Clear widget composition patterns
   - Self-documenting widget API

4. **Maintenance**
   - Reduced bug reports for navigation
   - Faster feature development
   - Easier onboarding for new developers

---

## Future Enhancements

Once the widget system is in place, these become possible:

1. **Mouse Support**: Click on any focusable widget
2. **Keyboard Shortcuts**: Jump to specific widgets with hotkeys
3. **Accessibility**: Screen reader support via widget metadata
4. **Animations**: Focus transitions and widget state changes
5. **Theming**: Widget-level style customization
6. **Plugin System**: Custom widgets for extensions

---

## Conclusion

This refactoring will transform the NHL TUI from a state-machine-based navigation system to a modern, composable widget architecture. The benefits include:

- **Consistency**: Same navigation patterns everywhere
- **Maintainability**: Clear separation of concerns
- **Extensibility**: Easy to add new features
- **Testability**: Widgets testable in isolation
- **User Experience**: Every relevant item is clickable

The phased approach with feature flags ensures a safe migration path with minimal disruption to users.