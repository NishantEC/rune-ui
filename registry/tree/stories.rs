use rune::El;

use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, specs};
use super::super::icons;
use super::super::tree::{Tree, TreeNode, TreeState};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tree", "Tree", "structure").story("Keys, one or several chosen, lazy children", all::<S>)
}

/// A placeholder outline; `loaded` fills the lazy branch.
pub fn outline(loaded: bool) -> Vec<TreeNode> {
    let later = if loaded { TreeNode::branch("archive", "Archive", vec![TreeNode::leaf("q1", "First quarter"), TreeNode::leaf("q2", "Second quarter")]) } else { TreeNode::lazy("archive", "Archive") };
    vec![
        TreeNode::branch("plans", "Plans", vec![TreeNode::leaf("roadmap", "Roadmap").meta("3"), TreeNode::leaf("budget", "Budget"), TreeNode::branch("launch", "Launch", vec![TreeNode::leaf("checklist", "Checklist"), TreeNode::leaf("press", "Press notes").disabled(true)])]).icon(&icons::FOLDER),
        TreeNode::branch("people", "People", vec![TreeNode::leaf("asha", "Asha").icon(&icons::USER), TreeNode::leaf("lee", "Lee").icon(&icons::USER)]).icon(&icons::FOLDER),
        later.icon(&icons::FOLDER),
        TreeNode::leaf("notes", "Notes").icon(&icons::FILE),
    ]
}

fn slot<'a>(g: &'a mut GalleryState, key: &str) -> &'a mut TreeState {
    if !g.slots.contains_key(key) {
        let st = g.slot_mut::<TreeState>(key);
        st.expanded.insert("plans".to_owned());
        if key == "tree-demo" {
            st.selected.insert("budget".to_owned());
            st.cursor = Some("budget".to_owned());
        }
    }
    g.slot_mut::<TreeState>(key)
}

fn state(g: &GalleryState, key: &str) -> TreeState {
    if g.slots.contains_key(key) { g.slot(key) } else {
        let mut st = TreeState::default();
        st.expanded.insert("plans".to_owned());
        if key == "tree-demo" {
            st.selected.insert("budget".to_owned());
            st.cursor = Some("budget".to_owned());
        }
        st
    }
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let nodes = outline(g.flag("tree-loaded"));
    let single = Tree::<S>::new("tree-demo", "Project", &nodes, &state(g, "tree-demo"), move |s: &mut S| slot(acc(s), "tree-demo"))
        .on_activate(move |s: &mut S, id: &str| acc(s).choose("tree-opened", id))
        .on_expand(move |s: &mut S, id: &str, open| {
            if id == "archive" && open {
                acc(s).choose("tree-asked", id);
            }
        })
        .build(t);
    let multi_nodes = outline(true);
    let multi = Tree::<S>::new("tree-multi", "Choose several", &multi_nodes, &state(g, "tree-multi"), move |s: &mut S| slot(acc(s), "tree-multi")).multi_select(true).build(t);
    specs(t, vec![
        spec(t, "One choice: it follows the keys. Archive loads when opened", rune::col().w(300.0).child(single)),
        spec(t, "Several: Space chooses, Shift and the arrows extend", rune::col().w(300.0).child(multi)),
    ])
}
