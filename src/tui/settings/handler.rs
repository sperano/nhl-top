use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::{State, build_settings_list, SettingValue};
use crate::SharedDataHandle;
use crate::config;

/// Handle key events for settings tab
pub async fn handle_key(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle) -> bool {
    // If color modal is open, handle color picker keys
    if let Some(setting_name) = &state.color_modal {
        return handle_color_modal(key, state, shared_data, setting_name.clone()).await;
    }

    // If list modal is open, handle modal keys
    if let Some((setting_name, options, selected_index)) = &state.list_modal {
        return handle_list_modal(key, state, shared_data, setting_name.clone(), options.clone(), *selected_index).await;
    }

    // If we're in editing mode, handle editing keys
    if let Some((setting_name, edit_buffer)) = &state.editing {
        return handle_editing_mode(key, state, shared_data, setting_name.clone(), edit_buffer.clone()).await;
    }

    // Clear any existing status message when navigating
    if matches!(key.code, KeyCode::Up | KeyCode::Down) {
        shared_data.write().await.clear_status();
    }

    // Get the number of settings
    let data = shared_data.read().await;
    let settings = build_settings_list(&data.config);
    let num_settings = settings.len();
    drop(data);

    match key.code {
        KeyCode::Up => {
            if state.selected_setting_index > 0 {
                // Move up in the list
                state.selected_setting_index -= 1;
                true
            } else {
                // At first setting, signal to exit subtab mode
                false
            }
        }
        KeyCode::Down => {
            // Move down in the list
            if state.selected_setting_index + 1 < num_settings {
                state.selected_setting_index += 1;
            }
            true
        }
        KeyCode::Enter => {
            let setting = &settings[state.selected_setting_index];

            match &setting.value {
                SettingValue::Bool(current_value) => {
                    let new_value = !current_value;

                    // Update config in SharedData
                    let mut data = shared_data.write().await;

                    // Match by setting key to update the correct field
                    match setting.key.as_str() {
                        "Use Unicode" => {
                            data.config.display.use_unicode = new_value;
                            data.config.display.box_chars = crate::formatting::BoxChars::from_use_unicode(new_value);
                            data.set_status(format!("Use Unicode: {}", if new_value { "enabled" } else { "disabled" }));
                        }
                        "Western Teams First" => {
                            data.config.display_standings_western_first = new_value;
                            data.set_status(format!("Western Teams First: {}", if new_value { "enabled" } else { "disabled" }));
                        }
                        _ => {
                            data.set_error(format!("Unknown bool setting: {}", setting.key));
                        }
                    }

                    tracing::info!("Toggled {}: {} -> {}", setting.key, current_value, new_value);

                    // Save config to disk
                    if let Err(e) = save_config_to_disk(&data.config) {
                        data.set_error(e);
                    }
                    drop(data); // Release write lock
                    true
                }
                SettingValue::String(current_value) => {
                    // Enter editing mode
                    state.editing = Some((setting.key.clone(), current_value.clone()));
                    shared_data.write().await.set_status("Editing... (Enter to save, Esc to cancel)".to_string());
                    tracing::info!("Started editing string setting: {}", setting.key);
                    true
                }
                SettingValue::Int(current_value) => {
                    // Enter editing mode with int as string
                    state.editing = Some((setting.key.clone(), current_value.to_string()));
                    shared_data.write().await.set_status("Editing... (Type digits, Up/Down arrows, Enter to save, Esc to cancel)".to_string());
                    tracing::info!("Started editing int setting: {}", setting.key);
                    true
                }
                SettingValue::List { options, current_index } => {
                    // Open list modal
                    state.list_modal = Some((setting.key.clone(), options.clone(), *current_index));
                    shared_data.write().await.set_status("Select option (Up/Down, Enter to confirm, Esc to cancel)".to_string());
                    tracing::info!("Opened list modal for: {}", setting.key);
                    true
                }
                SettingValue::Color(_current_color) => {
                    // Open color picker modal
                    state.color_modal = Some(setting.key.clone());
                    shared_data.write().await.set_status("Select color (Arrow keys, Enter to confirm, Esc to cancel)".to_string());
                    tracing::info!("Opened color picker modal for: {}", setting.key);
                    true
                }
            }
        }
        _ => false,
    }
}

/// Handle key events while in editing mode
async fn handle_editing_mode(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    setting_name: String,
    mut edit_buffer: String,
) -> bool {
    // Check if this is an int setting (Refresh Interval)
    let is_int_setting = setting_name == "Refresh Interval (seconds)";

    match key.code {
        KeyCode::Enter => {
            // Save the edited value
            let mut data = shared_data.write().await;

            match setting_name.as_str() {
                "Log File" => {
                    data.config.log_file = edit_buffer.clone();
                    data.set_status(format!("Log File updated (restart required): {}", edit_buffer));
                }
                "Time Format" => {
                    data.config.time_format = edit_buffer.clone();
                    data.set_status(format!("Time Format updated to: {}", edit_buffer));
                }
                "Refresh Interval (seconds)" => {
                    // Parse as u32
                    match edit_buffer.parse::<u32>() {
                        Ok(value) => {
                            data.config.refresh_interval = value;
                            data.set_status(format!("Refresh Interval updated to: {} seconds", value));
                        }
                        Err(_) => {
                            data.set_error("Invalid number".to_string());
                            state.editing = None;
                            return true;
                        }
                    }
                }
                _ => {
                    data.set_error(format!("Unknown setting: {}", setting_name));
                }
            }

            tracing::info!("Saved {} = {}", setting_name, edit_buffer);

            // Save config to disk
            if let Err(e) = save_config_to_disk(&data.config) {
                data.set_error(e);
            }
            drop(data); // Release write lock
            state.editing = None;
            true
        }
        KeyCode::Esc => {
            // Cancel editing
            state.editing = None;
            shared_data.write().await.set_status("Editing cancelled".to_string());
            tracing::info!("Cancelled editing {}", setting_name);
            true
        }
        KeyCode::Up if is_int_setting => {
            // Increment by 1
            if let Ok(mut value) = edit_buffer.parse::<u32>() {
                value = value.saturating_add(1);
                edit_buffer = value.to_string();
                state.editing = Some((setting_name, edit_buffer));
            }
            true
        }
        KeyCode::Down if is_int_setting => {
            // Decrement by 1
            if let Ok(mut value) = edit_buffer.parse::<u32>() {
                value = value.saturating_sub(1);
                edit_buffer = value.to_string();
                state.editing = Some((setting_name, edit_buffer));
            }
            true
        }
        KeyCode::Backspace => {
            // Delete last character
            edit_buffer.pop();
            state.editing = Some((setting_name, edit_buffer));
            true
        }
        KeyCode::Char(c) => {
            // For int settings, only allow digits
            if is_int_setting && !c.is_ascii_digit() {
                // Ignore non-digit characters for int settings
                return true;
            }
            // Add character to buffer
            edit_buffer.push(c);
            state.editing = Some((setting_name, edit_buffer));
            true
        }
        _ => true, // Consume all other keys in editing mode
    }
}

/// Handle key events while list modal is open
async fn handle_list_modal(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    setting_name: String,
    options: Vec<String>,
    mut selected_index: usize,
) -> bool {
    match key.code {
        KeyCode::Up => {
            // Move up in the list
            if selected_index > 0 {
                selected_index -= 1;
                state.list_modal = Some((setting_name, options, selected_index));
            }
            true
        }
        KeyCode::Down => {
            // Move down in the list
            if selected_index + 1 < options.len() {
                selected_index += 1;
                state.list_modal = Some((setting_name, options, selected_index));
            }
            true
        }
        KeyCode::Enter => {
            // Save the selected option
            let selected_value = &options[selected_index];
            let mut data = shared_data.write().await;

            match setting_name.as_str() {
                "Log Level" => {
                    data.config.log_level = selected_value.clone();
                    data.set_status(format!("Log Level set to: {} (restart required)", selected_value));
                }
                _ => {
                    data.set_error(format!("Unknown list setting: {}", setting_name));
                }
            }

            tracing::info!("Selected {} = {}", setting_name, selected_value);

            // Save config to disk
            if let Err(e) = save_config_to_disk(&data.config) {
                data.set_error(e);
            }
            drop(data); // Release write lock
            state.list_modal = None;
            true
        }
        KeyCode::Esc => {
            // Cancel modal
            state.list_modal = None;
            shared_data.write().await.set_status("Selection cancelled".to_string());
            tracing::info!("Cancelled list modal for {}", setting_name);
            true
        }
        _ => true, // Consume all other keys in modal mode
    }
}

/// Handle key events while color picker modal is open
async fn handle_color_modal(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    setting_name: String,
) -> bool {
    use super::view::COLORS;

    match key.code {
        KeyCode::Up => {
            // Move up one row (subtract 4)
            if state.selected_color_index >= 4 {
                state.selected_color_index -= 4;
            }
            true
        }
        KeyCode::Down => {
            // Move down one row (add 4)
            if state.selected_color_index + 4 < 24 {
                state.selected_color_index += 4;
            }
            true
        }
        KeyCode::Left => {
            // Move left one column
            if state.selected_color_index % 4 != 0 {
                state.selected_color_index -= 1;
            }
            true
        }
        KeyCode::Right => {
            // Move right one column
            if state.selected_color_index % 4 != 3 {
                state.selected_color_index += 1;
            }
            true
        }
        KeyCode::Enter => {
            // Save the selected color
            let (selected_color, selected_name) = COLORS[state.selected_color_index];
            let mut data = shared_data.write().await;

            match setting_name.as_str() {
                "Selection FG" => {
                    data.config.display.selection_fg = selected_color;
                    data.set_status(format!("Selection FG set to: {}", selected_name));
                }
                "Division Header FG" => {
                    data.config.display.division_header_fg = selected_color;
                    data.set_status(format!("Division Header FG set to: {}", selected_name));
                }
                "Error FG" => {
                    data.config.display.error_fg = selected_color;
                    data.set_status(format!("Error FG set to: {}", selected_name));
                }
                _ => {
                    data.set_error(format!("Unknown color setting: {}", setting_name));
                }
            }

            tracing::info!("Selected {} = {}", setting_name, selected_name);

            // Save config to disk
            if let Err(e) = save_config_to_disk(&data.config) {
                data.set_error(e);
            }
            drop(data); // Release write lock
            state.color_modal = None;
            true
        }
        KeyCode::Esc => {
            // Cancel modal
            state.color_modal = None;
            shared_data.write().await.set_status("Color selection cancelled".to_string());
            tracing::info!("Cancelled color modal for {}", setting_name);
            true
        }
        _ => true, // Consume all other keys in modal mode
    }
}

/// Helper function to save config to disk
/// Returns error if write fails
fn save_config_to_disk(config: &config::Config) -> Result<(), String> {
    config::write(config).map_err(|e| {
        let error_msg = format!("Failed to save config: {}", e);
        tracing::error!("{}", error_msg);
        error_msg
    })
}

// ============================================================================
// OLD COLOR PICKER HANDLER CODE (KEPT FOR REFERENCE)
// ============================================================================
/*

pub async fn handle_key_color_picker(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle) -> bool {
    // Clear any existing status message when navigating
    if matches!(key.code, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) {
        state.status_message = None;
    }

    // Color picker is active when in subtab mode
    match key.code {
        KeyCode::Up => {
            // Move up one row (subtract 4), or signal to exit if at top row
            if state.selected_color_index >= 4 {
                state.selected_color_index -= 4;
                true
            } else {
                // At top row, let default handler exit subtab mode
                false
            }
        }
        KeyCode::Down => {
            // Move down one row (add 4)
            if state.selected_color_index + 4 < 24 {
                state.selected_color_index += 4;
            }
            true
        }
        KeyCode::Left => {
            // Move left one column
            if state.selected_color_index % 4 != 0 {
                state.selected_color_index -= 1;
            }
            true
        }
        KeyCode::Right => {
            // Move right one column
            if state.selected_color_index % 4 != 3 {
                state.selected_color_index += 1;
            }
            true
        }
        KeyCode::Enter => {
            // Get the selected color
            let (selected_color, selected_name) = COLORS[state.selected_color_index];

            // Update the theme in SharedData
            let mut data = shared_data.write().await;
            data.config.display.selection_fg = selected_color;

            // Set status message
            state.status_message = Some(format!("✓ Theme color changed to {}", selected_name));

            tracing::info!("User selected color: {} - theme updated", selected_name);
            true
        }
        _ => false,
    }
}
*/
