use rune::El;

use super::super::base::{ButtonVariant, Size};
use super::super::button::Button;
use super::super::diff::{Diff, FileDiff, parse};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("diff", "Diff", "agent").story("Two files, one open", all::<S>).story("Figures set by the app, selectable lines, a foot", slots::<S>)
}

const PATCH: &str = "diff --git a/src/retry.rs b/src/retry.rs\n--- a/src/retry.rs\n+++ b/src/retry.rs\n@@ -10,7 +10,9 @@ impl Importer {\n     fn fetch(&self, page: u32) -> Result<Body> {\n-        self.client.get(page)\n+        for attempt in 1..=3 {\n+            if let Ok(b) = self.client.get(page) { return Ok(b); }\n+            sleep(backoff(attempt));\n+        }\n+        Err(Error::GaveUp(page))\n     }\n }\ndiff --git a/docs/importer.md b/docs/importer.md\n--- a/docs/importer.md\n+++ b/docs/importer.md\n@@ -1,3 +1,3 @@\n # Importer\n-A failed page stops the import.\n+A failed page is tried three times, then skipped.\n";

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let files = parse(PATCH);
    let mut col = rune::col().w_full().gap(t.space.at(2.0));
    for (i, f) in files.iter().enumerate() {
        let key = format!("diff-{i}");
        let open = g.flag(&key) != (i == 0);
        col = col.child(Diff::<S>::new(key.clone(), f, open).on_toggle(move |s: &mut S| acc(s).toggle(&key)).build(t));
    }
    specs(t, vec![spec_wide(t, "Press a file's header to fold it", 600.0, col)])
}

/// A folded file whose figures the app knows (no lines yet), a file that only grew, selectable lines with a foot of actions, and a
/// binary file with only a foot.
fn slots<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let files = parse(PATCH);
    let folded = FileDiff { path: "src/importer/very/deep/folder/that/keeps/going/retry.rs".to_owned(), lines: Vec::new() };
    let binary = FileDiff { path: "assets/logo.png".to_owned(), lines: Vec::new() };
    let foot = |key: &str| {
        rune::row().gap(t.space.at(1.0)).child(Button::<S>::new(format!("{key}-copy"), "Copy path").variant(ButtonVariant::Ghost).size(Size::Sm).on_press(|_: &mut S| {}).build(t))
            .child(Button::<S>::new(format!("{key}-reveal"), "Reveal").variant(ButtonVariant::Ghost).size(Size::Sm).on_press(|_: &mut S| {}).build(t))
    };
    let (k1, k2) = ("diff-sel".to_owned(), "diff-bin".to_owned());
    let (a, b) = (k1.clone(), k2.clone());
    let col = rune::col().w_full().gap(t.space.at(2.0))
        .child(Diff::<S>::new("diff-folded", &folded, false).stats(120, 34).on_toggle(|_: &mut S| {}).build(t))
        .child(Diff::<S>::new("diff-grew", &files[0], false).hide_zero(true).stats(9, 0).on_toggle(|_: &mut S| {}).build(t))
        .child(Diff::<S>::new(k1.clone(), &files[0], !g.flag(&k1)).selectable(true).foot(foot("diff-sel-foot")).on_toggle(move |s: &mut S| acc(s).toggle(&a)).build(t))
        .child(Diff::<S>::new(k2.clone(), &binary, !g.flag(&k2)).stats(0, 0).hide_zero(true).foot(rune::text("Binary file").font(t.ty.sm).color(t.ink_muted())).on_toggle(move |s: &mut S| acc(s).toggle(&b)).build(t));
    specs(t, vec![spec_wide(t, "Folded with the app's figures; only the figures that are not zero; selectable lines with a foot; a binary file with only a note", 600.0, col)])
}
