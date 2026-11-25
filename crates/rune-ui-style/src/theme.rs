//! A theme: [`ThemeData`] (hex strings and a few numbers, nothing else; layer 4 ships these) resolved into a [`Theme`] (colour roles,
//! the scales, and the materials as recipes) that components read. Nothing outside a theme names a colour.

use rune::{Color, TextStyle};

use crate::color::hex;
use crate::materials::{MaterialColors, Materials};
use crate::scale::{Controls, Elevation, Motion, Radius, Space, TypeScale, weight};

/// A meaning's two inks: the words and the dot. Its wash is the dot at the theme's `tone_wash` alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneData {
    pub ink: &'static str,
    pub dot: &'static str,
}

/// A theme as data. Colours are sRGB hex (`#rrggbb`, or `#rrggbbaa` for a colour at an alpha).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeData {
    pub name: &'static str,
    pub dark: bool,

    /// The window behind everything.
    pub page: &'static str,
    /// A group's slab, a step off the page.
    pub surface: &'static str,
    pub surface_hover: &'static str,
    /// What floats: menus, popovers, dialogs.
    pub floating: &'static str,
    /// Text: the first ink, the second (secondary), the third (hints; still 3:1 on the page).
    pub ink: &'static str,
    pub ink_muted: &'static str,
    pub ink_faint: &'static str,
    /// Words on the ink material.
    pub on_ink: &'static str,
    /// A hairline between rows.
    pub line: &'static str,
    /// The keyboard focus ring (neutral, not a hue).
    pub focus: &'static str,
    /// The one hue for selection and links.
    pub accent: &'static str,
    pub info: ToneData,
    pub success: ToneData,
    pub warning: ToneData,
    pub danger: ToneData,
    /// How strong a meaning's wash is behind its words (0 to 1).
    pub tone_wash: f32,

    pub raised_top: &'static str,
    pub raised_bottom: &'static str,
    pub raised_edge: &'static str,
    pub ink_top: &'static str,
    pub ink_bottom: &'static str,
    pub ink_rim: &'static str,
    pub ink_edge: &'static str,
    pub well: &'static str,
    pub well_shadow: &'static str,
    pub well_blur: f32,
    pub hover: &'static str,
    pub pressed: &'static str,
    pub selected: &'static str,
    pub thumb: &'static str,
    /// What shadows are made of.
    pub shade: &'static str,
    /// The hairline baked into floating shadows.
    pub float_ring: &'static str,
    /// Every shadow alpha is multiplied by this (a dark page needs deeper shadows to show them).
    pub elevation: f32,
    /// Grain over atmosphere surfaces (0 = none).
    pub grain: f32,
}

/// A meaning, resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub ink: Color,
    pub dot: Color,
    pub wash: Color,
}

/// The colour roles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub page: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub floating: Color,
    pub ink: Color,
    pub ink_muted: Color,
    pub ink_faint: Color,
    pub on_ink: Color,
    pub line: Color,
    pub focus: Color,
    pub accent: Color,
    pub hover: Color,
    pub pressed: Color,
    pub selected: Color,
    pub info: Tone,
    pub success: Tone,
    pub warning: Tone,
    pub danger: Tone,
}

/// Everything a component reads.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub dark: bool,
    pub c: Palette,
    pub space: Space,
    pub ty: TypeScale,
    pub radius: Radius,
    pub elevation: Elevation,
    pub motion: Motion,
    pub control: Controls,
    pub mat: Materials,
    kit: rune_kit::Kit,
}

impl Theme {
    pub fn from_data(d: &ThemeData) -> Theme {
        let tone = |t: ToneData| {
            let dot = hex(t.dot);
            Tone { ink: hex(t.ink), dot, wash: dot.alpha(d.tone_wash) }
        };
        let c = Palette {
            page: hex(d.page),
            surface: hex(d.surface),
            surface_hover: hex(d.surface_hover),
            floating: hex(d.floating),
            ink: hex(d.ink),
            ink_muted: hex(d.ink_muted),
            ink_faint: hex(d.ink_faint),
            on_ink: hex(d.on_ink),
            line: hex(d.line),
            focus: hex(d.focus),
            accent: hex(d.accent),
            hover: hex(d.hover),
            pressed: hex(d.pressed),
            selected: hex(d.selected),
            info: tone(d.info),
            success: tone(d.success),
            warning: tone(d.warning),
            danger: tone(d.danger),
        };
        let shade = hex(d.shade);
        let elevation = Elevation::new(shade, d.elevation);
        let (motion, control, radius, space, ty) = (Motion::DEFAULT, Controls::DEFAULT, Radius::DEFAULT, Space::DEFAULT, TypeScale::DEFAULT);
        let colors = MaterialColors {
            raised_top: hex(d.raised_top),
            raised_bottom: hex(d.raised_bottom),
            raised_edge: hex(d.raised_edge),
            ink_top: hex(d.ink_top),
            ink_bottom: hex(d.ink_bottom),
            ink_rim: hex(d.ink_rim),
            ink_edge: hex(d.ink_edge),
            danger: c.danger.dot,
            well: hex(d.well),
            well_shadow: hex(d.well_shadow),
            well_blur: d.well_blur,
            surface: c.surface,
            surface_hover: c.surface_hover,
            floating: c.floating,
            float_ring: hex(d.float_ring),
            hover: c.hover,
            pressed: c.pressed,
            selected: c.selected,
            thumb: hex(d.thumb),
            shade,
            focus: c.focus,
            grain: d.grain,
        };
        let mat = Materials::derive(&colors, &elevation, &motion, &control);
        let kit = kit_for(&c, &elevation, &radius, &ty, &mat);
        Theme { name: d.name, dark: d.dark, c, space, ty, radius, elevation, motion, control, mat, kit }
    }

    /// The rune-kit tokens for components whose behaviour is rune-kit's (a select's listbox, a menu): our floating surface and its
    /// shadows, no border (the shadow's baked ring is the edge), rows in the base size and the first ink, the hover wash under the
    /// pointer or the keys.
    pub fn kit(&self) -> &rune_kit::Kit {
        &self.kit
    }

    /// The control text style at a size (`sm` or not) and weight.
    pub fn control_text(&self, sm: bool, w: f32) -> TextStyle {
        weight(if sm { self.ty.sm } else { self.ty.base }, w)
    }
}

fn kit_for(c: &Palette, e: &Elevation, r: &Radius, ty: &TypeScale, mat: &Materials) -> rune_kit::Kit {
    let mut k = rune_kit::Kit::from_palette(rune_kit::PaletteSeed { bg: c.page, fg: c.ink, accent: c.accent });
    let floating: Vec<rune::Shadow> = crate::paint::plan(&mat.floating.rest).into_iter().filter_map(|op| if let crate::Op::BoxShadow(s) = op { Some(s) } else { None }).collect();
    k.c.surface_raised = c.floating;
    k.c.surface_float = c.floating;
    k.c.layer_01 = c.floating;
    k.c.surface_raised_hover = c.hover;
    k.c.hover = c.hover;
    k.c.pressed = c.pressed;
    // rune-kit draws this at half strength round a raised surface: the surface's own colour leaves only the shadow's ring as the edge.
    k.c.std_border = c.floating;
    k.c.std_icon = c.ink_muted;
    k.c.icon_base = c.ink_muted;
    k.c.text_strong = c.ink;
    k.c.text_base = c.ink;
    k.c.text_weak = c.ink_faint;
    k.c.text_faint = c.ink_faint;
    k.c.border_focus = c.focus;
    k.elevation.md = floating.clone();
    k.elevation.floating = floating;
    k.elevation.raised = e.sm.clone();
    k.radius.md = r.lg;
    k.radius.sm = Radius::inner(r.lg, 4.0);
    k.ty.ui_medium = ty.base;
    k.ty.text_small_medium = ty.base;
    k.ty.ui_compact = ty.base;
    k.ty.text_xsmall = ty.sm;
    k.ty.text_xsmall_medium = weight(ty.sm, ty.medium);
    k.ty.small_medium = crate::scale::tabular(ty.sm);
    k
}
