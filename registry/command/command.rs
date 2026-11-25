//! rune-ui registry: command 0.2.0. Copied into your app by `rune-ui add command`; yours to change. Needs: kbd.
//!
//! A command palette: a search over grouped commands. Typing filters (every word of the query must appear in the label); Up and Down
//! move the highlight, Return runs it, a press runs any row. The **floating** material; the search is a bare line over a hairline (the
//! panel is the well's frame); the highlighted row takes the selected wash; shortcuts are `Kbd` caps. `build` draws it inline;
//! `dialog` mounts it as a modal near the top of the window (Rune's `El::modal`: focus trapped, Escape and a press outside close it).
//!
//! ```ignore
//! Command::new("palette", &s.query, |s: &mut App| &mut s.query).items(&commands).highlight(s.at)
//!     .on_highlight(|s: &mut App, i| s.at = i).on_select(|s: &mut App, key: &str| s.run(key)).dialog(t, s.palette, |s: &mut App| s.palette = false, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Icon, Key, KeyEvent, TextEdit, col, el, row, text};
use rune_ui_style::weight;

use super::base::{States, Theme, glyph, paint, paint_state};
use super::icons;
use super::kbd::Kbd;

/// One command.
#[derive(Clone, Debug)]
pub struct CommandItem {
    pub key: String,
    pub label: String,
    pub group: String,
    pub icon: Option<&'static Icon>,
    pub shortcut: Option<String>,
}

impl CommandItem {
    pub fn new(key: impl Into<String>, label: impl Into<String>, group: impl Into<String>) -> CommandItem {
        CommandItem { key: key.into(), label: label.into(), group: group.into(), icon: None, shortcut: None }
    }

    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    pub fn shortcut(mut self, k: impl Into<String>) -> Self {
        self.shortcut = Some(k.into());
        self
    }
}

/// The items `query` keeps, in order: every word of the query appears in the label, ignoring case.
pub fn filter<'a>(items: &'a [CommandItem], query: &str) -> Vec<&'a CommandItem> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    items.iter().filter(|i| {
        let l = i.label.to_lowercase();
        words.iter().all(|w| l.contains(w.as_str()))
    }).collect()
}

pub struct Command<'a, S: 'static> {
    key: String,
    edit: &'a TextEdit,
    access: Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut TextEdit>,
    placeholder: String,
    items: &'a [CommandItem],
    highlight: usize,
    empty: String,
    width: f32,
    move_to: Option<Rc<dyn Fn(&mut S, usize)>>,
    select: Option<Rc<dyn Fn(&mut S, &str)>>,
}

impl<'a, S: 'static> Command<'a, S> {
    pub fn new(key: impl Into<String>, query: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> Command<'a, S> {
        Command { key: key.into(), edit: query, access: Rc::new(access), placeholder: "Type a command or search…".to_owned(), items: &[], highlight: 0, empty: "No results".to_owned(), width: 520.0, move_to: None, select: None }
    }

    pub fn items(mut self, items: &'a [CommandItem]) -> Self {
        self.items = items;
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    /// The highlighted row among the matches.
    pub fn highlight(mut self, i: usize) -> Self {
        self.highlight = i;
        self
    }

    pub fn empty(mut self, words: impl Into<String>) -> Self {
        self.empty = words.into();
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Up and Down moved the highlight to this row of the matches.
    pub fn on_highlight(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.move_to = Some(Rc::new(f));
        self
    }

    /// A command chosen (by Return or a press): its key.
    pub fn on_select(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.select = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let shown = filter(self.items, &self.edit.text);
        let at = self.highlight.min(shown.len().saturating_sub(1));
        let access = self.access.clone();
        let keys: Vec<String> = shown.iter().map(|i| i.key.clone()).collect();
        let descendant = keys.get(at).map(|k| format!("{}-{k}", self.key));
        let (move_to, select) = (self.move_to.clone(), self.select.clone());
        let n = shown.len();
        let mut input = rune::text_input(self.edit, t.ty.md)
            .key(format!("{}-input", self.key))
            .single_line()
            .grow(1.0)
            .min_w(0.0)
            .color(t.ink())
            .placeholder(self.placeholder.clone(), t.ink_faint())
            .caret(t.ink(), t.c.accent.fg.alpha(0.28))
            .label("Command")
            .on_edit(rune::edit_access(move |s: &mut S| access(s)))
            .on_key(move |s: &mut S, k: &KeyEvent, _| {
                match k.key {
                    Key::Down if n > 0 => move_to.as_ref().map(|f| f(s, (at + 1) % n)).is_some(),
                    Key::Up if n > 0 => move_to.as_ref().map(|f| f(s, (at + n - 1) % n)).is_some(),
                    Key::Enter => match (keys.get(at), select.as_ref()) {
                        (Some(key), Some(f)) => {
                            f(s, key);
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                }
            });
        if let Some(k) = descendant {
            input = input.active_descendant(k);
        }
        let search = row().w_full().h(44.0).px(t.space.at(4.0)).gap(t.space.at(2.5)).center_y().border_sides([0.0, 0.0, 1.0, 0.0], t.c.line).child(glyph(&icons::SEARCH, t.control.icon_lg, t.ink_faint())).child(input);
        let mut list = col().key(format!("{}-list", self.key)).role("listbox").label("Commands").w_full().max_h(320.0).padding(t.space.at(1.5)).gap(1.0).clip_rounded(0.0).scroll(rune::ScrollSpec { stick_bottom: false });
        if shown.is_empty() {
            list = list.child(row().w_full().h(64.0).center().child(text(self.empty.clone()).font(t.ty.base).color(t.ink_muted())));
        }
        let mut group: Option<&str> = None;
        for (i, item) in shown.iter().enumerate() {
            if group != Some(item.group.as_str()) {
                group = Some(item.group.as_str());
                list = list.child(row().h(28.0).px(t.space.at(2.5)).center_y().child(text(item.group.clone()).font(weight(t.ty.xs, t.ty.medium)).color(t.ink_faint())));
            }
            let st = States { selected: i == at, ..States::default() };
            let mut r = row().key(format!("{}-{}", self.key, item.key)).role("option").label(item.label.clone()).focus_selected(i == at).h(36.0).px(t.space.at(2.5)).gap(t.space.at(2.5)).center_y().radius(t.radius.md).continuous_corners();
            r = paint_state(r, &t.mat.row, st);
            if let Some(ic) = item.icon {
                r = r.child(glyph(ic, t.control.icon_lg, if i == at { t.ink() } else { t.ink_muted() }));
            }
            r = r.child(text(item.label.clone()).font(t.ty.base).color(t.ink()).grow(1.0).truncate().min_w(0.0).a11y_hidden());
            if let Some(k) = &item.shortcut {
                r = r.child(Kbd::new(format!("{}-{}-keys", self.key, item.key), k.clone()).build(t));
            }
            if let Some(f) = self.select.clone() {
                let key = item.key.clone();
                r = r.on_press(move |s: &mut S| f(s, &key));
            }
            list = list.child(r);
        }
        let p = col().key(self.key.clone()).role("dialog").label("Command palette").w(self.width).radius(t.radius.xl).continuous_corners().clip_rounded(t.radius.xl);
        paint(p, &t.mat.floating.base).child(search).child(list)
    }

    /// Mounts the palette as a modal near the top of the window while `open`.
    pub fn dialog(self, t: &Theme, open: bool, close: impl Fn(&mut S) + 'static, layer: &mut Vec<El>) {
        if !open {
            return;
        }
        let key = self.key.clone();
        let panel = self.build(t).modal(true, close);
        layer.push(el().key(format!("{key}-scrim")).absolute(0.0, 0.0).w_full().h_full().fill(t.c.page.fg.alpha(if t.dark { 0.45 } else { 0.12 })).a11y_hidden());
        layer.push(col().key(format!("{key}-layer")).absolute(0.0, 0.0).w_full().h_full().pt(t.space.at(24.0)).items_center().child(panel));
    }
}
