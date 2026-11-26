//! A [`Theme`]: one mode of a theme file resolved into what components read: colour roles ([`Palette`]), the scales and the materials.
//! Nothing outside a theme names a colour.

use rune::{Color, TextStyle};

use crate::materials::{MaterialColors, Materials};
use crate::scale::{Controls, Elevation, Metrics, Motion, Radius, Space, TypeScale, tabular, weight};
use crate::tokens::{Corners, Font, Role, ThemeFile, Tone};

/// The colour roles. Every surface role carries the text colour that reads on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// The window behind everything; `fg` is the first ink.
    pub page: Role,
    /// A group's slab, a step off the page.
    pub surface: Role,
    /// Menus, popovers, dialogs.
    pub floating: Role,
    /// A quiet tint; `fg` is the second ink.
    pub muted: Role,
    /// A quieter tint; `fg` is the third ink (hints, placeholders; still 3:1).
    pub subtle: Role,
    /// The ink: the primary action's surface and its words.
    pub primary: Role,
    /// The raised material's surface and its words.
    pub secondary: Role,
    /// Selection and links.
    pub accent: Role,
    /// A well's tint and the text typed into it.
    pub input: Role,
    pub info: Tone,
    pub success: Tone,
    pub warning: Tone,
    pub danger: Tone,
    /// An identity colour for one thing's own hue (a progress ring, a selected mark): a quiet tint, its text, a solid colour and
    /// its text. `None` when the theme file sets no identity role; read it through [`Theme::identity`], which falls back to `info`.
    pub identity: Option<Tone>,
    /// A hairline between rows.
    pub line: Color,
    /// The keyboard focus ring.
    pub ring: Color,
    /// State layers: the page text colour at the theme's alphas.
    pub hover: Color,
    pub pressed: Color,
    pub selected: Color,
}

/// Everything a component reads.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub dark: bool,
    /// How rounded corners are drawn (`base::Shape::round_as` applies it to every rounded surface). Continuous corners draw the
    /// ladder [`crate::tokens::CONTINUOUS_GAIN`] times larger, so `radius` already includes it.
    pub corners: Corners,
    /// The theme file's `base.radius`, before the corner style's gain: what an editor shows.
    pub base_radius: f32,
    pub font: Font,
    pub c: Palette,
    pub space: Space,
    pub ty: TypeScale,
    pub radius: Radius,
    pub elevation: Elevation,
    pub motion: Motion,
    pub control: Controls,
    /// Spacing that follows the density knob (paddings, gaps, row heights, insets).
    pub metrics: Metrics,
    pub mat: Materials,
    /// The person asked for less motion (macOS Reduce Motion). Not in the theme file: the app sets it from Rune's
    /// `Cx::is_reduced_motion` each frame (`Theme::with_reduced_motion`). Components read it to draw their still form: no sweeping
    /// shimmer, no turning spinner, no fade-in of streaming words, no sliding thumb, overlays that appear in place. The state stays
    /// visible (a busy flag, a fixed arc, the words).
    pub reduced_motion: bool,
    /// How strongly black dims the page behind a modal (the theme file's `number.scrim`; 0.18 in light and 0.5 in dark when absent).
    pub scrim_strength: f32,
    /// The text style of keycaps and menu shortcuts: the theme's `keys-font` (its own font when unset) at the hint size, medium,
    /// with figures of one width.
    pub keycap: TextStyle,
    kit: rune_kit::Kit,
}

/// Why a theme file could not be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeError(pub Vec<String>);

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "theme file has {} problem(s):\n  {}", self.0.len(), self.0.join("\n  "))
    }
}

impl std::error::Error for ThemeError {}

impl Theme {
    /// One mode of a theme file's JSON.
    pub fn from_json(json: &str, dark: bool) -> Result<Theme, ThemeError> {
        ThemeFile::parse(json).map(|f| Theme::from_file(&f, dark)).map_err(ThemeError)
    }

    /// One mode of a parsed theme file.
    pub fn from_file(f: &ThemeFile, dark: bool) -> Theme {
        let m = f.mode(dark);
        let r = |k: &str| m.role(k);
        let ink = r("page").fg;
        let c = Palette {
            page: r("page"),
            surface: r("surface"),
            floating: r("floating"),
            muted: r("muted"),
            subtle: r("subtle"),
            primary: r("primary"),
            secondary: r("secondary"),
            accent: r("accent"),
            input: r("input"),
            info: m.tone("info"),
            success: m.tone("success"),
            warning: m.tone("warning"),
            danger: m.tone("danger"),
            identity: m.identity(),
            line: m.line,
            ring: m.ring,
            hover: ink.alpha(m.state("hover")),
            pressed: ink.alpha(m.state("pressed")),
            selected: ink.alpha(m.state("selected")),
        };
        let shade = m.component("shade");
        let elevation = Elevation::new(shade, m.number("elevation"));
        let motion = Motion::DEFAULT;
        let control = Controls::from_density(f.base.density, m.state("disabled"));
        // Continuous corners are drawn on a larger ladder, or they would look the same as round ones (`CONTINUOUS_GAIN`).
        let radius = Radius::from_base(f.base.radius * f.base.corners.gain());
        let space = Space { unit: f.base.spacing };
        let ty = TypeScale::for_density(f.base.density).scaled(f.base.type_scale, f.base.font.family).fitted(&f.base.font);
        let metrics = Metrics::from_density(f.base.density);
        let colors = MaterialColors {
            raised_top: m.component("raised-top"),
            raised_bottom: m.component("raised-bottom"),
            raised_edge: m.component("raised-edge"),
            ink_top: m.component("ink-top"),
            ink_bottom: m.component("ink-bottom"),
            ink_rim: m.component("ink-rim"),
            ink_edge: m.component("ink-edge"),
            danger: c.danger.solid,
            danger_ring: c.danger.solid.alpha(0.75),
            well: c.input.bg,
            well_shadow: m.component("well-shadow"),
            well_blur: m.number("well-blur"),
            surface: c.surface.bg,
            floating: c.floating.bg,
            float_ring: m.component("float-ring"),
            line: c.line,
            hover: c.hover,
            pressed: c.pressed,
            selected: c.selected,
            thumb: m.component("thumb"),
            shade,
            focus: c.ring,
            grain: m.number("grain"),
            texture: m.number("texture"),
        };
        let mat = Materials::derive(&colors, &elevation, &motion, &control);
        let keys_font = f.base.keys_font.unwrap_or(f.base.font);
        let keycap = tabular(weight(TypeScale::for_density(f.base.density).scaled(f.base.type_scale, keys_font.family).fitted(&keys_font).sm, ty.medium));
        let scrim_strength = m.number_opt("scrim").unwrap_or(if dark { 0.5 } else { 0.18 });
        let mut t = Theme { name: f.name.clone(), dark, corners: f.base.corners, base_radius: f.base.radius, font: f.base.font, c, space, ty, radius, elevation, motion, control, metrics, mat, reduced_motion: false, scrim_strength, keycap, kit: rune_kit::Kit::neutral(dark) };
        t.kit = t.build_kit();
        t
    }

    /// This theme with Reduce Motion on or off: `theme.with_reduced_motion(cx.is_reduced_motion())` in the view.
    pub fn with_reduced_motion(mut self, on: bool) -> Theme {
        self.reduced_motion = on;
        self
    }

    /// The rune-kit tokens for components whose behaviour is rune-kit's (Select, Combobox, DropdownMenu, ContextMenu): our floating
    /// surface and its shadows, no border (the shadow's baked ring is the edge), rows in the base size and the first ink, the hover wash
    /// under the pointer or the keys. This is the adapter of `ARCHITECTURE.md`'s behaviour/paint seam.
    pub fn to_kit(&self) -> &rune_kit::Kit {
        &self.kit
    }

    fn build_kit(&self) -> rune_kit::Kit {
        let c = &self.c;
        let mut k = rune_kit::Kit::from_palette(rune_kit::PaletteSeed { bg: c.page.bg, fg: c.page.fg, accent: c.accent.fg });
        let floating: Vec<rune::Shadow> = crate::paint::plan(&self.mat.floating.base).into_iter().filter_map(|op| if let crate::Op::BoxShadow(s) = op { Some(s) } else { None }).collect();
        k.c.surface_raised = c.floating.bg;
        k.c.surface_float = c.floating.bg;
        k.c.layer_01 = c.floating.bg;
        k.c.surface_raised_hover = c.hover;
        k.c.hover = c.hover;
        k.c.pressed = c.pressed;
        // rune-kit draws this at half strength round a raised surface: the surface's own colour leaves only the shadow's ring as the edge.
        k.c.std_border = c.floating.bg;
        k.c.std_icon = c.muted.fg;
        k.c.icon_base = c.muted.fg;
        k.c.icon_muted = c.subtle.fg;
        k.c.text_strong = c.page.fg;
        k.c.text_base = c.page.fg;
        k.c.text_weak = c.muted.fg;
        k.c.text_faint = c.subtle.fg;
        k.c.border_focus = c.ring;
        k.elevation.md = floating.clone();
        k.elevation.floating = floating;
        k.elevation.raised = self.elevation.sm.clone();
        k.elevation.button_neutral = Vec::new();
        k.radius.md = self.radius.lg;
        k.radius.sm = Radius::inner(self.radius.lg, 4.0);
        k.ty.ui_medium = self.ty.base;
        // A standard menu row is its text's line plus 4 above and below: the line makes the row the density's row height.
        k.ty.text_small_medium = TextStyle { line_height: self.metrics.row - 8.0, ..self.ty.base };
        k.ty.ui_compact = self.ty.base;
        k.ty.text_xsmall = self.ty.sm;
        k.ty.text_xsmall_medium = weight(self.ty.sm, self.ty.medium);
        k.ty.small_medium = tabular(self.ty.sm);
        k
    }

    /// The control text style for a size.
    pub fn control_text(&self, sm: bool, w: f32) -> TextStyle {
        weight(if sm { self.ty.sm } else { self.ty.base }, w)
    }

    /// The identity colours: the theme's `identity` role, or its `info` meaning where the theme sets none, so a component can always
    /// draw with them. It never changes the ink (`primary`).
    pub fn identity(&self) -> Tone {
        self.c.identity.unwrap_or(self.c.info)
    }

    /// Whether the theme file sets an identity role of its own.
    pub fn has_identity(&self) -> bool {
        self.c.identity.is_some()
    }

    /// What dims the page behind a modal (a dialog, a sheet, the command palette): black at the theme's strength. Black in both
    /// modes: the page ink would fog a dark page with a pale veil instead of dimming it.
    pub fn scrim(&self) -> Color {
        Color::BLACK.alpha(self.scrim_strength)
    }

    /// A lighter scrim, for a palette that sits over the page without shutting it away.
    pub fn scrim_soft(&self) -> Color {
        Color::BLACK.alpha(self.scrim_strength * 0.7)
    }

    /// The first, second and third inks.
    pub fn ink(&self) -> Color {
        self.c.page.fg
    }
    pub fn ink_muted(&self) -> Color {
        self.c.muted.fg
    }
    pub fn ink_faint(&self) -> Color {
        self.c.subtle.fg
    }
}
