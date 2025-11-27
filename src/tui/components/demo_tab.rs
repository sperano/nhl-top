//! Demo tab component showcasing the document system
//!
//! This tab demonstrates the document system capabilities including:
//! - Document elements (headings, text, links, separators)
//! - Viewport-based scrolling
//! - Tab/Shift-Tab focus navigation through focusable elements
//! - Autoscrolling to keep focused elements visible
//! - Embedded tables (league standings) rendered at natural height

use std::sync::Arc;

use nhl_api::Standing;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::config::DisplayConfig;
use crate::tui::action::ComponentMessageTrait;
use crate::tui::component::{Component, Effect, Element, ElementWidget};
use crate::tui::components::create_standings_table_with_selection;
use crate::tui::components::TableWidget;
use crate::tui::document::{
    Document, DocumentBuilder, DocumentElement, DocumentView, FocusContext, LinkTarget,
};
use crate::tui::helpers::StandingsSorting;
use crate::tui::{Alignment, CellValue, ColumnDef};

/// Demo player data for the sample player table
#[derive(Clone)]
struct DemoPlayer {
    name: String,
    player_id: i64,
    team: String,
    games: u32,
    goals: u32,
    assists: u32,
}

impl DemoPlayer {
    fn new(name: &str, player_id: i64, team: &str, games: u32, goals: u32, assists: u32) -> Self {
        Self {
            name: name.to_string(),
            player_id,
            team: team.to_string(),
            games,
            goals,
            assists,
        }
    }

    fn points(&self) -> u32 {
        self.goals + self.assists
    }
}

/// Create sample forward player data for demo (top scorers)
fn create_demo_forwards() -> Vec<DemoPlayer> {
    vec![
        DemoPlayer::new("Nathan MacKinnon", 8477492, "COL", 82, 51, 89),
        DemoPlayer::new("Nikita Kucherov", 8476453, "TBL", 81, 44, 100),
        DemoPlayer::new("Connor McDavid", 8478402, "EDM", 76, 32, 100),
        DemoPlayer::new("Leon Draisaitl", 8477934, "EDM", 81, 41, 65),
        DemoPlayer::new("Auston Matthews", 8479318, "TOR", 69, 69, 38),
    ]
}

/// Create sample defenseman player data for demo (top scoring D)
fn create_demo_defensemen() -> Vec<DemoPlayer> {
    vec![
        DemoPlayer::new("Quinn Hughes", 8480800, "VAN", 82, 17, 75),
        DemoPlayer::new("Cale Makar", 8480069, "COL", 77, 21, 69),
        DemoPlayer::new("Roman Josi", 8474600, "NSH", 82, 23, 62),
        DemoPlayer::new("Evan Bouchard", 8480803, "EDM", 81, 18, 64),
        DemoPlayer::new("Adam Fox", 8479323, "NYR", 74, 17, 56),
    ]
}

/// Create a player stats table widget from given players
fn create_player_table(players: Vec<DemoPlayer>, focused_row: Option<usize>) -> TableWidget {
    let columns: Vec<ColumnDef<DemoPlayer>> = vec![
        ColumnDef::new("Player", 18, Alignment::Left, |p: &DemoPlayer| {
            CellValue::PlayerLink {
                display: p.name.clone(),
                player_id: p.player_id,
            }
        }),
        // Team as Text (not TeamLink) so only Player column is focusable per row
        ColumnDef::new("Team", 5, Alignment::Center, |p: &DemoPlayer| {
            CellValue::Text(p.team.clone())
        }),
        ColumnDef::new("GP", 3, Alignment::Right, |p: &DemoPlayer| {
            CellValue::Text(p.games.to_string())
        }),
        ColumnDef::new("G", 3, Alignment::Right, |p: &DemoPlayer| {
            CellValue::Text(p.goals.to_string())
        }),
        ColumnDef::new("A", 3, Alignment::Right, |p: &DemoPlayer| {
            CellValue::Text(p.assists.to_string())
        }),
        ColumnDef::new("PTS", 4, Alignment::Right, |p: &DemoPlayer| {
            CellValue::Text(p.points().to_string())
        }),
    ];

    TableWidget::from_data(&columns, players).with_focused_row(focused_row)
}

/// Props for the Demo tab
#[derive(Clone)]
pub struct DemoTabProps {
    /// Whether the tab content is focused
    pub content_focused: bool,
    /// Standings data for demonstrating embedded tables
    pub standings: Arc<Option<Vec<Standing>>>,
}


/// Messages that can be sent to the Demo tab
#[derive(Clone, Debug)]
pub enum DemoTabMessage {
    /// Document navigation (Phase 7: Delegated to DocumentNavMsg)
    DocNav(crate::tui::document_nav::DocumentNavMsg),
    /// Update viewport height
    UpdateViewportHeight(u16),
}

impl ComponentMessageTrait for DemoTabMessage {
    fn apply(&self, state: &mut dyn std::any::Any) -> Effect {
        if let Some(demo_state) = state.downcast_mut::<crate::tui::document_nav::DocumentNavState>() {
            let mut component = DemoTab;
            let msg = self.clone();
            component.update(msg, demo_state)
        } else {
            Effect::None
        }
    }

    fn clone_box(&self) -> Box<dyn ComponentMessageTrait> {
        Box::new(self.clone())
    }
}

/// Demo tab component
pub struct DemoTab;

impl Component for DemoTab {
    type Props = DemoTabProps;
    type State = crate::tui::document_nav::DocumentNavState;
    type Message = DemoTabMessage;

    fn init(_props: &Self::Props) -> Self::State {
        crate::tui::document_nav::DocumentNavState::default()
    }

    fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
        match msg {
            DemoTabMessage::DocNav(nav_msg) => {
                crate::tui::document_nav::handle_message(state, &nav_msg)
            }
            DemoTabMessage::UpdateViewportHeight(height) => {
                state.viewport_height = height;
                Effect::None
            }
        }
    }

    fn view(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::Widget(Box::new(DemoTabWidget {
            content_focused: props.content_focused,
            focus_index: state.focus_index,
            scroll_offset: state.scroll_offset,
            standings: props.standings.clone(),
        }))
    }
}

/// Widget for rendering the Demo tab
struct DemoTabWidget {
    content_focused: bool,
    focus_index: Option<usize>,
    scroll_offset: u16,
    standings: Arc<Option<Vec<Standing>>>,
}

impl ElementWidget for DemoTabWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Create document view with state from AppState
        let standings = (*self.standings).clone();
        let doc = Arc::new(DemoDocument::new(standings));
        let mut view = DocumentView::new(doc, area.height);

        // Apply focus state from AppState
        if let Some(idx) = self.focus_index {
            view.focus_by_index(idx);
        }

        // Apply scroll offset from AppState
        view.set_scroll_offset(self.scroll_offset);

        view.render(area, buf, config);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(DemoTabWidget {
            content_focused: self.content_focused,
            focus_index: self.focus_index,
            scroll_offset: self.scroll_offset,
            standings: self.standings.clone(),
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Fills available space
    }
}

/// Demo document showcasing all document element types
pub struct DemoDocument {
    standings: Option<Vec<Standing>>,
}

impl DemoDocument {
    pub fn new(standings: Option<Vec<Standing>>) -> Self {
        Self { standings }
    }

    /// Build standings table using the shared StandingsTable component
    fn build_standings_section(
        &self,
        builder: DocumentBuilder,
        focus: &FocusContext,
    ) -> DocumentBuilder {
        let builder = builder
            .heading(2, "League Standings")
            .spacer(1)
            .text("This demonstrates the shared standings table embedded in a document:");

        const TABLE_NAME: &str = "standings";

        match &self.standings {
            Some(standings) if !standings.is_empty() => {
                // Sort by points (highest first) using the shared sorting trait
                let mut sorted = standings.clone();
                sorted.sort_by_points_desc();

                // Use the shared standings table component with focus state
                let table = create_standings_table_with_selection(
                    sorted,
                    None,
                    focus.focused_table_row(TABLE_NAME),
                );

                builder.spacer(1).table(TABLE_NAME, table)
            }
            _ => builder.text("(No standings data loaded - try refreshing)"),
        }
    }

    /// Build the player stats table section with two tables side by side
    fn build_player_section(
        &self,
        builder: DocumentBuilder,
        focus: &FocusContext,
    ) -> DocumentBuilder {
        const FORWARDS_TABLE: &str = "forwards";
        const DEFENSEMEN_TABLE: &str = "defensemen";

        let forwards_table = create_player_table(
            create_demo_forwards(),
            focus.focused_table_row(FORWARDS_TABLE),
        );
        let defensemen_table = create_player_table(
            create_demo_defensemen(),
            focus.focused_table_row(DEFENSEMEN_TABLE),
        );

        builder
            .heading(2, "Top Scorers (2023-24)")
            .spacer(1)
            .text("These tables demonstrate side-by-side layout with focusable player links:")
            .spacer(1)
            .row(vec![
                DocumentElement::table(FORWARDS_TABLE, forwards_table),
                DocumentElement::table(DEFENSEMEN_TABLE, defensemen_table),
            ])
    }
}

impl Document for DemoDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        let builder = DocumentBuilder::new()
            .heading(1, "Document System Demo")
            .spacer(1)
            .text("This tab demonstrates the new document system for the NHL TUI.")
            .text("Press Tab/Shift-Tab to navigate between focusable elements.")
            .spacer(1)
            .separator()
            .spacer(1);

        // Add standings section first (showcases natural height rendering)
        let builder = self.build_standings_section(builder, focus);

        // Then add the rest of the demo content
        let builder = builder
            .spacer(1)
            .separator()
            .spacer(1)
            .heading(2, "Features")
            .text("- Viewport-based scrolling for unlimited content height")
            .text("- Tab/Shift-Tab navigation cycles through focusable elements")
            .text("- Autoscrolling keeps the focused element visible")
            .text("- Smart padding positions elements comfortably in view")
            .text("- Focus wrapping scrolls to top/bottom automatically")
            .spacer(1)
            .heading(2, "Example Links")
            .text("These links demonstrate focusable elements:")
            .spacer(1)
            .link_with_focus("link_bos", "Boston Bruins", LinkTarget::Action("team:BOS".to_string()), focus)
            .spacer(1)
            .link_with_focus("link_tor", "Toronto Maple Leafs", LinkTarget::Action("team:TOR".to_string()), focus)
            .spacer(1)
            .link_with_focus("link_nyr", "New York Rangers", LinkTarget::Action("team:NYR".to_string()), focus)
            .spacer(1)
            .link_with_focus("link_mtl", "Montreal Canadiens", LinkTarget::Action("team:MTL".to_string()), focus)
            .spacer(1)
            .separator()
            .spacer(1)
            .heading(2, "Implementation Notes")
            .text("The document system consists of several modules:")
            .spacer(1)
            .text("- viewport.rs: Manages scroll position and visible range")
            .text("- focus.rs: Tracks focusable elements and navigation")
            .text("- elements.rs: Document element types (text, headings, links)")
            .text("- builder.rs: Fluent API for building documents")
            .text("- mod.rs: Document trait and DocumentView container")
            .spacer(1)
            .text("Each document implements the Document trait to define its")
            .text("content structure. DocumentView manages the viewport and")
            .text("focus state for rendering and interaction.");

        // Add player stats table at the bottom
        let builder = self.build_player_section(builder, focus);

        builder
            .spacer(1)
            .text("End of demo document.")
            .build()
    }

    fn title(&self) -> String {
        "Document System Demo".to_string()
    }

    fn id(&self) -> String {
        "demo".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::assert_buffer;

    #[test]
    fn test_demo_document_builds() {
        let doc = DemoDocument::new(None);
        let elements = doc.build(&FocusContext::default());

        // Should have multiple elements
        assert!(elements.len() > 10);
    }

    #[test]
    fn test_demo_document_height() {
        let doc = DemoDocument::new(None);
        let height = doc.calculate_height();

        // Should have significant height (all the content)
        assert!(height > 30);
    }

    #[test]
    fn test_demo_tab_renders() {
        let props = DemoTabProps {
            content_focused: false,
            standings: Arc::new(None),
        };
        let state = crate::tui::document_nav::DocumentNavState::default();
        let demo_tab = DemoTab;

        let element = demo_tab.view(&props, &state);

        // Should return a widget element
        assert!(matches!(element, Element::Widget(_)));
    }

    #[test]
    fn test_demo_document_focusable_count_no_standings() {
        let doc = DemoDocument::new(None);
        let doc_arc = Arc::new(doc);
        let view = DocumentView::new(doc_arc, 20);

        // Should have 14 focusable elements:
        // - 4 example links (BOS, TOR, NYR, MTL)
        // - 10 player table cells (5 forwards + 5 defensemen, 1 link column each)
        assert_eq!(view.focus_manager().len(), 14);
    }

    #[test]
    fn test_demo_tab_widget_render() {
        let widget = DemoTabWidget {
            content_focused: true,
            focus_index: None,
            scroll_offset: 0,
            standings: Arc::new(None),
        };

        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 5));
        let config = DisplayConfig::default();

        widget.render(buf.area, &mut buf, &config);

        // Should render the heading and first lines of content
        assert_buffer(&buf, &[
            "Document System Demo",
            "════════════════════",
            "",
            "This tab demonstrates the new document system for the NHL TU",
            "Press Tab/Shift-Tab to navigate between focusable elements.",
        ]);
    }

    #[test]
    fn test_demo_tab_focus_navigation() {
        let doc = Arc::new(DemoDocument::new(None));
        let mut view = DocumentView::new(doc, 10);

        // Initially no focus
        assert!(view.focus_manager().current_index().is_none());

        // First Tab focuses first link
        view.focus_next();
        assert_eq!(view.focus_manager().current_index(), Some(0));

        // Second Tab focuses second link
        view.focus_next();
        assert_eq!(view.focus_manager().current_index(), Some(1));

        // Shift-Tab goes back
        view.focus_prev();
        assert_eq!(view.focus_manager().current_index(), Some(0));
    }
}
