use rune::{El, col, row, text};
use rune_ui_style::weight;

use super::super::base::glyph;
use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, specs};
use super::super::hover_card::{HoverCard, HoverCardState};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("hover_card", "HoverCard", "overlays").story("Rest the pointer on the name", all::<S>)
}

fn state(g: &GalleryState) -> HoverCardState {
    HoverCardState { trigger: g.flag("hover-trigger"), card: g.flag("hover-card"), pinned: g.flag("hover-pinned") || g.is_open("hover-demo") }
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let name = text("@placeholder-team").font(weight(t.ty.base, t.ty.medium)).color(t.c.accent.fg).a11y_hidden();
    let face = row().size(40.0, 40.0).radius(20.0).fill(t.c.muted.bg).center().child(text("PT").font(weight(t.ty.sm, t.ty.semibold)).color(t.ink_muted()));
    let about = col().grow(1.0).shrink().min_w(0.0).gap(t.space.at(1.0))
        .child(text("@placeholder-team").font(weight(t.ty.base, t.ty.semibold)).color(t.ink()))
        .child(text("Keeps the sample projects tidy and answers questions about the gallery.").font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0))
        .child(row().gap(t.space.at(1.5)).center_y().pt(t.space.at(1.0)).child(glyph(&icons::CALENDAR, t.control.icon_sm, t.ink_faint())).child(text("Joined December 2024").font(t.ty.xs).color(t.ink_faint())));
    let card = HoverCard::<S>::new("hover-demo", "The placeholder team's profile", state(g)).trigger(name).child(row().w_full().gap(t.space.at(3.0)).items_start().child(face).child(about)).on_change(move |s: &mut S, h: HoverCardState| {
        let g = acc(s);
        g.flags.insert("hover-trigger".to_owned(), h.trigger);
        g.flags.insert("hover-card".to_owned(), h.card);
        g.flags.insert("hover-pinned".to_owned(), h.pinned);
        g.close("hover-demo");
    }).build(t, c.layer);
    let line = row().gap(t.space.at(1.0)).center_y().child(text("Maintained by").font(t.ty.base).color(t.ink_muted())).child(card);
    specs(t, vec![spec(t, "Opens after 500 ms; Return pins it, Escape hides it", line)])
}
