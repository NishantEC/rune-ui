use super::super::calendar::{Calendar, Date, range_pick};
use super::super::gallery::{Access, Ctx, GalleryState, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("calendar", "Calendar", "form").story("Single, range, disabled days, month view", all::<S>)
}

/// The day the gallery calls today, so pictures and snapshots do not move.
pub const TODAY: Date = Date { year: 2026, month: 11, day: 25 };

fn date(g: &GalleryState, k: &str) -> Option<Date> {
    g.choices.get(k).and_then(|v| Date::parse(v))
}

fn set(g: &mut GalleryState, k: &str, d: Option<Date>) {
    match d {
        Some(d) => g.choose(k, &d.iso()),
        None => {
            g.choices.remove(k);
        }
    }
}

/// A calendar whose month lives in the gallery's choices under `<key>-month`.
pub fn shown<S: 'static>(key: &str, g: &GalleryState, acc: Access<S>) -> Calendar<S> {
    let mk = format!("{key}-month");
    let month = date(g, &mk).unwrap_or(TODAY);
    let pk = format!("{key}-picking");
    let picking = g.flag(&pk);
    Calendar::<S>::new(key.to_owned(), month).today(TODAY).picking(picking).on_month(move |s: &mut S, m| set(acc(s), &mk, Some(m))).on_pick(move |s: &mut S| acc(s).toggle(&pk))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let single = shown::<S>("calendar", g, acc).selected(date(g, "calendar-day").or(Some(Date::new(2026, 11, 12)))).on_select(move |s: &mut S, d| set(acc(s), "calendar-day", Some(d))).build(t);
    let (a, b) = (date(g, "calendar-range-a").or(Some(Date::new(2026, 11, 9))), if g.choices.contains_key("calendar-range-a") { date(g, "calendar-range-b") } else { Some(Date::new(2026, 11, 18)) });
    let range = shown::<S>("calendar-range", g, acc).range(a, b).on_select(move |s: &mut S, d| {
        let (a, b) = range_pick((a, b), d);
        let g = acc(s);
        set(g, "calendar-range-a", a);
        set(g, "calendar-range-b", b);
    }).build(t);
    // Weekends and days before today cannot be chosen.
    let limited = shown::<S>("calendar-limited", g, acc).selected(date(g, "calendar-limited-day")).disabled_if(|d| d < TODAY || d.weekday() == 0 || d.weekday() == 6).on_select(move |s: &mut S, d| set(acc(s), "calendar-limited-day", Some(d))).build(t);
    let picker = Calendar::<S>::new("calendar-picker", TODAY).today(TODAY).picking(true).build(t);
    specs(t, vec![spec(t, "One day, today marked", single), spec(t, "A range", range), spec(t, "Weekends and past days disabled", limited), spec(t, "Month view (press the caption)", picker)])
}
