//! rune-ui registry: context_menu 0.6.0. Copied into your app by `rune-ui add context_menu`; yours to change. Needs: popup.
//!
//! A menu where the pointer pressed with the secondary button: rune-ui's popup with role `menu`, opening at that point, flipping to stay
//! in the window, closing on Escape, Tab, a press outside or a choice, focus going back to the area. Store the point from
//! `El::on_secondary_press` in your state and clear it in `on_close`.
//!
//! ```ignore
//! ContextMenu::new("file-menu", area, s.menu_at).item(ContextMenuItem::new("Rename")).on_close(|s: &mut App| s.menu_at = None).build(t, &mut layer)
//! ```

use rune::{El, PointerPress, Side, row};

use super::base::{Press, Theme, press};
use super::popup::{Entry, Popup, Row, Mount, mount, point_anchor};

/// A menu row (see `popup::Row`).
pub type ContextMenuItem<S> = Row<S>;

pub struct ContextMenu<S: 'static> {
    key: String,
    area: El,
    point: Option<PointerPress>,
    entries: Vec<Entry<S>>,
    close: Option<Press<S>>,
}

impl<S: 'static> ContextMenu<S> {
    /// A context menu over `area`, open at `point` (closed while it is none).
    pub fn new(key: impl Into<String>, area: El, point: Option<PointerPress>) -> ContextMenu<S> {
        ContextMenu { key: key.into(), area, point, entries: Vec::new(), close: None }
    }

    pub fn item(mut self, i: ContextMenuItem<S>) -> Self {
        self.entries.push(Entry::Row(i));
        self
    }

    pub fn danger_item(mut self, i: ContextMenuItem<S>) -> Self {
        self.entries.push(Entry::Separator);
        self.entries.push(Entry::Row(i.danger(true)));
        self
    }

    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    pub fn label(mut self, words: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(words.into()));
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(press(f));
        self
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let area_key = format!("{}-area", self.key);
        let area = row().key(area_key.clone()).focusable(true).label("Context menu area").child(self.area);
        if let Some(p) = self.point {
            let close = self.close.unwrap_or_else(|| press(|_: &mut S| {}));
            let anchor = format!("{}-point", self.key);
            layer.push(point_anchor(&anchor, p.x, p.y));
            let mut popup = Popup::<S>::new(self.key.clone(), "menu", "Context menu");
            popup.min_w = 200.0;
            popup.entries = self.entries;
            let surface = popup.surface(t, close.clone());
            mount(t, layer, surface, Mount { key: &self.key, anchor: &anchor, side: Side::Bottom, return_to: &area_key }, close);
        }
        area
    }
}
