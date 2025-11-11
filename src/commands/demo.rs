/// Widget system demonstration mode
///
/// This module provides an interactive demo of the widget-based architecture
/// implemented in Phase 1 & 2 of the widget refactor. It showcases:
/// - FocusableTable with clickable rows
/// - BreadcrumbWidget with navigation
/// - Link widgets for navigation
/// - List widget with items
/// - WidgetTree for focus management
///
/// Available only in debug builds for testing and development.

use crate::config::DisplayConfig;
use crate::tui::widgets::{
    FocusableTable, ColumnDef, Alignment,
    BreadcrumbWidget,
    RenderableWidget,
    Link, List,
    Container,
};
use crate::tui::widgets::focus::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    Terminal,
};
use std::io;
use anyhow::Result;

/// Demo data structure for the table
#[derive(Clone)]
struct DemoTeam {
    name: String,
    abbrev: String,
    points: i32,
    wins: i32,
    losses: i32,
}

/// Demo state
struct DemoState {
    container: Container,
    config: DisplayConfig,
    breadcrumb: BreadcrumbWidget,
}

impl DemoState {
    fn new() -> Self {
        let config = DisplayConfig::default();

        // Create breadcrumb
        let breadcrumb = BreadcrumbWidget::from_trail(&[
            "Demo".to_string(),
            "Widget Showcase".to_string(),
        ]);

        // Create a table with demo data
        let teams = vec![
            DemoTeam {
                name: "Toronto Maple Leafs".to_string(),
                abbrev: "TOR".to_string(),
                points: 100,
                wins: 45,
                losses: 20,
            },
            DemoTeam {
                name: "Boston Bruins".to_string(),
                abbrev: "BOS".to_string(),
                points: 98,
                wins: 44,
                losses: 21,
            },
            DemoTeam {
                name: "Montreal Canadiens".to_string(),
                abbrev: "MTL".to_string(),
                points: 85,
                wins: 38,
                losses: 27,
            },
            DemoTeam {
                name: "Tampa Bay Lightning".to_string(),
                abbrev: "TBL".to_string(),
                points: 95,
                wins: 42,
                losses: 23,
            },
            DemoTeam {
                name: "Florida Panthers".to_string(),
                abbrev: "FLA".to_string(),
                points: 92,
                wins: 41,
                losses: 24,
            },
        ];

        let columns = vec![
            ColumnDef::new(
                "Team",
                30,
                |t: &DemoTeam| t.name.clone(),
                Alignment::Left,
                true,
            ),
            ColumnDef::new(
                "Abbrev",
                8,
                |t: &DemoTeam| t.abbrev.clone(),
                Alignment::Center,
                false,
            ),
            ColumnDef::new(
                "Points",
                8,
                |t: &DemoTeam| t.points.to_string(),
                Alignment::Right,
                false,
            ),
            ColumnDef::new(
                "Wins",
                6,
                |t: &DemoTeam| t.wins.to_string(),
                Alignment::Right,
                false,
            ),
            ColumnDef::new(
                "Losses",
                8,
                |t: &DemoTeam| t.losses.to_string(),
                Alignment::Right,
                false,
            ),
        ];

        let table = FocusableTable::new(teams.clone())
            .with_header("NHL Teams Demo")
            .with_columns(columns)
            .with_on_activate(|team: &DemoTeam| {
                NavigationAction::NavigateToTeam(team.abbrev.clone())
            });

        // Create action list with links
        let mut action_list = List::new();
        action_list.add_item(Box::new(
            Link::new("Refresh Data").with_action(|| NavigationAction::PopPanel)
        ));
        action_list.add_item(Box::new(
            Link::new("Exit Demo").with_action(|| NavigationAction::PopPanel)
        ));

        // Container automatically handles all inter-widget navigation!
        let mut container = Container::with_children(vec![
            Box::new(table),
            Box::new(action_list),
        ]).with_wrap(true);

        // Focus the container
        container.set_focused(true);

        Self {
            container,
            config,
            breadcrumb,
        }
    }

    fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let breadcrumb_height = self.breadcrumb.preferred_height().unwrap_or(2);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(breadcrumb_height),
                Constraint::Min(0),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(area);

        // Render widgets
        self.breadcrumb.render(chunks[0], buf, &self.config);

        // Render children from container
        if let Some(table) = self.container.child(0) {
            table.render(chunks[1], buf, &self.config);
        }
        if let Some(action_list) = self.container.child(1) {
            action_list.render(chunks[2], buf, &self.config);
        }

        // Render help text at bottom
        render_help(chunks[3], buf);
    }

    fn handle_event(&mut self, event: KeyEvent) -> bool {
        // Handle quit
        if event.code == KeyCode::Char('q')
            || event.code == KeyCode::Esc
            || (event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL))
        {
            return false;
        }

        // Delegate to container - it handles all navigation automatically!
        let result = self.container.handle_input(event);

        // Check for navigation actions (e.g., from Link widgets)
        if let InputResult::Navigate(action) = result {
            match action {
                NavigationAction::PopPanel => {
                    // Exit Demo was selected
                    return false;
                }
                NavigationAction::NavigateToTeam(abbrev) => {
                    // In a real app, this would navigate to the team detail page
                    // For demo purposes, we just update the breadcrumb
                    // (but breadcrumb.push doesn't exist, so we ignore it)
                    eprintln!("Would navigate to team: {}", abbrev);
                }
                _ => {
                    // Other navigation actions would be handled here
                }
            }
        }

        true
    }
}

fn render_help(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::Text;

    let help_text = "Arrow Keys: Navigate | Enter: Select | Tab/Shift+Tab: Cycle Focus | Q/ESC: Quit";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help ");

    let paragraph = Paragraph::new(Text::from(help_text))
        .block(block)
        .style(Style::default().fg(Color::Gray));

    ratatui::widgets::Widget::render(paragraph, area, buf);
}

/// Run the widget demonstration
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = DemoState::new();
    let mut should_quit = false;

    // Main event loop
    while !should_quit {
        terminal.draw(|f| {
            let mut buf = f.buffer_mut().clone();
            state.render(f.area(), &mut buf);
            *f.buffer_mut() = buf;
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                should_quit = !state.handle_event(key);
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Demo completed!");
    Ok(())
}

// Navigation tests are now handled by Container's tests.
// The Container widget automatically handles all the complex navigation logic
// that DemoContainer used to implement manually.
