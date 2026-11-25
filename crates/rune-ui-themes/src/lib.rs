//! # rune-ui-themes
//!
//! Layer 4 of rune-ui: themes, as data only. A theme is a [`ThemeData`]: hex strings and a few numbers. No code decides a colour
//! here; `rune_ui_style::Theme::from_data` turns the data into colour roles and material recipes.
//!
//! Two themes ship: [`LIGHT`] and [`DARK`]. Dark is graphite, not navy, and its materials are not an inverted copy of light: the
//! raised gradient is narrower, the lit edge much dimmer, inner shadows and drop shadows deeper, the ink a near white with a light
//! rim, the floating ring light instead of dark (a dark ring vanishes on a dark page).
//!
//! To add a theme, copy one of these, change the values, and run this crate's tests: they check text contrast and that every
//! material still reads in the right direction (raised lighter on top, wells below the page, ink darkest or lightest).

pub use rune_ui_style::{Theme, ThemeData, ToneData};

/// The light theme: an almost colourless page, raised controls lit from above, one ink object per view.
pub const LIGHT: ThemeData = ThemeData {
    name: "light",
    dark: false,
    page: "#f8f8f7",
    surface: "#efefed",
    surface_hover: "#e8e8e6",
    floating: "#ffffff",
    ink: "#1d1d20",
    ink_muted: "#5c5c63",
    ink_faint: "#86868d",
    on_ink: "#fbfbfb",
    line: "#00000012",
    focus: "#85858d",
    accent: "#3a6fd8",
    info: ToneData { ink: "#2c5fc4", dot: "#3f7ae6" },
    success: ToneData { ink: "#1f6e42", dot: "#2fa865" },
    warning: ToneData { ink: "#8f5c06", dot: "#e0a030" },
    danger: ToneData { ink: "#b3372f", dot: "#d9483e" },
    tone_wash: 0.13,
    raised_top: "#fdfdfd",
    raised_bottom: "#efeff0",
    raised_edge: "#ffffffeb",
    ink_top: "#47474d",
    ink_bottom: "#222226",
    ink_rim: "#0000007a",
    ink_edge: "#ffffff2e",
    well: "#0000000d",
    well_shadow: "#0000001c",
    well_blur: 3.0,
    hover: "#0000000a",
    pressed: "#00000008",
    selected: "#00000010",
    thumb: "#ffffff",
    shade: "#000000",
    float_ring: "#0000000f",
    elevation: 1.0,
    grain: 0.06,
};

/// The dark theme: graphite, with the materials retuned rather than inverted.
pub const DARK: ThemeData = ThemeData {
    name: "dark",
    dark: true,
    page: "#141416",
    surface: "#1b1b1e",
    surface_hover: "#232327",
    floating: "#232327",
    ink: "#ededef",
    ink_muted: "#a6a6ad",
    ink_faint: "#7a7a82",
    on_ink: "#161618",
    line: "#ffffff14",
    focus: "#80808a",
    accent: "#7aa2f0",
    info: ToneData { ink: "#8db2f5", dot: "#5d8ff0" },
    success: ToneData { ink: "#6fd29b", dot: "#3fbf78" },
    warning: ToneData { ink: "#f0c470", dot: "#e8a93a" },
    danger: ToneData { ink: "#f19189", dot: "#e5584d" },
    tone_wash: 0.16,
    raised_top: "#2b2b2f",
    raised_bottom: "#242427",
    raised_edge: "#ffffff14",
    ink_top: "#fafafa",
    ink_bottom: "#e2e2e5",
    ink_rim: "#ffffff59",
    ink_edge: "#ffffffff",
    well: "#ffffff0f",
    well_shadow: "#00000066",
    well_blur: 3.0,
    hover: "#ffffff0d",
    pressed: "#ffffff08",
    selected: "#ffffff14",
    thumb: "#c4c4ca",
    shade: "#000000",
    float_ring: "#ffffff1a",
    elevation: 3.2,
    grain: 0.1,
};

/// Every shipped theme.
pub const ALL: [ThemeData; 2] = [LIGHT, DARK];

/// The shipped theme for an appearance.
pub fn theme(dark: bool) -> Theme {
    Theme::from_data(if dark { &DARK } else { &LIGHT })
}

/// A shipped theme by name.
pub fn by_name(name: &str) -> Option<Theme> {
    ALL.iter().find(|d| d.name == name).map(Theme::from_data)
}
