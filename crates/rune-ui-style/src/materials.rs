//! The materials: the few surfaces everything is made of, built from a theme's tokens.
//!
//! * **Raised**: anything you press. Lighter at the top, a lit top edge, a contact shadow. It answers a press by sinking (the scale),
//!   not by changing colour. Selected, it stays down: it becomes a well.
//! * **Ink**: the one primary action in a view. The raised recipe in the darkest (light) or lightest (dark) colour, glossier, with an
//!   inner rim so it reads as one solid object.
//! * **Danger**: the ink recipe in the danger colour, for what cannot be taken back.
//! * **Well**: anything you put something into or that holds a value. A flat tint pressed into the page, a soft shadow inside its top
//!   edge. Focus is a ring that hugs it; invalid rings it in the danger colour.
//! * **Floating**: the only things with a real drop shadow (menus, popovers, dialogs). A smooth stack with a hairline ring baked in last.
//! * **Slab**: a group's surface, one fill a step off the page. Rows inside are split by hairlines.
//! * **Ghost**: nothing until hovered (toolbar actions). **Row**: a ghost that does not sink (list rows, menu-like items).
//! * **Outline**: a quiet control on the page with its rim drawn inside (shadcn's outline variant).
//! * **Thumb**: the disc a track holds. **Chip**: raised, the one control that lifts off the page.
//!
//! Tracks are wells holding a raised thumb: up means press me, down means put something in me.

use rune::{Color, Shadow};

use crate::color::mix;
use crate::recipe::{Delta, Material, Recipe, State};
use crate::scale::{Controls, Elevation, Motion};

/// The colours the materials are made from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialColors {
    pub raised_top: Color,
    pub raised_bottom: Color,
    pub raised_edge: Color,
    pub ink_top: Color,
    pub ink_bottom: Color,
    pub ink_rim: Color,
    pub ink_edge: Color,
    pub danger: Color,
    pub danger_ring: Color,
    pub well: Color,
    pub well_shadow: Color,
    pub well_blur: f32,
    pub surface: Color,
    pub floating: Color,
    pub float_ring: Color,
    pub line: Color,
    pub hover: Color,
    pub pressed: Color,
    pub selected: Color,
    pub thumb: Color,
    pub shade: Color,
    pub focus: Color,
    pub grain: f32,
}

/// Every material of a theme.
#[derive(Clone, Debug, PartialEq)]
pub struct Materials {
    pub raised: Material,
    pub ink: Material,
    pub danger: Material,
    pub well: Material,
    pub floating: Material,
    pub slab: Material,
    pub ghost: Material,
    pub row: Material,
    pub outline: Material,
    pub thumb: Material,
    pub chip: Material,
    /// A large backdrop with atmosphere (the slab with grain). Never on controls.
    pub atmosphere: Recipe,
}

impl Materials {
    pub fn derive(c: &MaterialColors, e: &Elevation, m: &Motion, k: &Controls) -> Materials {
        let ring = || Delta::new().ring(k.ring, k.ring_offset, c.focus);
        let sink = || Delta::new().shadow_none().scale(m.press_scale);
        let off = || Delta::new().shadow_none().opacity(k.disabled);
        let busy = || Delta::new().shadow_none();
        let control = |base: Recipe| Material::new(base).on(State::Pressed, sink()).on(State::FocusVisible, ring()).on(State::Disabled, off()).on(State::Loading, busy());

        let well_base = Recipe::new().fill(c.well).inner_shadow(1.0, c.well_blur, c.well_shadow);
        let well = Material::new(well_base.clone())
            .on(State::FocusVisible, Delta::new().ring(k.ring, 0.0, c.focus))
            .on(State::Open, Delta::new().ring(k.ring, 0.0, c.focus))
            .on(State::Invalid, Delta::new().ring(k.ring, 0.0, c.danger_ring))
            .on(State::Disabled, off());

        let raised_base = Recipe::new().gradient(c.raised_top, c.raised_bottom).inner_top_edge(c.raised_edge).shadows(&e.xs);
        let raised = control(raised_base.clone()).on(State::Selected, Delta::replace(well_base.clone()));

        let ink = control(Recipe::new().gradient(c.ink_top, c.ink_bottom).inner_rim(c.ink_rim).inner_top_edge(c.ink_edge).shadows(&e.xs));

        let danger = control(Recipe::new().gradient(mix(c.danger, Color::WHITE, 0.14), c.danger).inner_rim(mix(c.danger, Color::BLACK, 0.35).alpha(0.55)).inner_top_edge(Color::WHITE.alpha(0.22)).shadows(&e.xs));

        let floating = Material::new(Recipe::new().fill(c.floating).shadows(&e.md).shadow(Shadow { spread: 1.0, color: c.float_ring, ..Shadow::default() }));

        let slab = Material::new(Recipe::new().fill(c.surface));

        let ghost = Material::new(Recipe::new())
            .on(State::Hover, Delta::new().wash(c.hover))
            .on(State::Pressed, Delta::new().wash(c.hover).wash(c.pressed).scale(m.press_scale))
            .on(State::FocusVisible, ring())
            .on(State::Disabled, off())
            .on(State::Selected, Delta::new().fill(c.selected));

        let row = Material::new(Recipe::new())
            .on(State::Hover, Delta::new().wash(c.hover))
            .on(State::Pressed, Delta::new().wash(c.hover).wash(c.pressed))
            .on(State::FocusVisible, Delta::new().ring(k.ring, -k.ring, c.focus))
            .on(State::Disabled, off())
            .on(State::Selected, Delta::new().fill(c.selected));

        let outline = control(Recipe::new().inner_rim(c.line.alpha((c.line.a * 2.0).min(1.0))))
            .on(State::Hover, Delta::new().wash(c.hover))
            .on(State::Pressed, Delta::new().wash(c.hover).wash(c.pressed).scale(m.press_scale))
            .on(State::Selected, Delta::replace(well_base.clone()));

        let thumb = Material::new(Recipe::new().gradient(c.thumb, mix(c.thumb, c.raised_bottom, 0.6)).inner_top_edge(c.raised_edge).shadow(Shadow { y: 1.0, blur: 3.0, color: c.shade.alpha(0.16), ..Shadow::default() }).shadow(Shadow { blur: 1.0, color: c.shade.alpha(0.08), ..Shadow::default() })).on(State::Disabled, off());

        let lift = Shadow { y: 6.0, blur: 14.0, spread: -5.0, color: c.shade.alpha(0.22), ..Shadow::default() };
        let chip = control(raised_base.shadow(lift)).on(State::Selected, Delta::replace(well_base));

        let atmosphere = Recipe::new().fill(c.surface).grain(c.grain);

        Materials { raised, ink, danger, well, floating, slab, ghost, row, outline, thumb, chip, atmosphere }
    }

    /// Every material by name, for galleries and tests.
    pub fn all(&self) -> [(&'static str, &Material); 11] {
        [("raised", &self.raised), ("ink", &self.ink), ("danger", &self.danger), ("well", &self.well), ("floating", &self.floating), ("slab", &self.slab), ("ghost", &self.ghost), ("row", &self.row), ("outline", &self.outline), ("thumb", &self.thumb), ("chip", &self.chip)]
    }
}
