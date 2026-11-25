use rune::El;
use rune_ui_style::{ButtonVariant, Size};

use super::super::base::glyph;
use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::input_group::{InputGroup, words};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("input_group", "InputGroup", "form").story("Words, icons and buttons inside", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let site = InputGroup::<S>::new("group-site", g.edit("group-site"), move |s: &mut S| acc(s).edit_mut("group-site")).name("Website").placeholder("example.com").start(words(t, "https://")).end(Button::<S>::new("group-site-copy", "Copy address").variant(ButtonVariant::Ghost).size(Size::Sm).icon_only(&icons::COPY).build(t)).width(300.0).build(t);
    let search = InputGroup::<S>::new("group-search", g.edit("query"), move |s: &mut S| acc(s).edit_mut("query")).name("Search").placeholder("Search").start(glyph(&icons::SEARCH, t.control.icon, t.ink_faint())).end(words(t, "12 results")).width(300.0).build(t);
    let send = InputGroup::<S>::new("group-send", g.edit("group-send"), move |s: &mut S| acc(s).edit_mut("group-send")).name("Message").placeholder("Write a message").end(Button::<S>::new("group-send-go", "Send").size(Size::Sm).icon_only(&icons::ARROW_RIGHT).build(t)).width(300.0).build(t);
    let money = InputGroup::<S>::new("group-money", g.edit("budget"), move |s: &mut S| acc(s).edit_mut("budget")).name("Budget").start(words(t, "$")).end(words(t, "USD")).invalid(true).width(200.0).build(t);
    specs(t, vec![spec(t, "Words and a button", site), spec(t, "An icon and words", search), spec(t, "A send button", send), spec(t, "Invalid", money)])
}
