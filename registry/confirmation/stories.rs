use rune::El;

use super::super::confirmation::{Confirmation, Decision};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("confirmation", "Confirmation", "agent").story("Asking, then answered", all::<S>)
}

/// The live request's answer, kept in the gallery.
pub fn decision(g: &super::super::gallery::GalleryState) -> Decision {
    match g.choice("confirm", "").as_str() {
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
        g.choose("confirm", match d { Decision::AllowedOnce => "once", Decision::AlwaysAllowed => "always", Decision::Denied => "deny", _ => "" });
    }).build(t);
    specs(t, vec![
        spec_wide(t, "Needs you: Tab to an answer, Return gives it, Escape denies", 520.0, live),
        spec_wide(t, "Allowed once", 520.0, Confirmation::<S>::new("confirm-once", "Edit .cargo/config.toml", Decision::AllowedOnce).build(t)),
        spec_wide(t, "Always allowed", 520.0, Confirmation::<S>::new("confirm-always", "Read files in src/", Decision::AlwaysAllowed).build(t)),
        spec_wide(t, "Denied", 520.0, Confirmation::<S>::new("confirm-denied", "Push to main", Decision::Denied).build(t)),
        spec_wide(t, "Expired", 520.0, Confirmation::<S>::new("confirm-expired", "Install a package", Decision::Expired).build(t)),
    ])
}
