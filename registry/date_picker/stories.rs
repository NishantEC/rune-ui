use super::super::calendar::Date;
use super::super::date_picker::DatePicker;
use super::super::gallery::{Access, Ctx, GalleryState, StoryItem, spec, specs};
use super::super::stories::calendar::TODAY;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("date_picker", "DatePicker", "form").story("One date, a range, empty, disabled", all::<S>)
}

fn date(g: &GalleryState, k: &str) -> Option<Date> {
    g.choices.get(k).and_then(|v| Date::parse(v))
}

fn put(g: &mut GalleryState, k: &str, d: Option<Date>) {
    match d {
        Some(d) => g.choose(k, &d.iso()),
        None => {
            g.choices.remove(k);
        }
    }
}

fn base<S: 'static>(key: &str, name: &str, g: &GalleryState, acc: Access<S>) -> DatePicker<S> {
    let (a, b, mk, pk) = (key.to_owned(), key.to_owned(), format!("{key}-month"), format!("{key}-picking"));
    let mut p = DatePicker::<S>::new(key.to_owned(), name.to_owned()).today(TODAY).open(g.is_open(key)).on_toggle(move |s: &mut S| acc(s).toggle_open(&a)).on_close(move |s: &mut S| acc(s).close(&b));
    if let Some(m) = date(g, &mk) {
        p = p.month(m);
    }
    let (mk2, pk2) = (mk.clone(), pk.clone());
    p.on_month(move |s: &mut S, m| put(acc(s), &mk2, Some(m))).picking(g.flag(&pk), move |s: &mut S| acc(s).toggle(&pk2))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let one = base::<S>("date-demo", "Due date", g, acc).value(date(g, "date-demo-value").or(Some(Date::new(2026, 11, 12)))).on_pick(move |s: &mut S, d| put(acc(s), "date-demo-value", Some(d))).build(t, c.layer);
    let set = g.choices.contains_key("date-range-a");
    let (a, b) = if set { (date(g, "date-range-a"), date(g, "date-range-b")) } else { (Some(Date::new(2026, 11, 9)), Some(Date::new(2026, 11, 18))) };
    let range = base::<S>("date-range", "Trip", g, acc).width(260.0).range(a, b).on_range(move |s: &mut S, a, b| {
        let g = acc(s);
        put(g, "date-range-a", a);
        put(g, "date-range-b", b);
    }).build(t, c.layer);
    let empty = base::<S>("date-empty", "Start date", g, acc).disabled_if(|d| d < TODAY).on_pick(move |s: &mut S, d| put(acc(s), "date-empty-value", Some(d))).value(date(g, "date-empty-value")).build(t, c.layer);
    let off = DatePicker::<S>::new("date-off", "Locked date").today(TODAY).value(Some(TODAY)).disabled(true).build(t, c.layer);
    specs(t, vec![spec(t, "One date", one), spec(t, "A range", range), spec(t, "Empty, past days disabled", empty), spec(t, "Disabled", off)])
}
