//! rune-ui registry: model_selector 0.6.0. Copied into your app by `rune-ui add model_selector`; yours to change.
//!
//! Choosing a model: a quiet trigger showing the chosen model, which opens a floating panel from the shared overlay layer, anchored to
//! the trigger (it never moves the page). The panel has a search field and the models grouped by provider, each row with a tier or
//! price hint at its right and a check on the chosen one. Typing filters (every word must appear in the name or provider); Up and Down
//! move the highlight, Return chooses, Escape or a press outside closes, and focus goes back to the trigger.
//!
//! ```ignore
//! ModelSelector::new("model", &models, &s.model, &s.query, |s: &mut App| &mut s.query).open(s.picking).highlight(s.at)
//!     .on_open(|s: &mut App, o| s.picking = o).on_highlight(|s: &mut App, i| s.at = i)
//!     .on_select(|s: &mut App, key: &str| s.model = key.to_owned()).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, TextEdit, col, row, text};
use rune_kit::Placement;
use rune_ui_style::{tabular, weight};

use super::base::{Choose, Pick, Shape, States, Theme, glyph, hit_area, paint, paint_state};
use super::icons;

/// One model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub key: String,
    pub name: String,
    pub provider: String,
    /// A tier or price ("Fast", "$3 / 1M").
    pub hint: String,
}

impl Model {
    pub fn new(key: impl Into<String>, name: impl Into<String>, provider: impl Into<String>, hint: impl Into<String>) -> Model {
        Model { key: key.into(), name: name.into(), provider: provider.into(), hint: hint.into() }
    }
}

/// The models `query` keeps, in order: every word appears in the name or the provider, ignoring case.
pub fn filter<'a>(models: &'a [Model], query: &str) -> Vec<&'a Model> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    models.iter().filter(|m| {
        let hay = format!("{} {}", m.name, m.provider).to_lowercase();
        words.iter().all(|w| hay.contains(w.as_str()))
    }).collect()
}

/// What opening or closing does (`true`: opened).
pub type Toggle<S> = Rc<dyn Fn(&mut S, bool)>;

pub struct ModelSelector<'a, S: 'static> {
    key: String,
    models: &'a [Model],
    chosen: &'a str,
    query: &'a TextEdit,
    access: Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut TextEdit>,
    open: bool,
    highlight: usize,
    opened: Option<Toggle<S>>,
    moved: Option<Choose<S>>,
    select: Option<Pick<S>>,
}

impl<'a, S: 'static> ModelSelector<'a, S> {
    pub fn new(key: impl Into<String>, models: &'a [Model], chosen: &'a str, query: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> ModelSelector<'a, S> {
        ModelSelector { key: key.into(), models, chosen, query, access: Rc::new(access), open: false, highlight: 0, opened: None, moved: None, select: None }
    }

    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }

    /// The highlighted row among the matches.
    pub fn highlight(mut self, i: usize) -> Self {
        self.highlight = i;
        self
    }

    pub fn on_open(mut self, f: impl Fn(&mut S, bool) + 'static) -> Self {
        self.opened = Some(Rc::new(f));
        self
    }

    pub fn on_highlight(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.moved = Some(Rc::new(f));
        self
    }

    /// A model chosen (by Return or a press): its key. The panel closes.
    pub fn on_select(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.select = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let trigger = format!("{}-trigger", self.key);
        let name = self.models.iter().find(|m| m.key == self.chosen).map_or_else(|| "Choose a model".to_owned(), |m| m.name.clone());
        let h = t.control.sm;
        let mut face = row().key(trigger.clone()).role("combobox").label(format!("Model: {name}")).expanded(self.open).focusable(true).h(h).pl(t.space.at(2.5)).pr(t.space.at(1.5)).gap(t.space.at(1.5)).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t);
        face = paint_state(face, &t.mat.ghost, Default::default())
            .child(text(name).font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted()).truncate().min_w(0.0).a11y_hidden())
            .child(glyph(&icons::CHEVRON_UP_DOWN, t.control.icon_sm, t.ink_faint()));
        let toggle = self.opened.clone().map(|f| {
            let open = self.open;
            super::base::press(move |s: &mut S| f(s, !open))
        });
        if let Some(p) = toggle.clone() {
            face = face.on_press(move |s: &mut S| p(s));
        }
        if self.open {
            layer.extend(self.panel(t, &trigger));
        }
        hit_area(&trigger, face, 0.0, t.control.slop(h), toggle)
    }

    fn panel(self, t: &Theme, trigger: &str) -> Vec<El> {
        let shown = filter(self.models, &self.query.text);
        let n = shown.len();
        let at = self.highlight.min(n.saturating_sub(1));
        let keys: Vec<String> = shown.iter().map(|m| m.key.clone()).collect();
        let close = self.opened.clone();
        let choose: Option<Pick<S>> = self.select.clone().map(|f| {
            let c = close.clone();
            let p: Pick<S> = Rc::new(move |s: &mut S, k: &str| {
                f(s, k);
                if let Some(c) = &c {
                    c(s, false);
                }
            });
            p
        });
        let (moved, pick) = (self.moved.clone(), choose.clone());
        let access = self.access.clone();
        let mut input = rune::text_input(self.query, t.ty.base)
            .key(format!("{}-search", self.key)).single_line().grow(1.0).min_w(0.0).color(t.ink())
            .placeholder("Search models", t.ink_faint()).caret(t.ink(), t.c.accent.fg.alpha(0.28)).label("Search models")
            .on_edit(rune::edit_access(move |s: &mut S| access(s)))
            .on_input_key(move |s: &mut S, k: &KeyEvent, _: &mut rune::Tx| match k.key {
                Key::Down if n > 0 => moved.as_ref().map(|f| f(s, (at + 1) % n)).is_some(),
                Key::Up if n > 0 => moved.as_ref().map(|f| f(s, (at + n - 1) % n)).is_some(),
                Key::Enter => match (keys.get(at), pick.as_ref()) {
                    (Some(k), Some(f)) => {
                        f(s, k);
                        true
                    }
                    _ => false,
                },
                _ => false,
            });
        if let Some(m) = shown.get(at) {
            input = input.active_descendant(format!("{}-{}", self.key, m.key));
        }
        let search = row().w_full().h(t.control.md + 4.0).px(t.space.at(3.0)).gap(t.space.at(2.0)).center_y().border_sides([0.0, 0.0, 1.0, 0.0], t.c.line).child(glyph(&icons::SEARCH, t.control.icon, t.ink_faint())).child(input);
        let mut list = col().key(format!("{}-list", self.key)).role("listbox").label("Models").w_full().max_h(300.0).padding(t.space.at(1.5)).gap(1.0).scroll(rune::ScrollSpec { stick_bottom: false });
        if shown.is_empty() {
            list = list.child(row().w_full().h(56.0).center().child(text("No models match").font(t.ty.base).color(t.ink_muted())));
        }
        let mut group: Option<&str> = None;
        for (i, m) in shown.iter().enumerate() {
            if group != Some(m.provider.as_str()) {
                group = Some(m.provider.as_str());
                list = list.child(row().h(26.0).px(t.space.at(2.5)).center_y().child(text(m.provider.clone()).font(weight(t.ty.xs, t.ty.medium)).color(t.ink_faint()).truncate().min_w(0.0)));
            }
            let chosen = m.key == self.chosen;
            let st = States { selected: i == at, ..States::default() };
            let mut r = row().key(format!("{}-{}", self.key, m.key)).role("option").label(format!("{}, {}, {}", m.name, m.provider, m.hint)).checked(chosen).focus_selected(i == at).w_full().h(t.metrics.row).px(t.space.at(2.5)).gap(t.metrics.gap).center_y().radius(t.radius.md).round_as(t);
            r = paint_state(r, &t.mat.row, st)
                .child(text(m.name.clone()).font(t.ty.base).color(t.c.floating.fg).truncate().min_w(0.0).grow(1.0).a11y_hidden())
                .child(text(m.hint.clone()).font(tabular(t.ty.sm)).color(t.ink_faint()).a11y_hidden())
                .child(row().size(t.control.icon, t.control.icon).center().children(chosen.then(|| glyph(&icons::CHECK, t.control.icon, t.ink()))));
            if let Some(f) = choose.clone() {
                let k = m.key.clone();
                r = r.on_press(move |s: &mut S| f(s, &k));
            }
            list = list.child(r);
        }
        let mut panel = paint(col().key(format!("{}-panel", self.key)).role("dialog").label("Choose a model").w(300.0).radius(t.radius.xl).round_as(t).clip_rounded(t.radius.xl), &t.mat.floating.base).child(search).child(list).focus_return(trigger);
        let mut out = Vec::new();
        if let Some(f) = close {
            let g = f.clone();
            panel = panel.modal(true, move |s: &mut S| g(s, false));
            out.push(rune_kit::scrim(format!("{}-scrim", self.key), move |s: &mut S| f(s, false)));
        }
        out.push(rune_kit::popover::anchored(t.to_kit(), panel, trigger, Placement::BottomStart, t.space.at(1.5)));
        out
    }
}
