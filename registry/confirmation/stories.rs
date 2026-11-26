use rune::El;

use super::super::agent::Marks;
use super::super::confirmation::{Confirmation, Decision};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("confirmation", "Confirmation", "agent").story("Asking, then answered", all::<S>).story("Answered with neutral marks: no green, no red", neutral::<S>).story("In a list, with Escape left to the list", listed::<S>)
}

/// The live request's answer, kept in the gallery.
pub fn decision(g: &super::super::gallery::GalleryState) -> Decision {
    decision_of(g, "confirm")
}

fn decision_of(g: &super::super::gallery::GalleryState, key: &str) -> Decision {
    match g.choice(key, "").as_str() {
        "once" => Decision::AllowedOnce,
        "always" => Decision::AlwaysAllowed,
        "deny" => Decision::Denied,
        _ => Decision::Pending,
    }
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let live = Confirmation::<S>::new("confirm-demo", "Run a shell command", decision(g)).detail("rm -rf target/debug/incremental").on_decide(move |s: &mut S, d| {
        let g = acc(s);
        g.presses += 1;
        g.choose("confirm", word(d));
    }).build(t);
    let danger = Confirmation::<S>::new("confirm-danger", "Delete 214 files in fixtures/", decision_of(g, "confirm-danger")).detail("rm -rf fixtures/").why("Asked because commands can change anything on this Mac.").destructive(true).on_decide(move |s: &mut S, d| acc(s).choose("confirm-danger", word(d))).build(t);
    specs(t, vec![
        spec_wide(t, "Needs you: Tab to an answer, Return gives it, Escape denies", 520.0, live),
        spec_wide(t, "Destructive, with a why line: no Always allow, danger to confirm, Return does nothing", 520.0, danger),
        spec_wide(t, "Allowed once", 520.0, Confirmation::<S>::new("confirm-once", "Edit .cargo/config.toml", Decision::AllowedOnce).build(t)),
        spec_wide(t, "Always allowed", 520.0, Confirmation::<S>::new("confirm-always", "Read files in src/", Decision::AlwaysAllowed).build(t)),
        spec_wide(t, "Denied", 520.0, Confirmation::<S>::new("confirm-denied", "Push to main", Decision::Denied).build(t)),
        spec_wide(t, "Expired", 520.0, Confirmation::<S>::new("confirm-expired", "Install a package", Decision::Expired).build(t)),
    ])
}

fn word(d: Decision) -> &'static str {
    match d {
        Decision::AllowedOnce => "once",
        Decision::AlwaysAllowed => "always",
        Decision::Denied => "deny",
        _ => "",
    }
}

/// An answered request with quiet marks: a tick in the third ink, a dash.
fn neutral<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let one = |key: &str, title: &str, d: Decision| spec_wide(t, d.word(), 520.0, Confirmation::<S>::new(key.to_owned(), title.to_owned(), d).answered_marks(Marks::Neutral).build(t));
    specs(t, vec![
        one("confirm-n-once", "Edit .cargo/config.toml", Decision::AllowedOnce),
        one("confirm-n-always", "Read files in src/", Decision::AlwaysAllowed),
        one("confirm-n-denied", "Push to main", Decision::Denied),
        one("confirm-n-expired", "Install a package", Decision::Expired),
    ])
}

/// Three requests in one list: the first two wait with Escape left alone, the third is answered; each says its place.
fn listed<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let ask = |k: &'static str, title: &str, at: usize| {
        Confirmation::<S>::new(format!("confirm-l{at}"), title.to_owned(), decision_of(g, k)).escape_denies(false).in_list(at, 3).detail("cargo publish").on_decide(move |s: &mut S, d| acc(s).choose(k, word(d)))
    };
    let column = rune::col().key("confirm-list").role("list").label("Pending requests").w_full().gap(t.space.at(2.0))
        .child(ask("confirm-l1", "Publish the crate", 1).build(t))
        .child(ask("confirm-l2", "Push the tag", 2).build(t))
        .child(Confirmation::<S>::new("confirm-l3", "Edit Cargo.toml", Decision::AllowedOnce).in_list(3, 3).build(t));
    specs(t, vec![spec_wide(t, "In a list: \"1 of 3\"; Escape is left to the list and denies nothing", 520.0, column)])
}
