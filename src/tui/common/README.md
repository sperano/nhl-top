# TUI Common Widgets and Utilities

This module contains reusable UI components and utilities for the NHL TUI application.

## Widgets

### Scrollable (`scrollable.rs`)

A reusable wrapper that makes any content scrollable with keyboard navigation.

**Features:**
- Automatic scroll bounds management
- Keyboard navigation (Up/Down, PageUp/PageDown, Home/End)
- Viewport and content height tracking
- Helper method for rendering with Paragraph widget

**Usage Example:**
```rust
use crate::tui::common::scrollable::Scrollable;

let mut scrollable = Scrollable::new();

// In render function:
scrollable.render_paragraph(f, area, content_string, None);

// In key handler:
if scrollable.handle_key(key) {
    return true; // Key was handled
}
```

**API:**
- `new()` - Create new scrollable instance
- `handle_key(key)` - Handle scroll keys, returns true if handled
- `render_paragraph(f, area, content, block)` - Render scrollable content
- `reset()` - Reset scroll position to top
- `update_viewport_height(height)` - Update visible area size
- `update_content_height(height)` - Update total content size

---

### Tab Bar (`tab_bar.rs`)

Renders a horizontal tab bar with separators and selection highlighting.

**Features:**
- Box-drawing character separators (│)
- Selection highlighting with configurable colors
- Focused/unfocused states
- Auto-sized based on tab names

**Usage Example:**
```rust
use crate::tui::common::tab_bar;

tab_bar::render(
    f,
    area,
    &["Scores", "Standings", "Stats"],
    selected_index,
    focused,
    selection_fg,
    unfocused_selection_fg,
);
```

---

### Separator (`separator.rs`)

Builds horizontal separator lines with box-drawing connectors.

**Features:**
- Auto-positioned connectors (┴) under tab gaps
- Fills remaining width with horizontal lines (─)
- Styled output

**Usage Example:**
```rust
use crate::tui::common::separator::build_tab_separator_line;

let tab_names = vec!["Tab1".to_string(), "Tab2".to_string()];
let separator = build_tab_separator_line(
    tab_names.into_iter(),
    area_width,
    style,
);
```

---

### Styling (`styling.rs`)

Common styling utilities for consistent look and feel.

**Features:**
- Base tab styles (focused/unfocused)
- Selection styles (focused/unfocused selected items)
- Color configuration support

**Usage Example:**
```rust
use crate::tui::common::styling::{base_tab_style, selection_style};

let base = base_tab_style(focused);
let selected = selection_style(
    base,
    is_selected,
    focused,
    selection_fg,
    unfocused_selection_fg,
);
```

---

## Design Patterns

### Tab State Pattern

Each tab follows a consistent state structure:
- **State struct** - Holds UI state (selection, focus, scroll position)
- **View functions** - Pure rendering functions
- **Handler functions** - Event handling logic

**Example:**
```
src/tui/scores/
├── state.rs      # State { selected_index, subtab_focused, ... }
├── view.rs       # render_subtabs(), render_content()
├── handler.rs    # handle_key()
└── mod.rs        # Public exports
```

### Scrollable Pattern

Tabs with long content use the `Scrollable` wrapper:
1. Create `Scrollable` instance in state
2. Call `render_paragraph()` during render
3. Call `handle_key()` in key handler

### Subtab Pattern

Tabs with subtabs (Scores, Standings) follow:
1. `subtab_focused` boolean in state
2. `render_subtabs()` for subtab bar
3. Separate key handling for subtab vs main tab mode

---

## Common Constants

Defined in respective modules:
- `TAB_BAR_HEIGHT` - Height of main tab bar
- `SUBTAB_BAR_HEIGHT` - Height of subtab bar
- `STATUS_BAR_HEIGHT` - Height of status bar
- `SEPARATOR_*` - Separator line configuration

---

## Testing

Each widget should be tested for:
- Correct bounds checking
- Proper scroll limit handling
- Style application
- Key handling behavior

See existing tests in `scrollable.rs` and `styling.rs` for examples.
