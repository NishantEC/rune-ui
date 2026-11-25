//! rune-ui registry: calendar 0.5.1. Copied into your app by `rune-ui add calendar`; yours to change. Needs: button.
//!
//! A month as a grid of days (Sunday first, always six weeks so the height never changes), with today marked, one day or a range
//! selected, and days that cannot be chosen faint and skipped. The caption opens a month view (twelve months and the year), and the
//! arrows beside it step a month. Selected days are **ink**; a range is a quiet band between its two ink ends; today is a muted tint.
//!
//! Keys (the grid is one Tab stop on the selected day, or today, or the first): arrows move a day or a week, Home and End go to the
//! week's ends, Page Up and Page Down a month (with Shift a year), crossing into the next month when needed; Return or Space chooses.
//! Disabled days are stepped over. The app owns the month shown, the selection and whether the month view is open.
//!
//! ```ignore
//! Calendar::new("due", s.month).today(Date::today()).selected(s.due).on_month(|s: &mut App, m| s.month = m)
//!     .on_select(|s: &mut App, d| s.due = Some(d)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Tx, col, el, row, text};
use rune_ui_style::{ButtonVariant, Radius, tabular, weight};

use super::base::{Press, Shape, States, Theme, glyph, paint, paint_state, press, recipes};
use super::button::Button;
use super::icons;

/// A day on the proleptic Gregorian calendar. Ordered by time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

pub const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
pub const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

impl Date {
    /// A date; the day is kept inside its month.
    pub fn new(year: i32, month: u32, day: u32) -> Date {
        let month = month.clamp(1, 12);
        Date { year, month, day: day.clamp(1, days_in_month(year, month)) }
    }

    /// Today in UTC, from the system clock.
    pub fn today() -> Date {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        Date::from_days((secs / 86_400) as i64)
    }

    /// Days since 1970-01-01 (Howard Hinnant's algorithm).
    pub fn days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub fn from_days(z: i64) -> Date {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        Date { year: (yoe + era * 400 + i64::from(month <= 2)) as i32, month, day }
    }

    /// 0 is Sunday.
    pub fn weekday(self) -> u32 {
        (self.days() + 4).rem_euclid(7) as u32
    }

    pub fn add_days(self, n: i64) -> Date {
        Date::from_days(self.days() + n)
    }

    /// The same day `n` months on, kept inside the shorter month.
    pub fn add_months(self, n: i32) -> Date {
        let m = self.year * 12 + self.month as i32 - 1 + n;
        Date::new(m.div_euclid(12), (m.rem_euclid(12) + 1) as u32, self.day)
    }

    pub fn first_of_month(self) -> Date {
        Date { day: 1, ..self }
    }

    pub fn same_month(self, o: Date) -> bool {
        self.year == o.year && self.month == o.month
    }

    /// `2026-11-25`.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn parse(s: &str) -> Option<Date> {
        let mut p = s.splitn(3, '-');
        let (y, m, d) = (p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.parse().ok()?);
        ((1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)).then_some(Date { year: y, month: m, day: d })
    }

    /// `Nov 25, 2026`.
    pub fn short(self) -> String {
        format!("{} {}, {}", &MONTHS[self.month as usize - 1][..3], self.day, self.year)
    }

    /// `Wednesday, November 25, 2026`: a day's name for assistive technology.
    pub fn long(self) -> String {
        format!("{}, {} {}, {}", WEEKDAYS[self.weekday() as usize], MONTHS[self.month as usize - 1], self.day, self.year)
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// A range's next value after `d` is chosen, as shadcn's range calendar does: a first press starts it, a second ends it (either
/// order), a press once it is whole starts again.
pub fn range_pick(range: (Option<Date>, Option<Date>), d: Date) -> (Option<Date>, Option<Date>) {
    match range {
        (Some(a), None) if d >= a => (Some(a), Some(d)),
        (Some(a), None) => (Some(d), Some(a)),
        _ => (Some(d), None),
    }
}

/// A range as words: `Nov 3 – Nov 12, 2026`.
pub fn range_words(a: Date, b: Option<Date>) -> String {
    match b {
        Some(b) if a.year == b.year => format!("{} {} – {}", &MONTHS[a.month as usize - 1][..3], a.day, b.short()),
        Some(b) => format!("{} – {}", a.short(), b.short()),
        None => format!("{} – …", a.short()),
    }
}

pub type Disabled = Rc<dyn Fn(Date) -> bool>;
pub type OnDate<S> = Rc<dyn Fn(&mut S, Date)>;

pub struct Calendar<S: 'static> {
    key: String,
    shown: Date,
    today: Option<Date>,
    selected: Option<Date>,
    range: Option<(Date, Option<Date>)>,
    disabled: Option<Disabled>,
    picking: bool,
    framed: bool,
    month: Option<OnDate<S>>,
    select: Option<OnDate<S>>,
    pick: Option<Press<S>>,
}

impl<S: 'static> Calendar<S> {
    /// `shown`: any day of the month to show.
    pub fn new(key: impl Into<String>, shown: Date) -> Calendar<S> {
        Calendar { key: key.into(), shown: shown.first_of_month(), today: None, selected: None, range: None, disabled: None, picking: false, framed: true, month: None, select: None, pick: None }
    }

    pub fn today(mut self, d: Date) -> Self {
        self.today = Some(d);
        self
    }

    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }

    /// A range from its start to its end (the end is `None` while one end is chosen).
    pub fn range(mut self, start: Option<Date>, end: Option<Date>) -> Self {
        self.range = start.map(|a| (a, end));
        self
    }

    /// Days that cannot be chosen.
    pub fn disabled_if(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.disabled = Some(Rc::new(f));
        self
    }

    /// The month view (twelve months and the year) in place of the days.
    pub fn picking(mut self, on: bool) -> Self {
        self.picking = on;
        self
    }

    /// Drawn on the slab with padding (on by default); off inside a popover.
    pub fn framed(mut self, on: bool) -> Self {
        self.framed = on;
        self
    }

    /// The month to show: an arrow, a key that crosses a month, a month chosen, a day outside the month.
    pub fn on_month(mut self, f: impl Fn(&mut S, Date) + 'static) -> Self {
        self.month = Some(Rc::new(f));
        self
    }

    pub fn on_select(mut self, f: impl Fn(&mut S, Date) + 'static) -> Self {
        self.select = Some(Rc::new(f));
        self
    }

    /// The caption pressed (or Escape in the month view): open or close the month view.
    pub fn on_pick(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.pick = Some(press(f));
        self
    }

    /// The key of the day a calendar keyed `key` gives `d`.
    pub fn day_key(key: &str, d: Date) -> String {
        format!("{key}-day-{}", d.iso())
    }

    /// The day the keyboard lands on: the selection, the range's start, today, or the month's first day that can be chosen.
    pub fn landing(&self) -> Date {
        let off = |d: Date| self.disabled.as_ref().is_some_and(|f| f(d));
        let here = |d: &Date| d.same_month(self.shown) && !off(*d);
        if let Some(d) = self.selected.filter(here).or(self.range.map(|r| r.0).filter(here)).or(self.today.filter(here)) {
            return d;
        }
        (0..31).map(|i| self.shown.add_days(i)).find(|d| d.same_month(self.shown) && !off(*d)).unwrap_or(self.shown)
    }

    pub fn build(self, t: &Theme) -> El {
        let cell = t.control.md;
        let gap = t.space.at(0.5);
        let head = self.header(t, cell);
        let body = if self.picking { self.months(t, cell, gap) } else { self.days(t, cell, gap) };
        let c = col().key(self.key.clone()).role("group").label(format!("{} {}", MONTHS[self.shown.month as usize - 1], self.shown.year)).w(cell * 7.0).gap(t.space.at(2.0)).child(head).child(body);
        if self.framed {
            let r = t.radius.xl;
            paint(col().key(format!("{}-frame", self.key)).padding(t.metrics.pad_sm).radius(r).round_as(t), &t.mat.slab.base).child(c)
        } else {
            c
        }
    }

    fn header(&self, t: &Theme, cell: f32) -> El {
        let (shown, k) = (self.shown, &self.key);
        let step = if self.picking { 12 } else { 1 };
        let arrow = |dir: i32, icon, name: &str| {
            let mut b = Button::<S>::new(format!("{k}-{}", if dir < 0 { "prev" } else { "next" }), name.to_owned()).variant(ButtonVariant::Ghost).size(rune_ui_style::Size::Sm).icon_only(icon);
            if let Some(f) = self.month.clone() {
                b = b.on_press(move |s: &mut S| f(s, shown.add_months(dir * step)));
            }
            b.build(t)
        };
        let words = if self.picking { shown.year.to_string() } else { format!("{} {}", MONTHS[shown.month as usize - 1], shown.year) };
        let mut cap = row().key(format!("{k}-caption")).role("button").label(if self.picking { "Choose a day" } else { "Choose a month" }).expanded(self.picking).focusable(true).h(t.control.sm).px(t.space.at(2.0)).gap(t.space.at(1.0)).center().radius(Radius::fit(t.radius.md, t.control.sm)).round_as(t);
        cap = paint_state(cap, &t.mat.ghost, States::default()).child(text(words).font(weight(tabular(t.ty.base), t.ty.medium)).color(t.ink()).a11y_hidden()).child(glyph(&icons::CHEVRON_DOWN, t.control.icon_sm, t.ink_muted()).rotate(if self.picking { std::f32::consts::PI } else { 0.0 }));
        if let Some(f) = self.pick.clone() {
            cap = cap.on_press(move |s: &mut S| f(s));
        }
        let (prev, next) = if self.picking { ("Previous year", "Next year") } else { ("Previous month", "Next month") };
        row().key(format!("{k}-header")).w_full().h(cell).center_y().justify_between().child(arrow(-1, &icons::CHEVRON_LEFT, prev)).child(cap).child(arrow(1, &icons::CHEVRON_RIGHT, next))
    }

    fn days(&self, t: &Theme, cell: f32, gap: f32) -> El {
        let k = self.key.clone();
        let shown = self.shown;
        let start = shown.add_days(-i64::from(shown.weekday()));
        let landing = self.landing();
        let off = self.disabled.clone();
        let is_off = |d: Date| off.as_ref().is_some_and(|f| f(d));
        let r = Radius::fit(t.radius.md, cell);
        let mut names = row().w_full().h(t.ty.sm.line_height + 4.0);
        for w in WEEKDAYS {
            names = names.child(row().w(cell).h_full().center().role("columnheader").label(w.to_owned()).child(text(w[..2].to_owned()).font(t.ty.sm).color(t.ink_faint()).a11y_hidden()));
        }
        let mut grid = col().key(format!("{k}-grid")).role("grid").label(format!("{} {}", MONTHS[shown.month as usize - 1], shown.year)).focus_group().gap(gap).child(names);
        for w in 0..6 {
            let mut line = row().role("row");
            for wd in 0..7 {
                let d = start.add_days(w * 7 + wd);
                line = line.child(self.day(t, d, cell, r, d == landing, is_off(d)));
            }
            grid = grid.child(line);
        }
        let (month, prefix) = (self.month.clone(), format!("{k}-day-"));
        grid.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let Some(at) = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(Date::parse) else { return false };
            let (by_days, by_months): (i64, i32) = match ev.key {
                Key::Left => (-1, 0),
                Key::Right => (1, 0),
                Key::Up => (-7, 0),
                Key::Down => (7, 0),
                Key::Home => (-i64::from(at.weekday()), 0),
                Key::End => (6 - i64::from(at.weekday()), 0),
                Key::PageUp => (0, if ev.mods.shift { -12 } else { -1 }),
                Key::PageDown => (0, if ev.mods.shift { 12 } else { 1 }),
                _ => return false,
            };
            let mut to = if by_months != 0 { at.add_months(by_months) } else { at.add_days(by_days) };
            // Step over days that cannot be chosen, a day at a time in the direction of travel, for at most six weeks.
            let dir = if by_days < 0 || by_months < 0 { -1 } else { 1 };
            for _ in 0..42 {
                if !off.as_ref().is_some_and(|f| f(to)) {
                    break;
                }
                to = to.add_days(dir);
            }
            let key = format!("{prefix}{}", to.iso());
            if to.same_month(shown) {
                tx.focus(key, true);
            } else if let Some(f) = &month {
                f(s, to.first_of_month());
                tx.focus_when_shown(key, true);
            }
            true
        })
    }

    fn day(&self, t: &Theme, d: Date, cell: f32, r: f32, landing: bool, off: bool) -> El {
        let inside = d.same_month(self.shown);
        let (start, end) = match self.range {
            Some((a, Some(b))) => (Some(a), Some(b)),
            Some((a, None)) => (Some(a), Some(a)),
            None => (None, None),
        };
        let in_range = start.zip(end).is_some_and(|(a, b)| d >= a && d <= b);
        let is_end = start == Some(d) || end == Some(d);
        let chosen = self.selected == Some(d) || is_end;
        let today = self.today == Some(d);
        let mat = if chosen {
            recipes::check(t, true)
        } else if today {
            let mut m = t.mat.row.clone();
            m.base = m.base.fill(t.c.muted.bg);
            m
        } else {
            t.mat.row.clone()
        };
        let ink = if chosen { t.c.primary.fg } else if off || !inside { t.ink_faint() } else { t.ink() };
        let mut name = d.long();
        if today {
            name.push_str(", today");
        }
        let mut face = row().key(Self::day_key(&self.key, d)).role("gridcell").label(name).checked(chosen || (in_range && !is_end)).focusable(!off).focus_selected(landing).size(cell, cell).center().radius(r).round_as(t);
        face = paint_state(face, &mat, States { disabled: off, ..States::default() });
        let w = if chosen || today { t.ty.medium } else { t.ty.regular };
        face = face.child(text(d.day.to_string()).font(weight(tabular(t.ty.base), w)).color(ink).a11y_hidden());
        if off {
            face = face.disabled(true);
        } else if let Some(f) = self.select.clone() {
            let (month, shown) = (self.month.clone(), self.shown);
            face = face.on_press(move |s: &mut S| {
                if !d.same_month(shown)
                    && let Some(m) = &month
                {
                    m(s, d.first_of_month());
                }
                f(s, d);
            });
        }
        // The range's band sits behind the days, rounded at its ends and at the ends of each week.
        let mut holder = el().size(cell, cell);
        let whole = start.zip(end).is_some_and(|(a, b)| a != b);
        if in_range && whole {
            let left = start == Some(d) || d.weekday() == 0;
            let right = end == Some(d) || d.weekday() == 6;
            holder = holder.corners([if left { r } else { 0.0 }, if right { r } else { 0.0 }, if right { r } else { 0.0 }, if left { r } else { 0.0 }]).round_as(t).fill(t.c.muted.bg);
        }
        holder.child(face)
    }

    fn months(&self, t: &Theme, cell: f32, gap: f32) -> El {
        let k = self.key.clone();
        let shown = self.shown;
        // As tall as the weekday names and six weeks, so opening it moves nothing.
        let total = t.ty.sm.line_height + 4.0 + 6.0 * cell + 6.0 * gap;
        let h = cell;
        let w = (cell * 7.0 - 2.0 * gap) / 3.0;
        let r = Radius::fit(t.radius.md, h);
        let mut grid = col().key(format!("{k}-months")).role("grid").label(shown.year.to_string()).focus_group().h(total).justify_around();
        for q in 0..4 {
            let mut line = row().role("row").gap(gap);
            for i in 0..3 {
                let m = q * 3 + i + 1;
                let on = m == shown.month;
                let mat = if on { recipes::check(t, true) } else { t.mat.row.clone() };
                let mut b = row().key(format!("{k}-month-{m}")).role("gridcell").label(format!("{} {}", MONTHS[m as usize - 1], shown.year)).checked(on).focusable(true).focus_selected(on).size(w, h).center().radius(r).round_as(t);
                b = paint_state(b, &mat, States::default()).child(text(MONTHS[m as usize - 1][..3].to_owned()).font(weight(t.ty.base, if on { t.ty.medium } else { t.ty.regular })).color(if on { t.c.primary.fg } else { t.ink() }).a11y_hidden());
                let (f, p) = (self.month.clone(), self.pick.clone());
                b = b.on_press(move |s: &mut S| {
                    if let Some(f) = &f {
                        f(s, Date::new(shown.year, m, 1));
                    }
                    if let Some(p) = &p {
                        p(s);
                    }
                });
                line = line.child(b);
            }
            grid = grid.child(line);
        }
        let (pick, prefix) = (self.pick.clone(), format!("{k}-month-"));
        grid.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let at: Option<i32> = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(|m| m.parse().ok());
            match (&ev.key, at) {
                (Key::Escape, _) => {
                    if let Some(p) = &pick {
                        p(s);
                    }
                    true
                }
                (Key::Up | Key::Down, Some(m)) => {
                    let to = m + if ev.key == Key::Up { -3 } else { 3 };
                    if (1..=12).contains(&to) {
                        tx.focus(format!("{prefix}{to}"), true);
                    }
                    true
                }
                _ => false,
            }
        })
    }
}
