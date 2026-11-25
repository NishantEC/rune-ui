//! The registry's sources, compiled exactly as an app that ran `rune-ui init` and `rune-ui add …` compiles its copies in `src/ui/`.
//! The gallery points at the registry instead of copying it, so every change to a component is built and pictured here first.

#[path = "../../../../registry/base.rs"]
pub mod base;
#[path = "../../../../registry/icons.rs"]
pub mod icons;

#[path = "../../../../registry/badge.rs"]
pub mod badge;
#[path = "../../../../registry/button.rs"]
pub mod button;
#[path = "../../../../registry/card.rs"]
pub mod card;
#[path = "../../../../registry/chip.rs"]
pub mod chip;
#[path = "../../../../registry/segmented.rs"]
pub mod segmented;
#[path = "../../../../registry/select.rs"]
pub mod select;
#[path = "../../../../registry/switch.rs"]
pub mod switch;
#[path = "../../../../registry/text_field.rs"]
pub mod text_field;
