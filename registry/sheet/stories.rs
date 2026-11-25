use rune::text;
use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::sheet::Sheet;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("sheet", "Sheet", "overlays").story("Side sheet", all::<S>)
}

fn sheet<S: 'static>(c: &Ctx<S>, key: &str) -> Sheet<S> {
    let acc = c.access;
    let t = c.t;
    let k = key.to_owned();
    Sheet::<S>::new(key.to_owned(), "Details").description("Everything about the placeholder task.").child(text("Started 2 hours ago by someone@example.com").font(t.ty.base).color(t.ink()).wrap().min_w(0.0)).on_close(move |s: &mut S| acc(s).close(&k))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    let mut inplace = sheet(c, "sheet-inplace").size(320.0);
    let panel = rune::col().h(260.0).child(inplace.panel(t));
    let open = c.g.is_open("sheet-demo");
    sheet(c, "sheet-demo").open(open).build(t, c.layer);
    specs(t, vec![spec(t, "Panel", panel), spec(t, "Trigger (open in the open-sheet scene)", Button::<S>::new("sheet-open", "Details").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| acc(s).toggle_open("sheet-demo")).build(t))])
}
