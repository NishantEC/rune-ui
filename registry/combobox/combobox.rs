//! rune-ui registry: combobox 0.6.0. Copied into your app by `rune-ui add combobox`; yours to change. Needs: popup.
//!
//! One choice of many, with a search (shadcn's combobox: a popover holding a command list). The trigger is one **well**, like a select's.
//! Open, it shows rune-ui's popup under the trigger, at least its width: a search line over the matching rows, "No results" when
//! nothing matches. Focus stays in the search; Up and Down move the highlight (wrapping) and it follows the filter; Return chooses;
//! Escape or Tab closes, focus back on the trigger; a press outside closes. The app keeps the query, the highlight and whether it is open.
//!
//! ```ignore
//! Combobox::new("repo", "Repository", &options).current(s.repo.as_deref()).query(&s.q, |s: &mut App| &mut s.q).highlight(s.hl)
//!     .open(s.open).on_toggle(..).on_close(..).on_highlight(|s: &mut App, i| s.hl = i).on_choose(|s: &mut App, id: &str| ..).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Side, TextEdit, Tx, col, el, row, text};

use super::base::{Choose, Pick, Press, Shape, Theme, glyph, paint, press, recipes, resolve};
use super::icons;
use super::popup::{INSET, Row, Mount, mount, paint_row};

/// One option: an id and its words.
#[derive(Clone, Debug)]
pub struct ComboboxOption {
    pub id: String,
    pub label: String,
}

impl ComboboxOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> ComboboxOption {
        ComboboxOption { id: id.into(), label: label.into() }
    }
}

/// The options `query` keeps, in order: every word of it appears in the label, ignoring case.
pub fn filter<'a>(options: &'a [ComboboxOption], query: &str) -> Vec<&'a ComboboxOption> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    options.iter().filter(|o| words.iter().all(|w| o.label.to_lowercase().contains(w.as_str()))).collect()
}

type Access<S> = Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut TextEdit>;

pub struct Combobox<'a, S: 'static> {
    key: String,
    name: String,
    options: &'a [ComboboxOption],
    current: Option<String>,
    placeholder: String,
    width: f32,
    query: Option<(&'a TextEdit, Access<S>)>,
    highlight: usize,
    open: bool,
    toggle: Option<Press<S>>,
    close: Option<Press<S>>,
    move_to: Option<Choose<S>>,
    choose: Option<Pick<S>>,
}

impl<'a, S: 'static> Combobox<'a, S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, options: &'a [ComboboxOption]) -> Combobox<'a, S> {
        Combobox { key: key.into(), name: name.into(), options, current: None, placeholder: "Choose…".to_owned(), width: 240.0, query: None, highlight: 0, open: false, toggle: None, close: None, move_to: None, choose: None }
    }

    /// The chosen option's id.
    pub fn current(mut self, id: Option<&str>) -> Self {
        self.current = id.map(str::to_owned);
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// The search text, and how to reach it.
    pub fn query(mut self, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> Self {
        self.query = Some((edit, Rc::new(access)));
        self
    }

    /// The highlighted row among the matches.
    pub fn highlight(mut self, i: usize) -> Self {
        self.highlight = i;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(press(f));
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(press(f));
        self
    }

    /// Up or Down moved the highlight to this row of the matches (and typing should reset it to 0).
    pub fn on_highlight(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.move_to = Some(Rc::new(f));
        self
    }

    pub fn on_choose(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.choose = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let anchor = format!("{}-trigger", self.key);
        let parts = recipes::field(t);
        let current = self.current.as_deref().and_then(|id| self.options.iter().find(|o| o.id == id));
        let (words, ink) = match current {
            Some(o) => (o.label.clone(), t.ink()),
            None => (self.placeholder.clone(), t.ink_faint()),
        };
        let st = rune_ui_style::States { open: self.open, ..Default::default() };
        let mut trigger = row().key(anchor.clone()).role("combobox").label(self.name.clone()).expanded(self.open).focusable(true).w(self.width).h(t.control.md).pl(t.metrics.pad_md - 2.0).pr(t.metrics.gap).gap(t.metrics.gap).center_y().radius(t.radius.md).round_as(t);
        trigger = paint(trigger, &resolve(&parts.root, st));
        if let Some((w, o, c)) = parts.root.live().focus_ring {
            trigger = trigger.focus_ring(w, o, c);
        }
        trigger = trigger.child(text(words).font(t.ty.base).color(ink).grow(1.0).truncate().min_w(0.0).a11y_hidden()).child(glyph(&icons::CHEVRON_UP_DOWN, t.control.icon, t.ink_muted()));
        if let Some(f) = self.toggle.clone() {
            let input = format!("{}-input", self.key);
            trigger = trigger.on_press_tx(move |s: &mut S, tx: &mut Tx| {
                f(s);
                tx.focus_when_shown(input.clone(), true);
            });
        }
        if self.open {
            let close = self.close.clone().unwrap_or_else(|| press(|_: &mut S| {}));
            let q = self.query.as_ref().map_or("", |(e, _)| e.text.as_str());
            let shown = filter(self.options, q);
            let at = self.highlight.min(shown.len().saturating_sub(1));
            let ids: Vec<String> = shown.iter().map(|o| o.id.clone()).collect();
            let row_key = |id: &str| format!("{}-option-{id}", self.key);
            let mut search = row().key(format!("{}-search", self.key)).w_full().h(t.control.md).px(t.metrics.pad_md - INSET).gap(t.metrics.gap).center_y().child(glyph(&icons::SEARCH, t.control.icon, t.ink_faint()));
            if let Some((edit, access)) = &self.query {
                let access = access.clone();
                let (move_to, choose, keys_close) = (self.move_to.clone(), self.choose.clone(), close.clone());
                let n = ids.len();
                let picks = ids.clone();
                let mut input = rune::text_input(edit, t.ty.base).key(format!("{}-input", self.key)).single_line().grow(1.0).min_w(0.0).color(t.ink()).placeholder("Search", t.ink_faint()).caret(t.ink(), t.c.accent.fg.alpha(0.28)).label("Search").autofocus().on_edit(rune::edit_access(move |s: &mut S| access(s))).on_input_key(move |s: &mut S, k: &KeyEvent, _: &mut Tx| match k.key {
                    Key::Down if n > 0 => move_to.as_ref().map(|f| f(s, (at + 1) % n)).is_some(),
                    Key::Up if n > 0 => move_to.as_ref().map(|f| f(s, (at + n - 1) % n)).is_some(),
                    Key::Enter => match (picks.get(at), choose.as_ref()) {
                        (Some(id), Some(f)) => {
                            f(s, id);
                            keys_close(s);
                            true
                        }
                        _ => false,
                    },
                    Key::Tab => {
                        keys_close(s);
                        true
                    }
                    _ => false,
                });
                if let Some(id) = ids.get(at) {
                    input = input.active_descendant(row_key(id));
                }
                search = search.child(input);
            }
            let mut rows = col().key(format!("{}-list", self.key)).role("listbox").label(self.name.clone()).w_full().gap(1.0);
            if shown.is_empty() {
                rows = rows.child(row().key(format!("{}-empty", self.key)).w_full().h(t.metrics.row + 16.0).center().child(text("No results").font(t.ty.base).color(t.ink_muted())));
            }
            for (i, o) in shown.iter().enumerate() {
                let r = Row::<S>::new(o.label.clone()).checked(current.is_some_and(|c| c.id == o.id));
                let mut e = paint_row(t, &r, row_key(&o.id), Some(i == at)).role("option").checked(current.is_some_and(|c| c.id == o.id));
                if let Some(f) = self.choose.clone() {
                    let (id, close) = (o.id.clone(), close.clone());
                    e = e.on_press(move |s: &mut S| {
                        f(s, &id);
                        close(s);
                    });
                }
                rows = rows.child(e);
            }
            let body = col().w_full().max_h(280.0).shrink().min_h(0.0).padding(INSET).child(rune_kit::popover::fit_scroll(rows));
            let surface = col().key(format!("{}-content", self.key)).shrink().min_h(0.0).min_w(self.width).max_w(self.width.max(360.0)).radius(t.radius.lg).round_as(t);
            let surface = paint(surface, &t.mat.floating.base).child(search).child(el().w_full().h(1.0).fill(t.c.line).a11y_hidden()).child(body);
            mount(t, layer, surface, Mount { key: &self.key, anchor: &anchor, side: Side::Bottom, return_to: &anchor }, close);
        }
        row().key(self.key).child(trigger)
    }
}
