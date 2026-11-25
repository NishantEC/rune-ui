//! The materials: the few surfaces everything is made of, built from a theme's colours as recipes.
//!
//! * **Raised** — anything you press. Lighter at the top than the bottom, a lit top edge, a contact shadow so it sits on the page.
//!   It answers a press by sinking (the scale), not by changing colour. Selected, it stays down: it becomes a well.
//! * **Ink** — the one primary action in a view. The raised recipe in the darkest (light theme) or lightest (dark theme) colour,
//!   glossier, with its own inner rim so it reads as one solid object.
//! * **Danger** — the ink recipe in the danger colour, for what cannot be taken back.
//! * **Well** — anything you put something into, or that holds a value: a flat tint pressed into the page, a soft shadow inside its
//!   top edge. Focus is a ring around it, never a border.
//! * **Floating** — the only things with a real drop shadow: menus, popovers, dialogs. A smooth stack, with a hairline ring baked in
//!   as its last layer so the edge melts into the shadow instead of being drawn.
//! * **Slab** — a group's surface: one fill a step off the page, no edge, no shadow. Rows inside are split by hairlines.
//! * **Ghost** — nothing until hovered (toolbar and row actions).
//! * **Thumb** — the raised disc a track holds (a switch, a segmented control's chosen segment).
//! * **Chip** — raised, and the one control that lifts off the page with a soft shadow.
//!
//! Tracks are wells holding a raised thumb, so the two directions explain the control without words: up means press me, down means
//! put something in me.

use rune::{Color, Shadow};

use crate::color::mix;
use crate::recipe::{Material, Recipe};
use crate::scale::{Controls, Elevation, Motion};

/// The colours the materials are made from (a theme's data, resolved).
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
    pub well: Color,
    pub well_shadow: Color,
    /// How soft the well's inner shadow is (CSS blur).
    pub well_blur: f32,
    pub surface: Color,
    pub surface_hover: Color,
    pub floating: Color,
    /// The hairline ring baked into floating shadows.
    pub float_ring: Color,
    pub hover: Color,
    pub pressed: Color,
    pub selected: Color,
    pub thumb: Color,
    pub shade: Color,
    pub focus: Color,
    /// Grain over atmosphere surfaces (0 = none). Rune cannot draw it yet; recipes carry it so a later painter can.
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
    pub thumb: Material,
    pub chip: Material,
    /// A large surface with atmosphere (a home page's backdrop): the slab with grain. Never on controls.
    pub atmosphere: Recipe,
}

impl Materials {
    pub fn derive(c: &MaterialColors, e: &Elevation, m: &Motion, k: &Controls) -> Materials {
        let ring = |r: Recipe| r.ring(k.ring, k.ring_offset, c.focus);
        let sink = |r: Recipe| r.shadow_none().scale(m.press_scale);
        let off = |r: Recipe| r.shadow_none().opacity(k.disabled);

        let well_rest = Recipe::new().fill(c.well).inner_shadow(1.0, c.well_blur, c.well_shadow);
        let well = Material::new(well_rest.clone())
            // A well shows focus by a ring that hugs it (no gap): it is a field, not a button.
            .focused(|r| r.ring(k.ring, 0.0, c.focus))
            .disabled(off)
            .selected(well_rest.clone());

        let raised_rest = Recipe::new().gradient(c.raised_top, c.raised_bottom).inner_top_edge(c.raised_edge).shadows(&e.xs);
        let raised = Material::new(raised_rest.clone()).pressed(sink).focused(ring).disabled(off).selected(well_rest.clone());

        let ink_rest = Recipe::new().gradient(c.ink_top, c.ink_bottom).inner_rim(c.ink_rim).inner_top_edge(c.ink_edge).shadows(&e.xs);
        let ink = Material::new(ink_rest).pressed(sink).focused(ring).disabled(off);

        let danger_rest = Recipe::new().gradient(mix(c.danger, Color::WHITE, 0.14), c.danger).inner_rim(mix(c.danger, Color::BLACK, 0.35).alpha(0.55)).inner_top_edge(Color::WHITE.alpha(0.22)).shadows(&e.xs);
        let danger = Material::new(danger_rest).pressed(sink).focused(ring).disabled(off);

        let float_rest = Recipe::new().fill(c.floating).shadows(&e.md).shadow(Shadow { spread: 1.0, color: c.float_ring, ..Shadow::default() });
        let floating = Material::new(float_rest);

        let slab = Material::new(Recipe::new().fill(c.surface)).hover(|_| Recipe::new().fill(c.surface_hover));

        let ghost = Material::new(Recipe::new())
            .hover(|r| r.wash(c.hover))
            .pressed(|r| r.wash(c.hover).wash(c.pressed).scale(m.press_scale))
            .focused(ring)
            .disabled(off)
            .selected(Recipe::new().fill(c.selected));

        let thumb_rest = Recipe::new().gradient(c.thumb, mix(c.thumb, c.raised_bottom, 0.6)).inner_top_edge(c.raised_edge).shadow(Shadow { y: 1.0, blur: 3.0, color: c.shade.alpha(0.16), ..Shadow::default() }).shadow(Shadow { blur: 1.0, color: c.shade.alpha(0.08), ..Shadow::default() });
        let thumb = Material::new(thumb_rest).disabled(off);

        let lift = Shadow { y: 6.0, blur: 14.0, spread: -5.0, color: c.shade.alpha(0.22), ..Shadow::default() };
        let chip_rest = raised_rest.clone().shadow(lift);
        let chip = Material::new(chip_rest).pressed(sink).focused(ring).disabled(off).selected(well_rest);

        let atmosphere = Recipe::new().fill(c.surface).grain(c.grain);

        Materials { raised, ink, danger, well, floating, slab, ghost, thumb, chip, atmosphere }
    }

    /// Every material by name, for galleries and tests.
    pub fn all(&self) -> [(&'static str, &Material); 9] {
        [("raised", &self.raised), ("ink", &self.ink), ("danger", &self.danger), ("well", &self.well), ("floating", &self.floating), ("slab", &self.slab), ("ghost", &self.ghost), ("thumb", &self.thumb), ("chip", &self.chip)]
    }
}
