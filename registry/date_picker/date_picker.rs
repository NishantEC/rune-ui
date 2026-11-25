//! rune-ui registry: date_picker 0.6.0. Copied into your app by `rune-ui add date_picker`; yours to change. Needs: calendar, popup.
//!
//! A date in a field. The trigger is a **well** (it holds a value) with a calendar glyph and the date, or a quiet placeholder; pressing
//! it opens a Calendar on the **floating** material under it through the shared popup host (anchored at the trigger's left edge,
//! flipping above near the window's bottom, a scrim that closes it on a press outside, Escape to close, Tab kept inside, focus back on the trigger).
//! Opening puts the keyboard on the chosen day (or today). One date closes it once chosen; a range closes once both ends are chosen.
//!
//! ```ignore
//! DatePicker::new("due", "Due date").value(s.due).month(s.month).open(s.open == Some("due"))
//!     .on_toggle(|s: &mut App| s.toggle("due")).on_close(|s: &mut App| s.open = None)
//!     .on_month(|s: &mut App, m| s.month = m).on_pick(|s: &mut App, d| s.due = Some(d)).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Side, Tx, col, row, text};

use super::base::{Press, Shape, Size, SizeExt, State, Theme, glyph, paint, press, recipes, resolve, states};
use super::calendar::{Calendar, Date, Disabled, OnDate, range_pick, range_words};
use super::icons;
use super::popup::{Mount, mount};

pub type OnRange<S> = Rc<dyn Fn(&mut S, Option<Date>, Option<Date>)>;

pub struct DatePicker<S: 'static> {
    key: String,
    name: String,
    value: Option<Date>,
    range: Option<(Option<Date>, Option<Date>)>,
    month: Option<Date>,
    today: Date,
    placeholder: String,
    width: f32,
    disabled: Option<Disabled>,
    off: bool,
    invalid: bool,
    pinned: Option<State>,
    open: bool,
    picking: bool,
    toggle: Option<Press<S>>,
    close: Option<Press<S>>,
    on_month: Option<OnDate<S>>,
    on_picking: Option<Press<S>>,
    pick: Option<OnDate<S>>,
    pick_range: Option<OnRange<S>>,
}

impl<S: 'static> DatePicker<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> DatePicker<S> {
        DatePicker { key: key.into(), name: name.into(), value: None, range: None, month: None, today: Date::today(), placeholder: "Pick a date".to_owned(), width: 240.0, disabled: None, off: false, invalid: false, pinned: None, open: false, picking: false, toggle: None, close: None, on_month: None, on_picking: None, pick: None, pick_range: None }
    }

    /// One date.
    pub fn value(mut self, d: Option<Date>) -> Self {
        self.value = d;
        self
    }

    /// A range instead of one date (`on_range` hears it).
    pub fn range(mut self, start: Option<Date>, end: Option<Date>) -> Self {
        self.range = Some((start, end));
        self
    }

    /// The month the calendar shows while open (the chosen date's month if not given).
    pub fn month(mut self, m: Date) -> Self {
        self.month = Some(m);
        self
    }

    pub fn today(mut self, d: Date) -> Self {
        self.today = d;
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

    /// Days that cannot be chosen.
    pub fn disabled_if(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.disabled = Some(Rc::new(f));
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.off = on;
        self
    }

    pub fn invalid(mut self, on: bool) -> Self {
        self.invalid = on;
        self
    }

    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    /// Whether the calendar is showing (the app keeps this).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// The calendar's month view, and what its caption does.
    pub fn picking(mut self, on: bool, f: impl Fn(&mut S) + 'static) -> Self {
        self.picking = on;
        self.on_picking = Some(press(f));
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(press(f));
        self
    }

    /// Escape, a press outside, or a date chosen.
    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(press(f));
        self
    }

    pub fn on_month(mut self, f: impl Fn(&mut S, Date) + 'static) -> Self {
        self.on_month = Some(Rc::new(f));
        self
    }

    /// One date chosen (then it closes).
    pub fn on_pick(mut self, f: impl Fn(&mut S, Date) + 'static) -> Self {
        self.pick = Some(Rc::new(f));
        self
    }

    /// A range's new ends (it closes once both are chosen).
    pub fn on_range(mut self, f: impl Fn(&mut S, Option<Date>, Option<Date>) + 'static) -> Self {
        self.pick_range = Some(Rc::new(f));
        self
    }

    /// The trigger in place; the open calendar goes into `layer` (mounted at the view's root: `base::overlay_root`).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let anchor = format!("{}-trigger", self.key);
        let open = self.open && !self.off;
        let parts = recipes::field(t);
        let mut st = states(self.pinned, self.off, false);
        st.invalid = self.invalid;
        st.open = open;
        let chosen = match self.range {
            Some((Some(a), b)) => Some(range_words(a, b)),
            Some((None, _)) => None,
            None => self.value.map(Date::short),
        };
        let (words, ink) = match &chosen {
            Some(w) => (w.clone(), parts.label.resolve(st)),
            None => (self.placeholder.clone(), t.ink_faint()),
        };
        let size = Size::Md;
        let mut trigger = row().key(anchor.clone()).role("button").label(self.name.clone()).expanded(open).focusable(!self.off).w(self.width).h(size.height(t)).px(size.pad(t) - 2.0).gap(t.metrics.gap).center_y().radius(size.radius(t)).round_as(t);
        if let Some(c) = &chosen {
            trigger = trigger.a11y_value(c.clone());
        }
        trigger = paint(trigger, &resolve(&parts.root, st));
        if !self.off
            && let Some((w, o, c)) = parts.root.live().focus_ring
        {
            trigger = trigger.focus_ring(w, o, c);
        }
        trigger = trigger.child(glyph(&icons::CALENDAR, size.icon(t), parts.icon.resolve(st))).child(text(words).font(size.text(t, t.ty.regular)).color(ink).grow(1.0).truncate().min_w(0.0).a11y_hidden());
        let start = self.range.map_or(self.value, |r| r.0);
        let shown = self.month.or(start).unwrap_or(self.today);
        let cal_key = format!("{}-calendar", self.key);
        let close = self.close.clone().unwrap_or_else(|| press(|_: &mut S| {}));
        let mut cal = Calendar::<S>::new(cal_key.clone(), shown).today(self.today).framed(false).picking(self.picking);
        cal = match self.range {
            Some((a, b)) => cal.range(a, b),
            None => cal.selected(self.value),
        };
        if let Some(f) = self.disabled.clone() {
            cal = cal.disabled_if(move |d| f(d));
        }
        if let Some(f) = self.on_month.clone() {
            cal = cal.on_month(move |s: &mut S, m| f(s, m));
        }
        if let Some(f) = self.on_picking.clone() {
            cal = cal.on_pick(move |s: &mut S| f(s));
        }
        if self.off {
            trigger = trigger.disabled(true);
        } else if let Some(f) = self.toggle.clone() {
            let landing = Calendar::<S>::day_key(&cal_key, cal.landing());
            trigger = trigger.on_press_tx(move |s: &mut S, tx: &mut Tx| {
                f(s);
                tx.focus_when_shown(landing.clone(), true);
            });
        }
        if open {
            let c = close.clone();
            cal = match (self.range, self.pick_range.clone(), self.pick.clone()) {
                (Some(r), Some(f), _) => cal.on_select(move |s: &mut S, d| {
                    let (a, b) = range_pick(r, d);
                    f(s, a, b);
                    if b.is_some() {
                        c(s);
                    }
                }),
                (None, _, Some(f)) => cal.on_select(move |s: &mut S, d| {
                    f(s, d);
                    c(s);
                }),
                _ => cal,
            };
            let surface = paint(col().key(format!("{}-content", self.key)).role("dialog").label(self.name.clone()).padding(t.metrics.pad_sm).radius(t.radius.xl).round_as(t), &t.mat.floating.base).child(cal.build(t));
            mount(t, layer, surface, Mount { key: &self.key, anchor: &anchor, side: Side::Bottom, return_to: &anchor }, close);
        }
        row().key(self.key).child(trigger)
    }
}
