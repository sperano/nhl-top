use ratatui::{
    buffer::Buffer,
    layout::{Constraint as RatatuiConstraint, Direction, Layout as RatatuiLayout, Rect},
};

use super::component::{Constraint, ContainerLayout, Element};
use crate::config::DisplayConfig;

/// Renders virtual element tree to ratatui buffer
///
/// The Renderer takes a virtual Element tree produced by components
/// and renders it to the terminal using ratatui.
///
/// Future optimizations:
/// - Diffing: Compare previous tree with current tree to minimize redraws
/// - Memoization: Cache rendered widgets for identical props
pub struct Renderer {
    /// Previous frame's tree (for future diffing optimization)
    prev_tree: Option<Element>,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self { prev_tree: None }
    }

    /// Render an element tree to the given area in the buffer
    ///
    /// This is the main entry point for rendering.
    pub fn render(&mut self, element: Element, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        self.render_element(&element, area, buf, config);
        self.prev_tree = Some(element);
    }

    /// Render a single element to the given area
    fn render_element(&self, element: &Element, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        match element {
            Element::Widget(widget) => {
                // Directly render widget to buffer
                widget.render(area, buf, config);
            }

            Element::Container { children, layout } => {
                // Calculate layout and render children
                let chunks = self.calculate_layout(layout, area);

                // Render each child in its corresponding chunk
                for (child, chunk) in children.iter().zip(chunks.iter()) {
                    self.render_element(child, *chunk, buf, config);
                }
            }

            Element::Fragment(children) => {
                // Fragments render all children in the same area
                // This is useful for conditional rendering where only one child should be visible
                for child in children {
                    self.render_element(child, area, buf, config);
                }
            }

            Element::Overlay { base, overlay } => {
                // First render the base content
                self.render_element(base, area, buf, config);
                // Then render the overlay on top
                self.render_element(overlay, area, buf, config);
            }

            Element::Component(_) => {
                // Components should already be resolved to concrete elements
                // before reaching the renderer. If we encounter one here,
                // it's a bug in the runtime.
                panic!("Unresolved component in render tree - components should be resolved to elements before rendering");
            }

            Element::None => {
                // Render nothing
            }
        }
    }

    /// Calculate layout constraints and split the area
    fn calculate_layout(&self, layout: &ContainerLayout, area: Rect) -> Vec<Rect> {
        match layout {
            ContainerLayout::Vertical(constraints) => {
                let ratatui_constraints = constraints
                    .iter()
                    .map(|c| self.convert_constraint(*c))
                    .collect::<Vec<_>>();

                RatatuiLayout::default()
                    .direction(Direction::Vertical)
                    .constraints(ratatui_constraints)
                    .split(area)
                    .to_vec()
            }

            ContainerLayout::Horizontal(constraints) => {
                let ratatui_constraints = constraints
                    .iter()
                    .map(|c| self.convert_constraint(*c))
                    .collect::<Vec<_>>();

                RatatuiLayout::default()
                    .direction(Direction::Horizontal)
                    .constraints(ratatui_constraints)
                    .split(area)
                    .to_vec()
            }
        }
    }

    /// Convert our Constraint type to ratatui's Constraint
    fn convert_constraint(&self, constraint: Constraint) -> RatatuiConstraint {
        match constraint {
            Constraint::Length(n) => RatatuiConstraint::Length(n),
            Constraint::Min(n) => RatatuiConstraint::Min(n),
            Constraint::Max(n) => RatatuiConstraint::Max(n),
            Constraint::Percentage(n) => RatatuiConstraint::Percentage(n),
            Constraint::Ratio(a, b) => RatatuiConstraint::Ratio(a, b),
        }
    }

    /// Get the previous tree (for future diffing)
    #[allow(dead_code)]
    pub fn prev_tree(&self) -> Option<&Element> {
        self.prev_tree.as_ref()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        buffer::Buffer,
        text::Text,
        widgets::{Paragraph, Widget},
    };

    /// Test widget that renders "TEST" in the center
    #[derive(Clone)]
    struct TestWidget {
        text: String,
    }

    impl super::super::component::RenderableWidget for TestWidget {
        fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
            let text = Text::from(self.text.clone());
            Paragraph::new(text).render(area, buf);
        }

        fn clone_box(&self) -> Box<dyn super::super::component::RenderableWidget> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_render_none() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 10));
        let config = DisplayConfig::default();

        renderer.render(Element::None, buffer.area, &mut buffer, &config);

        // Buffer should remain empty
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(buffer[(x, y)].symbol(), " ");
            }
        }
    }

    #[test]
    fn test_render_widget() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 3));
        let config = DisplayConfig::default();

        let widget = Box::new(TestWidget {
            text: "Hello".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;
        let element = Element::Widget(widget);

        renderer.render(element, buffer.area, &mut buffer, &config);

        // Should render "Hello" in the first row
        let line = (0..10)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(line.contains("Hello"));
    }

    #[test]
    fn test_render_container_vertical() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 6));
        let config = DisplayConfig::default();

        let top_widget = Box::new(TestWidget {
            text: "TOP".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let bottom_widget = Box::new(TestWidget {
            text: "BOTTOM".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let element = Element::Container {
            layout: ContainerLayout::Vertical(vec![Constraint::Length(3), Constraint::Length(3)]),
            children: vec![Element::Widget(top_widget), Element::Widget(bottom_widget)],
        };

        renderer.render(element, buffer.area, &mut buffer, &config);

        // Top should be in first 3 rows, bottom in last 3 rows
        let top_line = (0..10)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(top_line.contains("TOP"));

        let bottom_line = (0..10)
            .map(|x| buffer[(x, 3)].symbol())
            .collect::<String>();
        assert!(bottom_line.contains("BOTTOM"));
    }

    #[test]
    fn test_render_container_horizontal() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 3));
        let config = DisplayConfig::default();

        let left_widget = Box::new(TestWidget {
            text: "LEFT".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let right_widget = Box::new(TestWidget {
            text: "RIGHT".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let element = Element::Container {
            layout: ContainerLayout::Horizontal(vec![
                Constraint::Length(10),
                Constraint::Length(10),
            ]),
            children: vec![Element::Widget(left_widget), Element::Widget(right_widget)],
        };

        renderer.render(element, buffer.area, &mut buffer, &config);

        // Left should be in first 10 columns, right in last 10 columns
        let left_part = (0..10)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(left_part.contains("LEFT"));

        let right_part = (10..20)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(right_part.contains("RIGHT"));
    }

    #[test]
    fn test_render_fragment() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 3));
        let config = DisplayConfig::default();

        // Fragment renders multiple children in same area
        // The second child should overwrite the first
        let widget1 = Box::new(TestWidget {
            text: "First".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let widget2 = Box::new(TestWidget {
            text: "Second".to_string(),
        }) as Box<dyn super::super::component::RenderableWidget>;

        let element = Element::Fragment(vec![Element::Widget(widget1), Element::Widget(widget2)]);

        renderer.render(element, buffer.area, &mut buffer, &config);

        // Should render "Second" (overwrites "First")
        let line = (0..10)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(line.contains("Second"));
    }

    #[test]
    fn test_constraint_conversion() {
        let renderer = Renderer::new();

        assert_eq!(
            renderer.convert_constraint(Constraint::Length(10)),
            RatatuiConstraint::Length(10)
        );
        assert_eq!(
            renderer.convert_constraint(Constraint::Min(5)),
            RatatuiConstraint::Min(5)
        );
        assert_eq!(
            renderer.convert_constraint(Constraint::Max(20)),
            RatatuiConstraint::Max(20)
        );
        assert_eq!(
            renderer.convert_constraint(Constraint::Percentage(50)),
            RatatuiConstraint::Percentage(50)
        );
        assert_eq!(
            renderer.convert_constraint(Constraint::Ratio(1, 3)),
            RatatuiConstraint::Ratio(1, 3)
        );
    }

    #[test]
    fn test_prev_tree_stored() {
        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 3));
        let config = DisplayConfig::default();

        assert!(renderer.prev_tree().is_none());

        let element = Element::None;
        renderer.render(element.clone(), buffer.area, &mut buffer, &config);

        assert!(renderer.prev_tree().is_some());
    }

    #[test]
    #[should_panic(expected = "Unresolved component")]
    fn test_unresolved_component_panics() {
        // Components should never reach the renderer
        // This test verifies that we panic if they do

        // We need a dummy component wrapper for this test
        struct DummyWrapper;

        impl super::super::component::ComponentWrapper for DummyWrapper {
            fn view_any(&self) -> Element {
                Element::None
            }

            fn clone_box(&self) -> Box<dyn super::super::component::ComponentWrapper> {
                Box::new(DummyWrapper)
            }
        }

        let mut renderer = Renderer::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 3));
        let config = DisplayConfig::default();

        let element = Element::Component(Box::new(DummyWrapper));

        // This should panic
        renderer.render(element, buffer.area, &mut buffer, &config);
    }
}
