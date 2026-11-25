use rune::El;

use super::super::file_tree::{FileEntry, FileTree, nodes};
use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, specs};
use super::super::tree::TreeState;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("file_tree", "FileTree", "structure").story("Changed files, by kind", all::<S>)
}

/// Placeholder changes.
pub fn files() -> Vec<FileEntry> {
    [
        ("src/main.rs", "+4 −1"),
        ("src/ui/button.rs", "+12 −3"),
        ("src/ui/tree.rs", "+210"),
        ("src/ui/file10.rs", "+1"),
        ("src/ui/file2.rs", "+2"),
        ("docs/notes.md", "+8 −8"),
        ("docs/screens/home.png", "new"),
        ("Cargo.toml", "+1 −1"),
        (".gitignore", "+1"),
    ]
    .iter()
    .map(|(p, m)| FileEntry::new(*p).meta(*m))
    .collect()
}

fn seeded() -> TreeState {
    let mut st = TreeState::default();
    for d in ["src", "src/ui"] {
        st.expanded.insert(d.to_owned());
    }
    st.selected.insert("src/ui/tree.rs".to_owned());
    st.cursor = Some("src/ui/tree.rs".to_owned());
    st
}

fn slot(g: &mut GalleryState) -> &mut TreeState {
    if !g.slots.contains_key("file-tree") {
        *g.slot_mut::<TreeState>("file-tree") = seeded();
    }
    g.slot_mut::<TreeState>("file-tree")
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let tree = nodes(&files());
    let st = if g.slots.contains_key("file-tree") { g.slot("file-tree") } else { seeded() };
    let live = FileTree::<S>::new("file-tree", "Changed files", &tree, &st, move |s: &mut S| slot(acc(s))).on_activate(move |s: &mut S, path: &str| acc(s).choose("file-opened", path)).build(t);
    specs(t, vec![spec(t, "Folders first, natural order, an icon per kind; Return opens a file", rune::col().w(320.0).child(live))])
}
