//! rune-ui registry: context_menu 0.3.0. Copied into your app by `rune-ui add context_menu`; yours to change.
//!
//! A menu where the pointer pressed with the secondary button. Behaviour is rune-kit's `context_menu` (it opens at the point, flips to
//! stay in the window, holds the keys while open); our look comes through `Theme::to_kit()`. Store the point from
//! `El::on_secondary_press` in your state, and clear it in `on_close` and in each item's handler.
//!
//! ```ignore
//! ContextMenu::new("file-menu", area, s.menu_at).item(ContextMenuItem::new("Rename")).on_close(|s: &mut App| s.menu_at = None).build(t, &mut layer)
//! ```

use rune::{El, PointerPress};
pub use rune_kit::MenuItem as ContextMenuItem;

use super::base::Theme;

pub struct ContextMenu {
    inner: rune_kit::Menu,
}

impl ContextMenu {
    /// A context menu over `area`, open at `point` (closed while it is none).
    pub fn new(key: impl Into<String>, area: El, point: Option<PointerPress>) -> ContextMenu {
        ContextMenu { inner: rune_kit::context_menu(key, area, point).standard() }
    }

    /// Open at a window point you computed (a keyboard "menu" key, a picture).
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.inner = self.inner.open(true).at(x, y);
        self
    }

    pub fn item(mut self, i: ContextMenuItem) -> Self {
        self.inner = self.inner.item(i);
        self
    }

    pub fn danger_item(mut self, i: ContextMenuItem) -> Self {
        self.inner = self.inner.separator().item(i.danger(true));
        self
    }

    pub fn separator(mut self) -> Self {
        self.inner = self.inner.separator();
        self
    }

    pub fn label(mut self, words: impl Into<String>) -> Self {
        self.inner = self.inner.group_label(words);
        self
    }

    pub fn on_close<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.inner = self.inner.on_close(f);
        self
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        self.inner.build(t.to_kit(), layer)
    }
}
