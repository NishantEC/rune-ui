use rune::{El, col, row, text};
use rune_ui_style::{ButtonVariant, Size, tabular, weight};

use super::super::button::Button;
use super::super::drawer::Drawer;
use super::super::gallery::{Ctx, StoryItem, spec, spec_wide, specs};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("drawer", "Drawer", "overlays").story("Bottom sheet", all::<S>)
}

fn drawer<S: 'static>(c: &Ctx<S>, key: &str) -> Drawer<S> {
    let (t, g, acc) = (c.t, c.g, c.access);
    let goal = g.value("drawer-goal", 350.0);
    let step = |k: String, name: &str, icon, by: f32| Button::<S>::new(k, name.to_owned()).variant(ButtonVariant::Outline).size(Size::Lg).icon_only(icon).disabled((goal + by) < 200.0 || (goal + by) > 600.0).on_press(move |s: &mut S| {
        let g = acc(s);
        let v = g.value("drawer-goal", 350.0) + by;
        g.set_value("drawer-goal", v);
    }).build(t);
    let figure = col().grow(1.0).items_center().child(text(format!("{goal:.0}")).font(weight(tabular(t.ty.xxl), t.ty.semibold)).color(t.ink())).child(text("calories a day").font(t.ty.sm).color(t.ink_muted()));
    let body = row().w_full().center_y().gap(t.space.at(4.0)).py(t.space.at(2.0)).child(step(format!("{key}-less"), "Ten less", &icons::MINUS, -10.0)).child(figure).child(step(format!("{key}-more"), "Ten more", &icons::PLUS, 10.0));
    let (a, b) = (key.to_owned(), key.to_owned());
    Drawer::<S>::new(key.to_owned(), "Move goal").description("Set your daily activity goal.").child(body)
        .footer(vec![Button::<S>::new(format!("{key}-submit"), "Submit").full().on_press(move |s: &mut S| acc(s).close(&a)).build(t), Button::<S>::new(format!("{key}-cancel"), "Cancel").variant(ButtonVariant::Outline).full().on_press(move |s: &mut S| acc(s).close(&b)).build(t)])
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    let k = "drawer-demo".to_owned();
    let mut inplace = drawer(c, "drawer-inplace");
    let panel = col().w_full().child(inplace.panel(t));
    let open = c.g.is_open("drawer-demo");
    drawer(c, "drawer-demo").open(open).return_focus("drawer-open").on_close(move |s: &mut S| acc(s).close(&k)).build(t, c.layer);
    specs(t, vec![spec_wide(t, "Panel", 520.0, panel), spec(t, "Trigger (press it: the drawer rises over a scrim)", Button::<S>::new("drawer-open", "Open drawer").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| acc(s).toggle_open("drawer-demo")).build(t))])
}
