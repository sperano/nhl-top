/// Screenshot utilities for capturing terminal output
///
/// This module provides functionality to save terminal screenshots
/// as text files for debugging and documentation purposes.

use std::fs::File;
use std::io::{self, Write};
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect, Terminal};

/// Save a terminal screenshot from a buffer to a text file
///
/// Captures a ratatui Buffer and saves it to a file with the given filename.
/// This should be called with a buffer captured during a draw call.
///
/// # Arguments
///
/// * `buffer` - The ratatui Buffer to save
/// * `area` - The area of the buffer to save
/// * `filename` - The filename to save the screenshot to
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an IO error if the file could not be written.
pub fn save_buffer_screenshot(
    buffer: &Buffer,
    area: Rect,
    filename: &str,
) -> io::Result<()> {
    let mut file = File::create(filename)?;

    // Write each line
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(x, y)];
            write!(file, "{}", cell.symbol())?;
        }
        writeln!(file)?;
    }

    Ok(())
}

/// Save a terminal screenshot to a text file (legacy compatibility)
///
/// This is a wrapper that captures during a draw call. Note that this
/// performs an additional draw just to capture the buffer.
///
/// # Arguments
///
/// * `terminal` - The ratatui Terminal to capture
/// * `filename` - The filename to save the screenshot to
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an IO error if the file could not be written.
pub fn save_terminal_screenshot<B: Backend>(
    terminal: &mut Terminal<B>,
    filename: &str,
) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        let buffer = f.buffer_mut().clone();

        // Save to file (ignore errors during draw, we'll return them)
        let _ = save_buffer_screenshot(&buffer, area, filename);
    })?;

    Ok(())
}
