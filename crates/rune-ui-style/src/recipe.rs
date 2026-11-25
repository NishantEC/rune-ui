//! Recipes: what a surface is made of, as data. A [`Recipe`] is an ordered list of [`Layer`]s, bottom of
//! the stack first; a [`Material`] is a recipe for each state a control can be in. Nothing here draws:
//! [`crate::paint`] turns a recipe into Rune layers, and tests read recipes as plain values.
//!
//! ```
//! use rune::{Color, Shadow};
//! use rune_ui_style::{Layer, Material, Recipe};
//!
//! let raised = Recipe::new()
//!     .gradient(Color::WHITE, Color::WHITE.alpha(0.9))
//!     .inner_top_edge(Color::WHITE)
//!     .shadow(Shadow { y: 1.0, blur: 2.0, color: Color::BLACK.alpha(0.06), ..Shadow::default() });
//! let button = Material::new(raised).pressed(|r| r.scale(0.96).shadow_none());
//! assert!(button.pressed.has(rune_ui_style::LayerKind::Scale));
//! assert!(!button.pressed.has(rune_ui_style::LayerKind::Shadow));
//! # let _ = Layer::Fill(Color::WHITE);
//! ```

use rune::{Color, Shadow};

/// One layer of a surface, bottom first. Outer shadows sit under the surface whatever their place in the list; inner layers draw in
/// their order (fill, then gradients, then inner shadows and edges, then a border), as Rune draws them.
#[derive(Clone, Debug, PartialEq)]
pub enum Layer {
    /// A flat colour.
    Fill(Color),
    /// A vertical gradient, top colour to bottom colour, over the fill.
    Gradient { top: Color, bottom: Color },
    /// A linear gradient at `angle` radians (CSS convention: 0 is bottom to top) with up to 16 stops.
    Linear { angle: f32, stops: Vec<(f32, Color)> },
    /// A lit top edge: a `width` point line of light just inside the top edge (an inset shadow pushed down by its own width).
    InnerTopEdge { color: Color, width: f32 },
    /// A soft shadow inside the shape (a well's top, pressed in).
    InnerShadow(Shadow),
    /// A rim just inside the edge, all round (an inset ring): part of the object, not a separator.
    InnerRim { color: Color, width: f32 },
    /// A drop shadow outside the shape. Several make a smooth stack.
    Shadow(Shadow),
    /// A ring outside the shape, `offset` points away (focus, an invalid field).
    Ring { width: f32, offset: f32, color: Color },
    /// A hairline just inside the edge. Structure only (a divider, a table cell), never depth.
    Border { width: f32, color: Color },
    /// A state's tint over everything below (hover, press), at the colour's own alpha.
    Wash(Color),
    /// The whole object scaled about its centre (a press sinks it).
    Scale(f32),
    /// The whole object's opacity (disabled).
    Opacity(f32),
    /// What is behind the surface, blurred by `sigma` points (glass). Rune allows a few per frame.
    BackdropBlur(f32),
    /// Film grain at `amount` (0 to 1). Rune has no grain primitive at this rev, so the painter records it and skips it.
    Grain { amount: f32 },
}

/// A layer's kind, for asking a recipe what it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Fill,
    Gradient,
    Linear,
    InnerTopEdge,
    InnerShadow,
    InnerRim,
    Shadow,
    Ring,
    Border,
    Wash,
    Scale,
    Opacity,
    BackdropBlur,
    Grain,
}

impl Layer {
    pub fn kind(&self) -> LayerKind {
        match self {
            Layer::Fill(_) => LayerKind::Fill,
            Layer::Gradient { .. } => LayerKind::Gradient,
            Layer::Linear { .. } => LayerKind::Linear,
            Layer::InnerTopEdge { .. } => LayerKind::InnerTopEdge,
            Layer::InnerShadow(_) => LayerKind::InnerShadow,
            Layer::InnerRim { .. } => LayerKind::InnerRim,
            Layer::Shadow(_) => LayerKind::Shadow,
            Layer::Ring { .. } => LayerKind::Ring,
            Layer::Border { .. } => LayerKind::Border,
            Layer::Wash(_) => LayerKind::Wash,
            Layer::Scale(_) => LayerKind::Scale,
            Layer::Opacity(_) => LayerKind::Opacity,
            Layer::BackdropBlur(_) => LayerKind::BackdropBlur,
            Layer::Grain { .. } => LayerKind::Grain,
        }
    }
}

/// An ordered stack of layers. Built by chaining; every method returns a new recipe, so a state is "the rest recipe, plus or minus".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recipe {
    pub layers: Vec<Layer>,
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe::default()
    }

    /// Adds any layer.
    pub fn layer(mut self, l: Layer) -> Recipe {
        self.layers.push(l);
        self
    }

    pub fn fill(self, c: Color) -> Recipe {
        self.layer(Layer::Fill(c))
    }

    pub fn gradient(self, top: Color, bottom: Color) -> Recipe {
        self.layer(Layer::Gradient { top, bottom })
    }

    pub fn linear(self, angle: f32, stops: &[(f32, Color)]) -> Recipe {
        self.layer(Layer::Linear { angle, stops: stops.to_vec() })
    }

    /// A 1 point lit top edge.
    pub fn inner_top_edge(self, c: Color) -> Recipe {
        self.layer(Layer::InnerTopEdge { color: c, width: 1.0 })
    }

    /// An inner shadow `y` down with a CSS `blur`.
    pub fn inner_shadow(self, y: f32, blur: f32, c: Color) -> Recipe {
        self.layer(Layer::InnerShadow(Shadow { y, blur, color: c, inset: true, ..Shadow::default() }))
    }

    /// A 1 point inner rim.
    pub fn inner_rim(self, c: Color) -> Recipe {
        self.layer(Layer::InnerRim { color: c, width: 1.0 })
    }

    pub fn shadow(self, s: Shadow) -> Recipe {
        self.layer(Layer::Shadow(Shadow { inset: false, ..s }))
    }

    /// A stack of drop shadows (an [`crate::Elevation`] step), closest first.
    pub fn shadows(mut self, stack: &[Shadow]) -> Recipe {
        self.layers.extend(stack.iter().map(|s| Layer::Shadow(Shadow { inset: false, ..*s })));
        self
    }

    /// A `width` ring `offset` points outside.
    pub fn ring(self, width: f32, offset: f32, c: Color) -> Recipe {
        self.layer(Layer::Ring { width, offset, color: c })
    }

    pub fn border(self, width: f32, c: Color) -> Recipe {
        self.layer(Layer::Border { width, color: c })
    }

    pub fn wash(self, c: Color) -> Recipe {
        self.layer(Layer::Wash(c))
    }

    pub fn scale(self, k: f32) -> Recipe {
        self.layer(Layer::Scale(k))
    }

    pub fn opacity(self, o: f32) -> Recipe {
        self.layer(Layer::Opacity(o))
    }

    pub fn backdrop_blur(self, sigma: f32) -> Recipe {
        self.layer(Layer::BackdropBlur(sigma))
    }

    pub fn grain(self, amount: f32) -> Recipe {
        self.layer(Layer::Grain { amount })
    }

    /// The recipe without any layer of `kind`.
    pub fn without(mut self, kind: LayerKind) -> Recipe {
        self.layers.retain(|l| l.kind() != kind);
        self
    }

    /// No drop shadows (a pressed control sits on the page; a disabled one does not lift).
    pub fn shadow_none(self) -> Recipe {
        self.without(LayerKind::Shadow)
    }

    /// `other`'s layers after this recipe's.
    pub fn then(mut self, other: &Recipe) -> Recipe {
        self.layers.extend(other.layers.iter().cloned());
        self
    }

    pub fn has(&self, kind: LayerKind) -> bool {
        self.layers.iter().any(|l| l.kind() == kind)
    }

    pub fn count(&self, kind: LayerKind) -> usize {
        self.layers.iter().filter(|l| l.kind() == kind).count()
    }

    pub fn kinds(&self) -> Vec<LayerKind> {
        self.layers.iter().map(Layer::kind).collect()
    }

    /// The first ring, if any.
    pub fn ring_of(&self) -> Option<(f32, f32, Color)> {
        self.layers.iter().find_map(|l| match l {
            Layer::Ring { width, offset, color } => Some((*width, *offset, *color)),
            _ => None,
        })
    }

    /// Every wash, laid over each other, as one colour.
    pub fn wash_of(&self) -> Option<Color> {
        self.layers.iter().filter_map(|l| if let Layer::Wash(c) = l { Some(*c) } else { None }).reduce(crate::color::over)
    }

    /// The last scale, if any.
    pub fn scale_of(&self) -> Option<f32> {
        self.layers.iter().rev().find_map(|l| if let Layer::Scale(k) = l { Some(*k) } else { None })
    }

    /// Washes, scales and rings removed: what a state changes is gone, the material itself stays.
    pub fn at_rest(self) -> Recipe {
        self.without(LayerKind::Wash).without(LayerKind::Scale).without(LayerKind::Ring)
    }
}

/// The states a control is drawn in. Hover, pressed and focused happen live (Rune draws them as the pointer and keys move);
/// disabled and selected are what the app says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
    #[default]
    Rest,
    Hover,
    Pressed,
    Focused,
    Disabled,
    Selected,
}

impl State {
    pub const ALL: [State; 6] = [State::Rest, State::Hover, State::Pressed, State::Focused, State::Disabled, State::Selected];

    pub fn word(self) -> &'static str {
        match self {
            State::Rest => "Rest",
            State::Hover => "Hover",
            State::Pressed => "Pressed",
            State::Focused => "Focused",
            State::Disabled => "Disabled",
            State::Selected => "Selected",
        }
    }
}

/// A recipe for every state. Each state is a whole recipe (data, not a rule), made from the rest recipe when the material is built.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub rest: Recipe,
    pub hover: Recipe,
    pub pressed: Recipe,
    pub focused: Recipe,
    pub disabled: Recipe,
    pub selected: Recipe,
}

impl Material {
    /// Every state the same as `rest` until a builder below changes it.
    pub fn new(rest: Recipe) -> Material {
        Material { hover: rest.clone(), pressed: rest.clone(), focused: rest.clone(), disabled: rest.clone(), selected: rest.clone(), rest }
    }

    pub fn hover(mut self, f: impl FnOnce(Recipe) -> Recipe) -> Material {
        self.hover = f(self.rest.clone());
        self
    }

    pub fn pressed(mut self, f: impl FnOnce(Recipe) -> Recipe) -> Material {
        self.pressed = f(self.rest.clone());
        self
    }

    pub fn focused(mut self, f: impl FnOnce(Recipe) -> Recipe) -> Material {
        self.focused = f(self.rest.clone());
        self
    }

    pub fn disabled(mut self, f: impl FnOnce(Recipe) -> Recipe) -> Material {
        self.disabled = f(self.rest.clone());
        self
    }

    /// The selected state from scratch (it is often another material: a raised toggle that stays down is a well).
    pub fn selected(mut self, r: Recipe) -> Material {
        self.selected = r;
        self
    }

    pub fn state(&self, s: State) -> &Recipe {
        match s {
            State::Rest => &self.rest,
            State::Hover => &self.hover,
            State::Pressed => &self.pressed,
            State::Focused => &self.focused,
            State::Disabled => &self.disabled,
            State::Selected => &self.selected,
        }
    }

    /// What Rune draws live for this material: the hover and press washes, the press scale and the keyboard focus ring.
    pub fn live(&self) -> Live {
        Live { hover: self.hover.wash_of(), press: self.pressed.wash_of(), press_scale: self.pressed.scale_of(), focus_ring: self.focused.ring_of() }
    }
}

/// The interactive parts of a material, as Rune draws them while the app runs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Live {
    pub hover: Option<Color>,
    pub press: Option<Color>,
    pub press_scale: Option<f32>,
    pub focus_ring: Option<(f32, f32, Color)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recipe_keeps_its_order_and_states_are_whole_recipes() {
        let rest = Recipe::new().fill(Color::WHITE).inner_top_edge(Color::WHITE).shadow(Shadow { y: 1.0, ..Shadow::default() });
        assert_eq!(rest.kinds(), vec![LayerKind::Fill, LayerKind::InnerTopEdge, LayerKind::Shadow]);
        let m = Material::new(rest.clone()).pressed(|r| r.scale(0.96).shadow_none()).focused(|r| r.ring(2.0, 2.0, Color::BLACK));
        assert_eq!(m.rest, rest, "building a state never changes rest");
        assert_eq!(m.pressed.kinds(), vec![LayerKind::Fill, LayerKind::InnerTopEdge, LayerKind::Scale]);
        assert_eq!(m.hover, rest, "an unset state is the rest recipe");
        let live = m.live();
        assert_eq!(live.press_scale, Some(0.96));
        assert_eq!(live.focus_ring, Some((2.0, 2.0, Color::BLACK)));
        assert_eq!(live.hover, None);
    }

    #[test]
    fn washes_fold_into_one_colour_and_at_rest_drops_state_layers() {
        let r = Recipe::new().fill(Color::WHITE).wash(Color::BLACK.alpha(0.1)).wash(Color::BLACK.alpha(0.1)).scale(0.9).ring(2.0, 2.0, Color::BLACK);
        let w = r.wash_of().unwrap();
        assert!((w.a - 0.19).abs() < 1e-4, "two 10% washes are 19%");
        assert_eq!(r.at_rest().kinds(), vec![LayerKind::Fill]);
    }
}
