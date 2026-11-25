//! Recipes: what a surface is made of, as data.
//!
//! * A [`Recipe`] is an ordered list of [`Layer`]s, bottom of the stack first.
//! * A [`Delta`] changes a recipe: layers to remove, layers to add, or a whole replacement.
//! * A [`Material`] is a base recipe and one delta for each [`State`] of the closed set.
//! * [`Parts`] names a component's parts: its `root` material and the [`Tint`] of its `label` and `icon`.
//! * [`resolve`] is the one resolver: the base, then `selected`, then the one interaction state that wins
//!   (**disabled > loading > pressed > hover > rest**), then `invalid`, `open` and the focus ring on top.
//!
//! ```
//! use rune::{Color, Shadow};
//! use rune_ui_style::{Delta, Material, Recipe, State, States, resolve};
//!
//! let raised = Recipe::new().gradient(Color::WHITE, Color::WHITE.alpha(0.9)).inner_top_edge(Color::WHITE)
//!     .shadow(Shadow { y: 1.0, blur: 2.0, color: Color::BLACK.alpha(0.06), ..Shadow::default() });
//! let button = Material::new(raised).on(State::Pressed, Delta::new().shadow_none().scale(0.96));
//! let pressed = resolve(&button, States::rest().with(State::Pressed));
//! assert_eq!(pressed.scale_of(), Some(0.96));
//! ```

use rune::{Color, Shadow};

/// One layer of a surface. Outer shadows sit under the surface whatever their place in the list; inner layers draw in their order (fill,
/// gradients, inner shadows and edges, then a border), as Rune draws them.
#[derive(Clone, Debug, PartialEq)]
pub enum Layer {
    Fill(Color),
    /// A vertical gradient over the fill.
    Gradient { top: Color, bottom: Color },
    /// A linear gradient at `angle` radians (CSS convention: 0 is bottom to top), up to 16 stops.
    Linear { angle: f32, stops: Vec<(f32, Color)> },
    /// A lit top edge: a line of light just inside the top edge.
    InnerTopEdge { color: Color, width: f32 },
    /// A soft shadow inside the shape (a well's top: pressed in).
    InnerShadow(Shadow),
    /// A rim just inside the edge, all round: part of the object, not a separator.
    InnerRim { color: Color, width: f32 },
    /// A drop shadow outside the shape. Several make a smooth stack.
    Shadow(Shadow),
    /// A ring outside the shape, `offset` points away (focus, invalid).
    Ring { width: f32, offset: f32, color: Color },
    /// A hairline just inside the edge. Structure only, never depth.
    Border { width: f32, color: Color },
    /// A state's tint over everything below (hover, press, selected).
    Wash(Color),
    /// The whole object scaled about its centre.
    Scale(f32),
    Opacity(f32),
    /// What is behind, blurred by `sigma` points (glass).
    BackdropBlur(f32),
    /// Film grain at `amount`. Rune has no grain primitive at this rev: the painter records it and skips it.
    Grain { amount: f32 },
}

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

/// Builders shared by [`Recipe`] and [`Delta`] (each adds a layer).
macro_rules! layer_builders {
    () => {
        pub fn fill(self, c: Color) -> Self {
            self.layer(Layer::Fill(c))
        }
        pub fn gradient(self, top: Color, bottom: Color) -> Self {
            self.layer(Layer::Gradient { top, bottom })
        }
        pub fn linear(self, angle: f32, stops: &[(f32, Color)]) -> Self {
            self.layer(Layer::Linear { angle, stops: stops.to_vec() })
        }
        /// A 1 point lit top edge.
        pub fn inner_top_edge(self, c: Color) -> Self {
            self.layer(Layer::InnerTopEdge { color: c, width: 1.0 })
        }
        pub fn inner_shadow(self, y: f32, blur: f32, c: Color) -> Self {
            self.layer(Layer::InnerShadow(Shadow { y, blur, color: c, inset: true, ..Shadow::default() }))
        }
        /// A 1 point inner rim.
        pub fn inner_rim(self, c: Color) -> Self {
            self.layer(Layer::InnerRim { color: c, width: 1.0 })
        }
        pub fn shadow(self, s: Shadow) -> Self {
            self.layer(Layer::Shadow(Shadow { inset: false, ..s }))
        }
        /// A stack of drop shadows (an elevation step), closest first.
        pub fn shadows(mut self, stack: &[Shadow]) -> Self {
            for s in stack {
                self = self.shadow(*s);
            }
            self
        }
        pub fn ring(self, width: f32, offset: f32, c: Color) -> Self {
            self.layer(Layer::Ring { width, offset, color: c })
        }
        pub fn border(self, width: f32, c: Color) -> Self {
            self.layer(Layer::Border { width, color: c })
        }
        pub fn wash(self, c: Color) -> Self {
            self.layer(Layer::Wash(c))
        }
        pub fn scale(self, k: f32) -> Self {
            self.layer(Layer::Scale(k))
        }
        pub fn opacity(self, o: f32) -> Self {
            self.layer(Layer::Opacity(o))
        }
        pub fn backdrop_blur(self, sigma: f32) -> Self {
            self.layer(Layer::BackdropBlur(sigma))
        }
        pub fn grain(self, amount: f32) -> Self {
            self.layer(Layer::Grain { amount })
        }
    };
}

/// An ordered stack of layers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recipe {
    pub layers: Vec<Layer>,
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe::default()
    }

    pub fn layer(mut self, l: Layer) -> Recipe {
        self.layers.push(l);
        self
    }

    layer_builders!();

    pub fn without(mut self, kind: LayerKind) -> Recipe {
        self.layers.retain(|l| l.kind() != kind);
        self
    }

    pub fn shadow_none(self) -> Recipe {
        self.without(LayerKind::Shadow)
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

    /// The last ring.
    pub fn ring_of(&self) -> Option<(f32, f32, Color)> {
        self.layers.iter().rev().find_map(|l| if let Layer::Ring { width, offset, color } = l { Some((*width, *offset, *color)) } else { None })
    }

    /// Every wash laid over the next, as one colour.
    pub fn wash_of(&self) -> Option<Color> {
        self.layers.iter().filter_map(|l| if let Layer::Wash(c) = l { Some(*c) } else { None }).reduce(crate::color::over)
    }

    /// Every scale multiplied.
    pub fn scale_of(&self) -> Option<f32> {
        self.has(LayerKind::Scale).then(|| self.layers.iter().filter_map(|l| if let Layer::Scale(k) = l { Some(*k) } else { None }).product())
    }

    /// `d` applied.
    pub fn apply(self, d: &Delta) -> Recipe {
        let mut r = d.replace.clone().unwrap_or(self);
        r.layers.retain(|l| !d.remove.contains(&l.kind()));
        r.layers.extend(d.add.iter().cloned());
        r
    }
}

/// A change to a recipe: a replacement (if any), then layer kinds removed, then layers added.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Delta {
    pub replace: Option<Recipe>,
    pub remove: Vec<LayerKind>,
    pub add: Vec<Layer>,
}

impl Delta {
    pub fn new() -> Delta {
        Delta::default()
    }

    /// Starts from `r` instead of what came before (a raised toggle that stays down becomes a well).
    pub fn replace(r: Recipe) -> Delta {
        Delta { replace: Some(r), ..Delta::default() }
    }

    pub fn layer(mut self, l: Layer) -> Delta {
        self.add.push(l);
        self
    }

    layer_builders!();

    pub fn remove(mut self, kind: LayerKind) -> Delta {
        self.remove.push(kind);
        self
    }

    pub fn shadow_none(self) -> Delta {
        self.remove(LayerKind::Shadow)
    }

    pub fn is_empty(&self) -> bool {
        self.replace.is_none() && self.remove.is_empty() && self.add.is_empty()
    }
}

/// The closed set of states a control is drawn in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
    #[default]
    Rest,
    Hover,
    Pressed,
    /// Focus from the keyboard (Rune shows the ring only then).
    FocusVisible,
    Disabled,
    Selected,
    Invalid,
    Loading,
    Open,
}

impl State {
    pub const ALL: [State; 9] = [State::Rest, State::Hover, State::Pressed, State::FocusVisible, State::Disabled, State::Selected, State::Invalid, State::Loading, State::Open];

    pub fn word(self) -> &'static str {
        match self {
            State::Rest => "Rest",
            State::Hover => "Hover",
            State::Pressed => "Pressed",
            State::FocusVisible => "Focus",
            State::Disabled => "Disabled",
            State::Selected => "Selected",
            State::Invalid => "Invalid",
            State::Loading => "Loading",
            State::Open => "Open",
        }
    }
}

/// Which states hold right now. Hover, pressed and focus are usually Rune's to draw live; a gallery sets them to pin a look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct States {
    pub hover: bool,
    pub pressed: bool,
    pub focus_visible: bool,
    pub disabled: bool,
    pub selected: bool,
    pub invalid: bool,
    pub loading: bool,
    pub open: bool,
}

impl States {
    pub fn rest() -> States {
        States::default()
    }

    pub fn with(mut self, s: State) -> States {
        self.set(s, true);
        self
    }

    pub fn set(&mut self, s: State, on: bool) {
        match s {
            State::Rest => {}
            State::Hover => self.hover = on,
            State::Pressed => self.pressed = on,
            State::FocusVisible => self.focus_visible = on,
            State::Disabled => self.disabled = on,
            State::Selected => self.selected = on,
            State::Invalid => self.invalid = on,
            State::Loading => self.loading = on,
            State::Open => self.open = on,
        }
    }

    /// Whether the app's state allows a press and live hover (not disabled, not loading).
    pub fn live(&self) -> bool {
        !self.disabled && !self.loading
    }

    /// Whether a look is pinned (hover, pressed or focus set by hand rather than by the pointer).
    pub fn pinned(&self) -> bool {
        self.hover || self.pressed || self.focus_visible
    }

    /// The interaction state that wins: disabled > loading > pressed > hover > rest.
    pub fn interaction(&self) -> State {
        if self.disabled {
            State::Disabled
        } else if self.loading {
            State::Loading
        } else if self.pressed {
            State::Pressed
        } else if self.hover {
            State::Hover
        } else {
            State::Rest
        }
    }
}

/// A base recipe and a delta per state.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub base: Recipe,
    pub hover: Delta,
    pub pressed: Delta,
    pub focus: Delta,
    pub disabled: Delta,
    pub selected: Delta,
    pub invalid: Delta,
    pub loading: Delta,
    pub open: Delta,
}

impl Material {
    pub fn new(base: Recipe) -> Material {
        Material { base, hover: Delta::new(), pressed: Delta::new(), focus: Delta::new(), disabled: Delta::new(), selected: Delta::new(), invalid: Delta::new(), loading: Delta::new(), open: Delta::new() }
    }

    /// Sets the delta for `s` (`Rest` changes the base itself).
    pub fn on(mut self, s: State, d: Delta) -> Material {
        match s {
            State::Rest => self.base = self.base.clone().apply(&d),
            State::Hover => self.hover = d,
            State::Pressed => self.pressed = d,
            State::FocusVisible => self.focus = d,
            State::Disabled => self.disabled = d,
            State::Selected => self.selected = d,
            State::Invalid => self.invalid = d,
            State::Loading => self.loading = d,
            State::Open => self.open = d,
        }
        self
    }

    pub fn delta(&self, s: State) -> Option<&Delta> {
        Some(match s {
            State::Rest => return None,
            State::Hover => &self.hover,
            State::Pressed => &self.pressed,
            State::FocusVisible => &self.focus,
            State::Disabled => &self.disabled,
            State::Selected => &self.selected,
            State::Invalid => &self.invalid,
            State::Loading => &self.loading,
            State::Open => &self.open,
        })
    }

    /// The recipe in one state alone (galleries, tests).
    pub fn state(&self, s: State) -> Recipe {
        resolve(self, States::rest().with(s))
    }

    /// What Rune draws live for this material: hover and press washes, press scale, keyboard focus ring.
    pub fn live(&self) -> Live {
        let added = |d: &Delta| Recipe { layers: d.add.clone() };
        Live { hover: added(&self.hover).wash_of(), press: added(&self.pressed).wash_of(), press_scale: added(&self.pressed).scale_of(), focus_ring: added(&self.focus).ring_of() }
    }
}

/// The parts Rune draws live.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Live {
    pub hover: Option<Color>,
    pub press: Option<Color>,
    pub press_scale: Option<f32>,
    pub focus_ring: Option<(f32, f32, Color)>,
}

/// The one resolver.
pub fn resolve(m: &Material, s: States) -> Recipe {
    let mut r = m.base.clone();
    if s.selected {
        r = r.apply(&m.selected);
    }
    if let Some(d) = m.delta(s.interaction()) {
        r = r.apply(d);
    }
    if !s.disabled {
        if s.invalid {
            r = r.apply(&m.invalid);
        }
        if s.open {
            r = r.apply(&m.open);
        }
        if s.focus_visible {
            r = r.apply(&m.focus);
        }
    }
    r
}

/// A text or icon colour per state: disabled > invalid > selected > rest. (Disabled usually dims the whole control instead.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub rest: Color,
    pub selected: Option<Color>,
    pub invalid: Option<Color>,
    pub disabled: Option<Color>,
}

impl Tint {
    pub fn new(rest: Color) -> Tint {
        Tint { rest, selected: None, invalid: None, disabled: None }
    }

    pub fn selected(mut self, c: Color) -> Tint {
        self.selected = Some(c);
        self
    }

    pub fn invalid(mut self, c: Color) -> Tint {
        self.invalid = Some(c);
        self
    }

    pub fn resolve(&self, s: States) -> Color {
        (s.disabled.then_some(self.disabled).flatten()).or(s.invalid.then_some(self.invalid).flatten()).or(s.selected.then_some(self.selected).flatten()).unwrap_or(self.rest)
    }
}

/// A component's named parts.
#[derive(Clone, Debug, PartialEq)]
pub struct Parts {
    pub root: Material,
    pub label: Tint,
    pub icon: Tint,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m() -> Material {
        Material::new(Recipe::new().fill(Color::WHITE).shadow(Shadow { y: 1.0, ..Shadow::default() }))
            .on(State::Hover, Delta::new().wash(Color::BLACK.alpha(0.1)))
            .on(State::Pressed, Delta::new().shadow_none().scale(0.96))
            .on(State::Disabled, Delta::new().shadow_none().opacity(0.45))
            .on(State::Loading, Delta::new().opacity(0.8))
            .on(State::FocusVisible, Delta::new().ring(2.0, 2.0, Color::BLACK))
            .on(State::Selected, Delta::replace(Recipe::new().fill(Color::BLACK)))
            .on(State::Invalid, Delta::new().ring(2.0, 0.0, Color::WHITE))
    }

    #[test]
    fn precedence_is_disabled_then_loading_then_pressed_then_hover() {
        let m = m();
        let all = States { hover: true, pressed: true, loading: true, disabled: true, ..States::default() };
        assert!(resolve(&m, all).has(LayerKind::Opacity) && resolve(&m, all).scale_of().is_none(), "disabled wins");
        let s = States { hover: true, pressed: true, loading: true, ..States::default() };
        assert_eq!(resolve(&m, s).kinds(), vec![LayerKind::Fill, LayerKind::Shadow, LayerKind::Opacity], "loading beats pressed");
        let s = States { hover: true, pressed: true, ..States::default() };
        assert_eq!(resolve(&m, s).scale_of(), Some(0.96), "pressed beats hover");
        assert!(resolve(&m, States::rest().with(State::Hover)).wash_of().is_some());
    }

    #[test]
    fn selected_sits_under_the_interaction_and_focus_sits_on_top() {
        let m = m();
        let s = States { selected: true, hover: true, focus_visible: true, ..States::default() };
        assert_eq!(resolve(&m, s).kinds(), vec![LayerKind::Fill, LayerKind::Wash, LayerKind::Ring]);
        assert_eq!(resolve(&m, s).layers[0], Layer::Fill(Color::BLACK));
    }

    #[test]
    fn a_disabled_control_shows_no_focus_or_invalid_ring() {
        let s = States { disabled: true, focus_visible: true, invalid: true, ..States::default() };
        assert!(!resolve(&m(), s).has(LayerKind::Ring));
    }

    #[test]
    fn live_reads_the_added_layers() {
        let live = m().live();
        assert_eq!(live.press_scale, Some(0.96));
        assert_eq!(live.focus_ring, Some((2.0, 2.0, Color::BLACK)));
        assert!(live.hover.is_some() && live.press.is_none());
    }

    #[test]
    fn a_tint_follows_the_same_precedence() {
        let t = Tint::new(Color::BLACK).selected(Color::WHITE).invalid(Color::TRANSPARENT);
        assert_eq!(t.resolve(States::rest().with(State::Selected)), Color::WHITE);
        assert_eq!(t.resolve(States { selected: true, invalid: true, ..States::default() }), Color::TRANSPARENT);
    }
}
