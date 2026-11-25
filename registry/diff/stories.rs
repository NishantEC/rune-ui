use rune::El;

use super::super::diff::{Diff, parse};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("diff", "Diff", "agent").story("Two files, one open", all::<S>)
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
