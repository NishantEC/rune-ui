use rune::El;

use super::super::commit::{Commit, CommitInfo, FileChange};
use super::super::diff::parse;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("commit", "Commit", "agent").story("A commit that folds to its files and diff", all::<S>)
}

pub fn info() -> CommitInfo {
    CommitInfo {
        hash: "3f9c2a71d04e8b6a5c1f0e9d8b7a6c5d4e3f2a10".to_owned(),
        subject: "fix(billing): round invoice totals half up".to_owned(),
        author: "Asha".to_owned(),
        time: "2 hours ago".to_owned(),
        files: vec![FileChange::new("src/billing/invoice.rs", 6, 2), FileChange::new("tests/invoice.rs", 14, 0)],
    }
}

const PATCH: &str = "diff --git a/src/billing/invoice.rs b/src/billing/invoice.rs\n--- a/src/billing/invoice.rs\n+++ b/src/billing/invoice.rs\n@@ -40,4 +40,8 @@\n fn total(lines: &[Line]) -> Cents {\n-    lines.iter().map(|l| l.cents).sum()\n+    let raw: f64 = lines.iter().map(|l| l.exact).sum();\n+    Cents((raw + 0.5).floor() as i64)\n }\n";

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let i = info();
    let live = Commit::<S>::new("commit-demo", &i, g.flag("commit-demo")).patch(parse(PATCH)).on_toggle(move |s: &mut S| acc(s).toggle("commit-demo"))
        .on_file(move |s: &mut S, path: &str| acc(s).choose("commit-file", path))
        .copied(g.flag("commit-copied")).on_copy(move |s: &mut S| { acc(s).flags.insert("commit-copied".to_owned(), true); }).build(t);
    let open = Commit::<S>::new("commit-open", &i, true).patch(parse(PATCH)).on_toggle(|_: &mut S| {}).on_file(|_: &mut S, _: &str| {}).build(t);
    specs(t, vec![
        spec_wide(t, "Folded: press it, or Return", 620.0, live),
        spec_wide(t, "Open: files and the diff", 620.0, open),
    ])
}
