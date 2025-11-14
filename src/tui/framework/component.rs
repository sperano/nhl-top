use ratatui::{buffer::Buffer, layout::Rect};
use std::future::Future;
use std::pin::Pin;

use super::action::Action;
use crate::config::DisplayConfig;

/// Core component trait - like React.Component
///
/// Components are the building blocks of the UI. Each component:
/// - Has Props (input data, like React props)
/// - Has State (internal state, like useState)
/// - Has Messages (internal events, like setState callbacks)
/// - Renders to an Element tree (virtual DOM)
pub trait Component: Send {
    /// Props type for this component
    type Props: Clone;

    /// Local state type (if any)
    type State: Default + Clone;

    /// Message type for internal events
    type Message;

    /// Create initial state from props (like useState)
    fn init(_props: &Self::Props) -> Self::State {
        Self::State::default()
    }

    /// Update state based on message (like reducer)
    fn update(&mut self, _msg: Self::Message, _state: &mut Self::State) -> Effect {
        Effect::None
    }

    /// Render component given props and state (pure function)
    fn view(&self, props: &Self::Props, state: &Self::State) -> Element;

    /// Lifecycle: called when props change
    fn did_update(&mut self, _old_props: &Self::Props, _new_props: &Self::Props) -> Effect {
        Effect::None
    }
}

/// Element in virtual component tree
#[derive(Clone)]
pub enum Element {
    /// A component that needs to be rendered
    Component(Box<dyn ComponentWrapper>),

    /// A widget that can be directly rendered to ratatui buffer
    Widget(Box<dyn RenderableWidget>),

    /// A container with layout and children
    Container {
        children: Vec<Element>,
        layout: ContainerLayout,
    },

    /// A fragment (just groups children, no layout)
    Fragment(Vec<Element>),

    /// Nothing to render
    None,
}

/// Layout for container elements
#[derive(Clone)]
pub enum ContainerLayout {
    Vertical(Vec<Constraint>),
    Horizontal(Vec<Constraint>),
}

/// Constraint for layout
#[derive(Clone, Copy)]
pub enum Constraint {
    Length(u16),
    Min(u16),
    Max(u16),
    Percentage(u16),
    Ratio(u32, u32),
}

/// Side effects to run after rendering
pub enum Effect {
    None,
    Action(Action),
    Batch(Vec<Effect>),
    Async(Pin<Box<dyn Future<Output = Action> + Send>>),
}

/// Trait for widgets that can be rendered directly
pub trait RenderableWidget: Send + Sync {
    /// Render this widget into the provided buffer
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render into
    /// * `buf` - The buffer to write to
    /// * `config` - Display configuration (colors, box chars, etc.)
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig);

    /// Clone this widget into a boxed trait object
    fn clone_box(&self) -> Box<dyn RenderableWidget>;

    /// Get the preferred height of this widget
    ///
    /// Returns None if the widget can adapt to any height.
    /// Returns Some(height) if the widget has a fixed or preferred height.
    fn preferred_height(&self) -> Option<u16> {
        None
    }

    /// Get the preferred width of this widget
    ///
    /// Returns None if the widget can adapt to any width.
    /// Returns Some(width) if the widget has a fixed or preferred width.
    fn preferred_width(&self) -> Option<u16> {
        None
    }
}

impl Clone for Box<dyn RenderableWidget> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Type-erased component wrapper for dynamic dispatch
pub trait ComponentWrapper: Send + Sync {
    fn view_any(&self) -> Element;
    fn clone_box(&self) -> Box<dyn ComponentWrapper>;
}

impl Clone for Box<dyn ComponentWrapper> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Helper to create a container with vertical layout
pub fn vertical<const N: usize>(constraints: [Constraint; N], children: Vec<Element>) -> Element {
    Element::Container {
        children,
        layout: ContainerLayout::Vertical(constraints.to_vec()),
    }
}

/// Helper to create a container with horizontal layout
pub fn horizontal<const N: usize>(constraints: [Constraint; N], children: Vec<Element>) -> Element {
    Element::Container {
        children,
        layout: ContainerLayout::Horizontal(constraints.to_vec()),
    }
}
