//! The painter: one function that turns a [`Recipe`] into Rune layers. It works in two steps so tests can read the first:
//! [`plan`] resolves a recipe into the [`Op`]s Rune will be asked for (washes folded into the fills they tint, opacities and scales
//! multiplied, grain recorded as skipped), and [`apply`] makes those calls on an [`El`].
//!
//! Rune's own draw order is fixed: drop shadows under the shape, then the fill, the gradient over it, inner shadows and edges, the
//! border, the children, and the outline outside. A recipe's order matters within a kind (the closest shadow first, a rim before the
//! lit edge that sits on it).

use rune::{Color, El, Shadow};

use crate::color::over;
use crate::recipe::{Layer, LayerKind, Live, Material, Recipe, State};

/// One call on a Rune element.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Fill(Color),
    Gradient { top: Color, bottom: Color },
    Linear { angle: f32, stops: Vec<(f32, Color)> },
    /// Inner (`inset`) and outer shadows, in the recipe's order.
    BoxShadow(Shadow),
    Border { width: f32, color: Color },
    Outline { width: f32, offset: f32, color: Color },
    BackdropBlur(f32),
    Opacity(f32),
    Scale(f32),
    /// A layer Rune cannot draw at this rev (grain): kept in the plan so a test or a gallery can say so.
    Skipped(LayerKind),
}

/// The Rune calls `r` turns into, in the order [`apply`] makes them.
pub fn plan(r: &Recipe) -> Vec<Op> {
    let wash = r.wash_of();
    let tint = |c: Color| wash.map_or(c, |w| over(c, w));
    let fills: Vec<Color> = r.layers.iter().filter_map(|l| if let Layer::Fill(c) = l { Some(*c) } else { None }).collect();
    let gradient = r.layers.iter().rev().find_map(|l| if let Layer::Gradient { top, bottom } = l { Some((*top, *bottom)) } else { None });
    let linear = r.layers.iter().rev().find_map(|l| if let Layer::Linear { angle, stops } = l { Some((*angle, stops.clone())) } else { None });
    let mut ops = Vec::new();
    match fills.into_iter().reduce(over) {
        Some(f) => ops.push(Op::Fill(tint(f))),
        // A wash on nothing (a quiet control) is the fill.
        None if gradient.is_none() && linear.is_none() => {
            if let Some(w) = wash {
                ops.push(Op::Fill(w));
            }
        }
        None => {}
    }
    if let Some((top, bottom)) = gradient {
        ops.push(Op::Gradient { top: tint(top), bottom: tint(bottom) });
    }
    if let Some((angle, stops)) = linear {
        ops.push(Op::Linear { angle, stops: stops.into_iter().map(|(t, c)| (t, tint(c))).collect() });
    }
    for l in &r.layers {
        match l {
            Layer::InnerTopEdge { color, width } => ops.push(Op::BoxShadow(Shadow { y: *width, color: *color, inset: true, ..Shadow::default() })),
            Layer::InnerShadow(s) => ops.push(Op::BoxShadow(Shadow { inset: true, ..*s })),
            Layer::InnerRim { color, width } => ops.push(Op::BoxShadow(Shadow { spread: *width, color: *color, inset: true, ..Shadow::default() })),
            Layer::Shadow(s) => ops.push(Op::BoxShadow(Shadow { inset: false, ..*s })),
            _ => {}
        }
    }
    if let Some((width, color)) = r.layers.iter().rev().find_map(|l| if let Layer::Border { width, color } = l { Some((*width, *color)) } else { None }) {
        ops.push(Op::Border { width, color });
    }
    if let Some((width, offset, color)) = r.layers.iter().rev().find_map(|l| if let Layer::Ring { width, offset, color } = l { Some((*width, *offset, *color)) } else { None }) {
        ops.push(Op::Outline { width, offset, color });
    }
    if let Some(s) = r.layers.iter().rev().find_map(|l| if let Layer::BackdropBlur(s) = l { Some(*s) } else { None }) {
        ops.push(Op::BackdropBlur(s));
    }
    let opacity: f32 = r.layers.iter().filter_map(|l| if let Layer::Opacity(o) = l { Some(*o) } else { None }).product();
    if r.has(LayerKind::Opacity) {
        ops.push(Op::Opacity(opacity));
    }
    let scale: f32 = r.layers.iter().filter_map(|l| if let Layer::Scale(k) = l { Some(*k) } else { None }).product();
    if r.has(LayerKind::Scale) {
        ops.push(Op::Scale(scale));
    }
    if r.has(LayerKind::Grain) {
        ops.push(Op::Skipped(LayerKind::Grain));
    }
    ops
}

/// Makes `ops`' calls on `e`.
pub fn apply(mut e: El, ops: &[Op]) -> El {
    for op in ops {
        e = match op {
            Op::Fill(c) => e.fill(*c),
            Op::Gradient { top, bottom } => e.gradient(*top, *bottom),
            Op::Linear { angle, stops } => e.linear_gradient(*angle, stops),
            Op::BoxShadow(s) => e.box_shadow(*s),
            Op::Border { width, color } => e.border(*width, *color),
            Op::Outline { width, offset, color } => e.outline(*width, *offset, *color),
            Op::BackdropBlur(s) => e.backdrop_blur(*s),
            Op::Opacity(o) => e.opacity(*o),
            Op::Scale(k) => e.scale(*k),
            Op::Skipped(_) => e,
        };
    }
    e
}

/// `e` painted with `r`: the one painter.
pub fn paint(e: El, r: &Recipe) -> El {
    apply(e, &plan(r))
}

/// What Rune is asked to draw live for a material: its hover and press washes, its press scale, its keyboard focus ring.
pub fn apply_live(mut e: El, live: &Live) -> El {
    if let Some(c) = live.hover {
        e = e.hover(c);
    }
    if let Some(c) = live.press {
        e = e.press(c);
    }
    if let Some(k) = live.press_scale {
        e = e.press_scale(k);
    }
    if let Some((w, o, c)) = live.focus_ring {
        e = e.focus_ring(w, o, c);
    }
    e
}

/// A live state pinned for a gallery or a test: `Live` is the real thing (Rune draws hover, press and focus as they happen).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Look {
    #[default]
    Live,
    Hover,
    Pressed,
    Focused,
}

impl Look {
    pub const ALL: [Look; 4] = [Look::Live, Look::Hover, Look::Pressed, Look::Focused];

    pub fn word(self) -> &'static str {
        match self {
            Look::Live => "Default",
            Look::Hover => "Hover",
            Look::Pressed => "Pressed",
            Look::Focused => "Focused",
        }
    }

    pub fn state(self) -> State {
        match self {
            Look::Live => State::Rest,
            Look::Hover => State::Hover,
            Look::Pressed => State::Pressed,
            Look::Focused => State::Focused,
        }
    }
}

/// The recipe a control is drawn with: disabled wins; selected is the base the live states add to; a pinned look is that state's
/// recipe (on a selected control, the selected recipe plus that state's wash, scale and ring).
pub fn resolve(m: &Material, look: Look, disabled: bool, selected: bool) -> Recipe {
    if disabled {
        return m.disabled.clone();
    }
    let state = m.state(look.state());
    if !selected {
        return state.clone();
    }
    let mut r = m.selected.clone();
    if let Some(w) = state.wash_of() {
        r = r.wash(w);
    }
    if let Some(k) = state.scale_of() {
        r = r.scale(k);
    }
    if let Some((w, o, c)) = state.ring_of() {
        r = r.ring(w, o, c);
    }
    r
}

/// `e` painted as `m` in `look`: the recipe from [`resolve`], and (when live and enabled) Rune's live hover, press and focus.
pub fn paint_material(e: El, m: &Material, look: Look, disabled: bool, selected: bool) -> El {
    let e = paint(e, &resolve(m, look, disabled, selected));
    if look == Look::Live && !disabled { apply_live(e, &m.live()) } else { e }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wash_folds_into_the_fill_and_the_gradient_it_tints() {
        let r = Recipe::new().fill(Color::WHITE).gradient(Color::WHITE, Color::WHITE).wash(Color::BLACK.alpha(0.1));
        let ops = plan(&r);
        let Op::Fill(f) = ops[0] else { panic!("fill first: {ops:?}") };
        let Op::Gradient { top, .. } = ops[1] else { panic!("gradient second: {ops:?}") };
        assert!(f.r < 1.0 && top.r < 1.0, "both tinted");
        assert_eq!(ops.len(), 2, "no op for the wash itself");
    }

    #[test]
    fn a_wash_on_nothing_is_the_fill() {
        let ops = plan(&Recipe::new().wash(Color::BLACK.alpha(0.05)));
        assert_eq!(ops, vec![Op::Fill(Color::BLACK.alpha(0.05))]);
    }

    #[test]
    fn edges_rims_and_drops_become_box_shadows_in_order() {
        let drop = Shadow { y: 2.0, blur: 4.0, color: Color::BLACK.alpha(0.1), ..Shadow::default() };
        let r = Recipe::new().fill(Color::BLACK).inner_rim(Color::BLACK.alpha(0.5)).inner_top_edge(Color::WHITE.alpha(0.2)).shadow(drop);
        let shadows: Vec<Shadow> = plan(&r).into_iter().filter_map(|o| if let Op::BoxShadow(s) = o { Some(s) } else { None }).collect();
        assert_eq!(shadows.len(), 3);
        assert!(shadows[0].inset && shadows[0].spread == 1.0 && shadows[0].y == 0.0, "rim: inset spread");
        assert!(shadows[1].inset && shadows[1].y == 1.0 && shadows[1].blur == 0.0, "edge: inset, pushed down by its width");
        assert!(!shadows[2].inset && shadows[2] == drop, "drop: outside");
    }

    #[test]
    fn opacity_and_scale_multiply_and_grain_is_skipped_not_lost() {
        let ops = plan(&Recipe::new().fill(Color::WHITE).opacity(0.5).opacity(0.5).scale(0.96).grain(0.05));
        assert!(ops.contains(&Op::Opacity(0.25)));
        assert!(ops.contains(&Op::Scale(0.96)));
        assert_eq!(ops.last(), Some(&Op::Skipped(LayerKind::Grain)));
    }

    #[test]
    fn a_selected_control_keeps_its_selected_look_under_a_pinned_state() {
        let m = Material::new(Recipe::new().fill(Color::WHITE)).hover(|r| r.wash(Color::BLACK.alpha(0.1))).selected(Recipe::new().fill(Color::BLACK));
        let r = resolve(&m, Look::Hover, false, true);
        assert_eq!(r.kinds(), vec![LayerKind::Fill, LayerKind::Wash]);
        assert_eq!(resolve(&m, Look::Hover, true, true), m.disabled, "disabled wins");
    }
}
