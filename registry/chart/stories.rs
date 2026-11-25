use rune::El;

use super::super::chart::{Chart, Series};
use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("chart", "Chart", "display").story("Line, bar, area, donut", all::<S>)
}

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn hover(g: &GalleryState, k: &str) -> Option<usize> {
    g.choices.get(k).and_then(|v| v.parse().ok()).or_else(|| g.open.as_deref().filter(|o| *o == k).map(|_| 3))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let keep = |k: &'static str| move |s: &mut S, i: Option<usize>| {
        let g = acc(s);
        match i {
            Some(i) => g.choose(k, &i.to_string()),
            None => {
                g.choices.remove(k);
            }
        }
        g.close(k);
    };
    let desktop = || Series::new("Desktop", vec![214.0, 302.0, 268.0, 341.0, 287.0, 355.0]);
    let mobile = || Series::new("Mobile", vec![96.0, 178.0, 141.0, 202.0, 166.0, 231.0]);
    let line = Chart::<S>::line("chart-line", "Visitors").categories(&MONTHS).series(desktop()).series(mobile()).hover(hover(g, "chart-line")).on_hover(keep("chart-line")).build(t, c.layer);
    let bar = Chart::<S>::bar("chart-bar", "Visitors").categories(&MONTHS).series(desktop()).series(mobile()).hover(hover(g, "chart-bar")).on_hover(keep("chart-bar")).build(t, c.layer);
    let area = Chart::<S>::area("chart-area", "Revenue").categories(&MONTHS).series(Series::new("Revenue", vec![1240.0, 1810.0, 1530.0, 2420.0, 2210.0, 2960.0])).format(|v| format!("${}", v.round() as i64)).hover(hover(g, "chart-area")).on_hover(keep("chart-area")).build(t, c.layer);
    let donut = Chart::<S>::donut("chart-donut", "Seats").categories(&["Free", "Team", "Business", "Enterprise", "Trial"]).series(Series::new("Seats", vec![420.0, 260.0, 180.0, 90.0, 50.0])).hover(hover(g, "chart-donut")).on_hover(keep("chart-donut")).build(t, c.layer);
    specs(t, vec![spec(t, "Line, two series", line), spec(t, "Bar, grouped", bar), spec(t, "Area", area), spec(t, "Donut", donut)])
}
