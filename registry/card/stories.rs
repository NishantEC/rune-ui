use rune::text;
use rune_ui_style::{ButtonVariant, Size};

use super::super::button::Button;
use super::super::card::{self, Card};
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::switch::Switch;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("card", "Card", "structure").story("Slab of rows and raised", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let sw = Switch::<S>::new("card-sync", "Sync", g.flag("switch-on")).on_toggle(move |s: &mut S| acc(s).toggle("switch-on")).build(t);
    let manage = Button::<S>::new("card-manage", "Manage").variant(ButtonVariant::Secondary).size(Size::Sm).build(t);
    let slab = Card::slab("card-slab").title("General").description("How the placeholder app behaves").rows(vec![
        card::line(t, "card-row-sync", "Sync in the background", Some("Keeps placeholder data fresh"), Some(sw)),
        card::line(t, "card-row-account", "Placeholder account", Some("Signed in as someone@example.com"), None),
        card::line(t, "card-row-data", "Stored data", Some("12 MB on this computer"), Some(manage)),
    ]);
    let raised = Card::raised("card-raised").title("A result that stands out").description("Raised cards lift a little; slab cards sit flat and are split by hairlines.").child(text("Three placeholder files changed.").font(t.ty.base).color(t.ink())).footer(rune::row().gap(t.space.at(2.0)).child(Button::<S>::new("card-dismiss", "Dismiss").variant(ButtonVariant::Ghost).build(t)).child(Button::<S>::new("card-open", "Open").build(t)));
    specs(t, vec![spec(t, "Slab", rune::col().w(440.0).child(slab.build(t))), spec(t, "Raised", rune::col().w(360.0).child(raised.build(t)))])
}
