//! # rune-ui-style
//!
//! Layer 2 of rune-ui (README.md): the style system that components are built from, the way Tailwind is for the web.
//!
//! * [`scale`]: space, type, radius, elevation (smooth shadow stacks) and motion, as short ladders of named steps;
//!   [`Controls`] for control heights and hit areas.
//! * [`theme`]: [`ThemeData`] (hex strings and a few numbers; themes ship as this) resolved into a [`Theme`]: colour roles
//!   ([`Palette`]), the scales, and the [`Materials`].
//! * [`recipe`]: a surface as data. A [`Recipe`] is an ordered list of [`Layer`]s (fill, gradient, lit top edge, inner shadow,
//!   inner rim, drop shadows, ring, wash, scale, opacity, backdrop blur, grain); a [`Material`] has a recipe for each [`State`].
//! * [`paint`]: the one painter. [`plan`] resolves a recipe into Rune calls ([`Op`]); [`paint()`] makes them on an `El`;
//!   [`paint_material`] adds Rune's live hover, press and focus.
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
pub mod scale;
pub mod theme;

pub use color::{contrast, hex, lightness, mix, over};
pub use materials::{MaterialColors, Materials};
pub use paint::{Look, Op, apply, apply_live, paint, paint_material, plan, resolve};
pub use recipe::{Layer, LayerKind, Live, Material, Recipe, State};
pub use scale::{Controls, Elevation, MAX_CORNER_SHARE, Motion, Radius, Space, TypeScale, smooth_stack, tabular, weight};
pub use theme::{Palette, Theme, ThemeData, Tone, ToneData};
