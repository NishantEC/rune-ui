//! rune-ui registry: dropdown_menu 0.2.0. Copied into your app by `rune-ui add dropdown_menu`; yours to change.
//!
//! A menu under its trigger. Behaviour is rune-kit's `Menu` (arrow keys, typeahead, submenus, Escape and a press outside, placement kept
//! in the window), handed our look through `Theme::to_kit()`: the **floating** material, rows that take the hover wash, a quiet group
//! label, separators, shortcuts at the right, the danger colour for what cannot be taken back. Radii nest: rows sit four points in.
//!
//! ```ignore
//! DropdownMenu::new("account").trigger(button).open(s.open == Some("account")).on_close(|s: &mut App| s.open = None)
//!     .label("Account").item(DropdownMenuItem::new("Settings").shortcut("⌘,")).separator().danger_item(DropdownMenuItem::new("Sign out"))
//!     .build(t, &mut layer)
//! ```

use rune::El;
pub use rune_kit::{MenuItem as DropdownMenuItem, Placement};

use super::base::Theme;

pub struct DropdownMenu {
    inner: rune_kit::Menu,
}

impl DropdownMenu {
    /// Rows are keyed `<key>-item<N>` (separators and labels count).
    pub fn new(key: impl Into<String>) -> DropdownMenu {
        DropdownMenu { inner: rune_kit::Menu::new(key).standard() }
    }

    pub fn trigger(mut self, t: El) -> Self {
        self.inner = self.inner.trigger(t);
        self
    }

    pub fn item(mut self, i: DropdownMenuItem) -> Self {
        self.inner = self.inner.item(i);
        self
    }

    /// A destructive row, set apart by a separator.
    pub fn danger_item(mut self, i: DropdownMenuItem) -> Self {
        self.inner = self.inner.separator().item(i.danger(true));
        self
    }

    pub fn separator(mut self) -> Self {
        self.inner = self.inner.separator();
        self
    }

    /// A quiet group label.
    pub fn label(mut self, words: impl Into<String>) -> Self {
        self.inner = self.inner.group_label(words);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.inner = self.inner.open(open);
        self
    }

    pub fn placement(mut self, p: Placement) -> Self {
        self.inner = self.inner.placement(p);
        self
    }

    pub fn min_width(mut self, w: f32) -> Self {
        self.inner = self.inner.min_width(w);
        self
    }

    pub fn on_close<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.inner = self.inner.on_close(f);
        self
    }

    /// The trigger in place; the open menu goes into `layer`.
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        self.inner.build(t.to_kit(), layer)
    }
}
