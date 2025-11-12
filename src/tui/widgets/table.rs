/// Focusable table widget for 2D navigation
///
/// This widget provides a generic table with keyboard navigation support.
/// It integrates with the focus system and supports clickable cells.

use super::focus::*;
use crate::config::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
};

/// A 2D navigable table widget with focus support
///
/// The table supports:
/// - Up/Down navigation between rows
/// - Left/Right navigation between clickable columns
/// - Enter to activate the current cell
/// - Automatic scrolling
/// - Customizable styling
pub struct FocusableTable<T> {
    id: WidgetId,
    /// Table data rows
    rows: Vec<T>,
    /// Column definitions
    columns: Vec<ColumnDef<T>>,
    /// Optional table header text
    header: Option<String>,
    /// Currently selected row index
    selected_row: usize,
    /// Currently selected column index
    selected_col: usize,
    /// Whether table has focus
    focused: bool,
    /// Scroll offset (first visible row)
    scroll_offset: usize,
    /// Number of visible rows (calculated during render)
    visible_rows: usize,
    /// Callback when cell is activated
    on_activate: Option<Box<dyn FnMut(&T) -> NavigationAction + Send>>,
    /// Visual styling
    style: TableStyle,
}

/// Column definition for FocusableTable
pub struct ColumnDef<T> {
    /// Column header text
    pub header: String,
    /// Column width (in characters)
    pub width: usize,
    /// Extract cell text from row data
    pub cell_fn: Box<dyn Fn(&T) -> String>,
    /// Text alignment
    pub align: Alignment,
    /// Whether this column is clickable/activatable
    pub clickable: bool,
}

/// Text alignment for table cells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// Visual styling for tables
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Show borders around table
    pub borders: bool,
    /// Show row separators
    pub row_separators: bool,
    /// Highlight entire row or just selected cell
    pub highlight_mode: HighlightMode,
    /// Left margin
    pub margin: u16,
}

/// How to highlight the selected cell/row
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightMode {
    /// Highlight only the selected cell
    Cell,
    /// Highlight the entire selected row
    Row,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            borders: false,
            row_separators: false,
            highlight_mode: HighlightMode::Row,
            margin: 0,
        }
    }
}

impl<T> ColumnDef<T> {
    /// Create a new column definition
    pub fn new<F>(
        header: impl Into<String>,
        width: usize,
        cell_fn: F,
        align: Alignment,
        clickable: bool,
    ) -> Self
    where
        F: Fn(&T) -> String + 'static,
    {
        Self {
            header: header.into(),
            width,
            cell_fn: Box::new(cell_fn),
            align,
            clickable,
        }
    }
}

impl<T> FocusableTable<T> {
    /// Create a new table with the given data
    pub fn new(rows: Vec<T>) -> Self {
        Self {
            id: WidgetId::new(),
            rows,
            columns: Vec::new(),
            header: None,
            selected_row: 0,
            selected_col: 0,
            focused: false,
            scroll_offset: 0,
            visible_rows: 10,
            on_activate: None,
            style: TableStyle::default(),
        }
    }

    /// Set the column definitions
    pub fn with_columns(mut self, columns: Vec<ColumnDef<T>>) -> Self {
        self.columns = columns;
        // Ensure selected_col points to a clickable column
        if !self.columns.is_empty() {
            self.selected_col = self.find_next_clickable_column(0).unwrap_or(0);
        }
        self
    }

    /// Set the table header
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Set the visual style
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the activation callback
    pub fn with_on_activate<F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> NavigationAction + Send + 'static,
    {
        self.on_activate = Some(Box::new(f));
        self
    }

    /// Get the number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Select a specific row
    pub fn select_row(&mut self, row: usize) {
        if row < self.rows.len() {
            self.selected_row = row;
            self.ensure_visible();
        }
    }

    /// Get the currently selected row index
    pub fn selected_row(&self) -> Option<usize> {
        if self.rows.is_empty() {
            None
        } else {
            Some(self.selected_row)
        }
    }

    /// Get the currently selected row data
    pub fn selected_row_data(&self) -> Option<&T> {
        self.rows.get(self.selected_row)
    }

    /// Find the next clickable column starting from the given index
    fn find_next_clickable_column(&self, start: usize) -> Option<usize> {
        for i in start..self.columns.len() {
            if self.columns[i].clickable {
                return Some(i);
            }
        }
        None
    }

    /// Find the previous clickable column starting from the given index
    fn find_prev_clickable_column(&self, start: usize) -> Option<usize> {
        if start == 0 {
            return None;
        }
        for i in (0..start).rev() {
            if self.columns[i].clickable {
                return Some(i);
            }
        }
        None
    }

    /// Move selection up one row
    fn move_up(&mut self) -> bool {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Move selection down one row
    fn move_down(&mut self) -> bool {
        if self.selected_row + 1 < self.rows.len() {
            self.selected_row += 1;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Move selection left to previous clickable column
    fn move_left(&mut self) -> bool {
        if let Some(prev_col) = self.find_prev_clickable_column(self.selected_col) {
            self.selected_col = prev_col;
            true
        } else {
            false
        }
    }

    /// Move selection right to next clickable column
    fn move_right(&mut self) -> bool {
        if let Some(next_col) = self.find_next_clickable_column(self.selected_col + 1) {
            self.selected_col = next_col;
            true
        } else {
            false
        }
    }

    /// Ensure the selected row is visible in the viewport
    fn ensure_visible(&mut self) {
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        } else if self.selected_row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_row.saturating_sub(self.visible_rows - 1);
        }
    }

    /// Jump to first row
    fn jump_to_first(&mut self) -> bool {
        if self.selected_row > 0 {
            self.selected_row = 0;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Jump to last row
    fn jump_to_last(&mut self) -> bool {
        let last = self.rows.len().saturating_sub(1);
        if self.selected_row < last {
            self.selected_row = last;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Scroll by page
    fn page_down(&mut self) -> bool {
        let target = (self.selected_row + self.visible_rows).min(self.rows.len() - 1);
        if target != self.selected_row {
            self.selected_row = target;
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    fn page_up(&mut self) -> bool {
        let target = self.selected_row.saturating_sub(self.visible_rows);
        if target != self.selected_row {
            self.selected_row = target;
            self.ensure_visible();
            true
        } else {
            false
        }
    }
}

impl<T> Focusable for FocusableTable<T>
where
    T: 'static,
{
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        !self.rows.is_empty()
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

        let is_shift = event.modifiers.contains(KeyModifiers::SHIFT);
        match event.code {
            // Up arrow or Shift+Tab (BackTab) - move up
            KeyCode::Up | KeyCode::BackTab => {
                if self.move_up() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Shift+Tab sends as Tab with SHIFT modifier (for tests/some terminals)
            KeyCode::Tab if is_shift => {
                if self.move_up() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Down arrow or Tab (without shift) - move down
            KeyCode::Down | KeyCode::Tab => {
                if self.move_down() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Left arrow - move left
            KeyCode::Left => {
                if self.move_left() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Right arrow - move right
            KeyCode::Right => {
                if self.move_right() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Enter key - activate cell
            KeyCode::Enter => {
                // Check if current column is clickable
                if self.columns.get(self.selected_col).map(|c| c.clickable).unwrap_or(false) {
                    // Get row data before mutable borrow
                    if self.selected_row < self.rows.len() {
                        if let Some(ref mut callback) = self.on_activate {
                            let row_data = &self.rows[self.selected_row];
                            let action = callback(row_data);
                            return InputResult::Navigate(action);
                        }
                    }
                }
                InputResult::Handled
            }
            // Home key - jump to first
            KeyCode::Home => {
                if self.jump_to_first() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // End key - jump to last
            KeyCode::End => {
                if self.jump_to_last() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // PageUp key - page up
            KeyCode::PageUp => {
                if self.page_up() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // PageDown key - page down
            KeyCode::PageDown => {
                if self.page_down() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // Any other key combination
            _ => InputResult::NotHandled,
        }
    }

    fn focus_first(&mut self) {
        self.select_row(0);
        self.set_focused(true);
    }

    fn focus_last(&mut self) {
        let last = self.rows.len().saturating_sub(1);
        self.select_row(last);
        self.set_focused(true);
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_row()
    }
}

impl<T> super::RenderableWidget for FocusableTable<T> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let x = area.x + self.style.margin;
        let mut y = area.y;

        // Render header if present
        if let Some(ref header_text) = self.header {
            // TODO: Render section header
            buf.set_string(x, y, header_text, Style::default().add_modifier(Modifier::BOLD));
            y += 1;
            if y >= area.bottom() {
                return;
            }
        }

        // Calculate visible rows
        let available_height = area.bottom().saturating_sub(y) as usize;
        let end_row = (self.scroll_offset + available_height).min(self.rows.len());

        // Render column headers
        let mut col_x = x;
        for col in &self.columns {
            let header_style = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            let text = format!("{:width$}", col.header, width = col.width);
            buf.set_string(col_x, y, &text, header_style);
            col_x += col.width as u16 + 2; // +2 for spacing
        }
        y += 1;

        // Render rows
        for (idx, row) in self.rows[self.scroll_offset..end_row].iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let row_idx = self.scroll_offset + idx;
            let is_selected = row_idx == self.selected_row && self.focused;

            col_x = x;
            for (col_idx, col) in self.columns.iter().enumerate() {
                let cell_text = (col.cell_fn)(row);
                let formatted = self.format_cell(&cell_text, col.width, col.align);

                let is_cell_selected = is_selected &&
                    (self.style.highlight_mode == HighlightMode::Row || col_idx == self.selected_col);

                let cell_style = if is_cell_selected {
                    Style::default().fg(config.selection_fg)
                } else {
                    Style::default()
                };

                buf.set_string(col_x, y, &formatted, cell_style);
                col_x += col.width as u16 + 2;
            }

            y += 1;
        }

        // Show scroll indicators
        if self.scroll_offset > 0 && area.width > 0 {
            buf.set_string(area.right() - 1, area.y, "▲", Style::default());
        }
        if end_row < self.rows.len() && area.width > 0 {
            buf.set_string(area.right() - 1, area.bottom() - 1, "▼", Style::default());
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        let header_height = if self.header.is_some() { 1 } else { 0 };
        let col_header_height = if !self.columns.is_empty() { 1 } else { 0 };
        let rows_height = self.rows.len() as u16;
        Some(header_height + col_header_height + rows_height)
    }
}

impl<T> FocusableTable<T> {
    /// Format a cell with alignment
    fn format_cell(&self, text: &str, width: usize, align: Alignment) -> String {
        let text_len = text.len();
        if text_len >= width {
            // Truncate
            if width > 3 {
                format!("{}...", &text[..width - 3])
            } else {
                text[..width].to_string()
            }
        } else {
            // Pad
            match align {
                Alignment::Left => format!("{:<width$}", text, width = width),
                Alignment::Right => format!("{:>width$}", text, width = width),
                Alignment::Center => {
                    let left_pad = (width - text_len) / 2;
                    let right_pad = width - text_len - left_pad;
                    format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DisplayConfig;

    #[derive(Debug, Clone)]
    struct TestRow {
        name: String,
        value: i32,
    }

    fn test_config() -> DisplayConfig {
        DisplayConfig::default()
    }

    fn test_table() -> FocusableTable<TestRow> {
        let rows = vec![
            TestRow { name: "Row 1".to_string(), value: 10 },
            TestRow { name: "Row 2".to_string(), value: 20 },
            TestRow { name: "Row 3".to_string(), value: 30 },
        ];

        FocusableTable::new(rows).with_columns(vec![
            ColumnDef::new("Name", 10, |r: &TestRow| r.name.clone(), Alignment::Left, true),
            ColumnDef::new("Value", 8, |r: &TestRow| r.value.to_string(), Alignment::Right, false),
        ])
    }

    #[test]
    fn test_table_creation() {
        let table = test_table();
        assert_eq!(table.len(), 3);
        assert!(!table.is_empty());
        assert!(table.can_focus());
        assert!(!table.is_focused());
    }

    #[test]
    fn test_table_empty() {
        let table: FocusableTable<TestRow> = FocusableTable::new(vec![]);
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
        assert!(!table.can_focus());
    }

    #[test]
    fn test_table_navigation_down() {
        let mut table = test_table();
        table.set_focused(true);

        assert_eq!(table.selected_row, 0);

        let result = table.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 1);

        let result = table.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 2);

        // At bottom - should block (return NotHandled)
        let result = table.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 2); // Stays at bottom
    }

    #[test]
    fn test_table_navigation_up() {
        let mut table = test_table();
        table.set_focused(true);
        table.select_row(2);

        assert_eq!(table.selected_row, 2);

        let result = table.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 1);

        let result = table.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 0);

        // At top - should block (return NotHandled)
        let result = table.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 0); // Stays at top
    }

    #[test]
    fn test_table_navigation_left_right() {
        let mut table = test_table();
        table.set_focused(true);

        // First column is clickable, should be selected by default
        assert_eq!(table.selected_col, 0);

        // Right - second column is not clickable, should block
        let result = table.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_col, 0); // Stays at first column

        // Left - already at leftmost clickable column, should block
        let result = table.handle_input(KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
    }

    #[test]
    fn test_table_with_multiple_clickable_columns() {
        let rows = vec![TestRow { name: "Test".to_string(), value: 1 }];
        let mut table = FocusableTable::new(rows).with_columns(vec![
            ColumnDef::new("Col1", 5, |r: &TestRow| r.name.clone(), Alignment::Left, true),
            ColumnDef::new("Col2", 5, |r: &TestRow| r.value.to_string(), Alignment::Left, true),
            ColumnDef::new("Col3", 5, |_: &TestRow| "X".to_string(), Alignment::Left, true),
        ]);
        table.set_focused(true);

        assert_eq!(table.selected_col, 0);

        // Right - move to next clickable column
        let result = table.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_col, 1);

        // Right again
        let result = table.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_col, 2);

        // Right at end - should block (return NotHandled)
        let result = table.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);

        // Left - move back
        let result = table.handle_input(KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_col, 1);
    }

    #[test]
    fn test_table_activation() {
        let rows = vec![
            TestRow { name: "Team A".to_string(), value: 1 },
            TestRow { name: "Team B".to_string(), value: 2 },
        ];

        let mut table = FocusableTable::new(rows)
            .with_columns(vec![
                ColumnDef::new("Team", 10, |r: &TestRow| r.name.clone(), Alignment::Left, true),
            ])
            .with_on_activate(|row| NavigationAction::NavigateToTeam(row.name.clone()));

        table.set_focused(true);

        // Press Enter on first row
        let result = table.handle_input(KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE));
        match result {
            InputResult::Navigate(NavigationAction::NavigateToTeam(name)) => {
                assert_eq!(name, "Team A");
            }
            _ => panic!("Expected Navigate action"),
        }
    }

    #[test]
    fn test_table_home_end() {
        let mut table = test_table();
        table.set_focused(true);

        // End - jump to last
        let result = table.handle_input(KeyEvent::new(KeyCode::End, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 2);

        // End again - no change
        let result = table.handle_input(KeyEvent::new(KeyCode::End, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);

        // Home - jump to first
        let result = table.handle_input(KeyEvent::new(KeyCode::Home, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 0);

        // Home again - no change
        let result = table.handle_input(KeyEvent::new(KeyCode::Home, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
    }

    #[test]
    fn test_table_select_row() {
        let mut table = test_table();

        table.select_row(1);
        assert_eq!(table.selected_row, 1);

        table.select_row(10); // Out of bounds
        assert_eq!(table.selected_row, 1); // Unchanged
    }

    #[test]
    fn test_table_selected_row_data() {
        let table = test_table();

        let row_data = table.selected_row_data().unwrap();
        assert_eq!(row_data.name, "Row 1");
        assert_eq!(row_data.value, 10);
    }

    #[test]
    fn test_table_focus_state() {
        let mut table = test_table();

        assert!(!table.is_focused());

        table.set_focused(true);
        assert!(table.is_focused());

        table.set_focused(false);
        assert!(!table.is_focused());
    }

    #[test]
    fn test_table_widget_id_unique() {
        let table1 = test_table();
        let table2 = test_table();

        assert_ne!(table1.widget_id(), table2.widget_id());
    }

    #[test]
    fn test_table_format_cell() {
        let table = test_table();

        // Left alignment
        assert_eq!(table.format_cell("Test", 10, Alignment::Left), "Test      ");

        // Right alignment
        assert_eq!(table.format_cell("Test", 10, Alignment::Right), "      Test");

        // Center alignment
        assert_eq!(table.format_cell("Test", 10, Alignment::Center), "   Test   ");

        // Truncation
        assert_eq!(table.format_cell("Very Long Text", 8, Alignment::Left), "Very ...");
    }

    // Regression tests for boundary navigation issues
    #[test]
    fn test_table_up_at_first_row_blocks() {
        // UP at first row should return NotHandled (blocked)
        let mut table = test_table();
        table.set_focused(true);
        assert_eq!(table.selected_row, 0);

        let result = table.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 0);
    }

    #[test]
    fn test_table_down_at_last_row_blocks() {
        // DOWN at last row should return NotHandled (blocked)
        let mut table = test_table();
        table.set_focused(true);
        table.select_row(2); // Last row

        let result = table.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 2);
    }

    #[test]
    fn test_table_tab_behaves_like_down() {
        // Tab should move down like arrow down
        let mut table = test_table();
        table.set_focused(true);
        assert_eq!(table.selected_row, 0);

        // Tab should move down
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 1);

        // Tab again
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 2);

        // Tab at last row should block
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 2);
    }

    #[test]
    fn test_table_shift_tab_behaves_like_up() {
        // Shift+Tab should move up like arrow up
        let mut table = test_table();
        table.set_focused(true);
        table.select_row(2); // Start at last row

        // Shift+Tab should move up
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 1);

        // Shift+Tab again
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(table.selected_row, 0);

        // Shift+Tab at first row should block
        let result = table.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(table.selected_row, 0);
    }
}
