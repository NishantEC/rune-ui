//! # rune-ui-style
//!
//! Layer 2 of rune-ui (`ARCHITECTURE.md`): the style system components are built from, the way Tailwind is for the web.
//!
//! * [`tokens`]: theme files. Three tiers (ramps, roles with a matching text colour, rare component tokens), state alphas, three base
//!   knobs (`spacing`, `radius`, `density`), a light and a dark mode, parsed strictly (unknown keys are errors).
//! * [`scale`]: space, type, radius, elevation (smooth shadow stacks), motion and control sizes.
//! * [`theme`]: a [`Theme`] (one mode resolved: [`Palette`], scales, [`Materials`]) and [`Theme::to_kit`], the rune-kit adapter.
//! * [`recipe`]: [`Recipe`] (layers), [`Delta`], [`Material`] (a delta per [`State`]), [`Parts`], and the one resolver [`resolve`]
//!   (disabled > loading > pressed > hover > rest; selected under, focus, invalid and open on top).
//! * [`recipes`]: component variants as enums ([`ButtonVariant`], [`Size`], [`Tone`]) and their parts.
//! * [`paint`]: the one painter: [`plan`] lists the Rune calls ([`Op`]), [`paint()`] makes them, [`paint_state`] adds Rune's live states.
//!
//! ```
//! use rune::{Color, Shadow, el};
//! use rune_ui_style::{Op, Recipe, paint, plan};
//!
//! let raised = Recipe::new()
//!     .gradient(Color::WHITE, Color::WHITE.alpha(0.92))
//!     .inner_top_edge(Color::WHITE)
//!     .shadow(Shadow { y: 1.0, blur: 2.0, color: Color::BLACK.alpha(0.06), ..Shadow::default() });
//! assert!(matches!(plan(&raised)[0], Op::Gradient { .. }));
//! let button = paint(el().size(80.0, 30.0).radius(8.0), &raised);
//! # let _ = button;
//! ```

pub mod color;
pub mod materials;
pub mod paint;
pub mod recipe;
pub mod recipes;
pub mod scale;
pub mod theme;
pub mod tokens;

pub use color::{contrast, hex, lightness, mix, over};
pub use materials::{MaterialColors, Materials};
pub use paint::{Op, apply, apply_live, paint, paint_state, plan};
pub use recipe::{Delta, Layer, LayerKind, Live, Material, Parts, Recipe, State, States, Tint, resolve};
pub use recipes::{ButtonVariant, Size, Tone};
pub use scale::{Controls, Elevation, MAX_CORNER_SHARE, Motion, Radius, Space, TypeScale, smooth_stack, tabular, weight};
pub use theme::{Palette, Theme, ThemeError};
pub use tokens::{Base, Mode, Role, ThemeFile};

/// This crate's version, which registry items name in `requires.style`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
