use rune::text;
use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::icons;
use super::super::popover::Popover;
use super::super::switch::Switch;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("popover", "Popover", "overlays").story("Panel and trigger", all::<S>)
}

fn body<S: 'static>(c: &Ctx<S>) -> Vec<rune::El> {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    vec![text("Show only what needs an answer.").font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0), Switch::<S>::new("popover-switch", "Hide finished work", g.flag("switch-on")).show_label().on_toggle(move |s: &mut S| acc(s).toggle("switch-on")).build(t)]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    let mut inplace = Popover::<S>::new("popover-inplace").title("Filters");
    for b in body(c) {
        inplace = inplace.child(b);
    }
    let open = c.g.is_open("popover-demo");
    let trigger = Button::<S>::new("popover-button", "Filters").variant(ButtonVariant::Secondary).icon(&icons::GEAR).on_press(move |s: &mut S| acc(s).toggle_open("popover-demo")).build(t);
    let mut live = Popover::<S>::new("popover-demo").title("Filters").trigger(trigger).open(open).on_close(move |s: &mut S| acc(s).close("popover-demo"));
    for b in body(c) {
        live = live.child(b);
    }
    let live = live.build(t, c.layer);
    rune::col().child(specs(t, vec![spec(t, "Panel", inplace.panel(t)), spec(t, "Trigger (open in the open-popover scene)", live)])).child(rune::el().h(if open { 120.0 } else { 0.0 }))
}
