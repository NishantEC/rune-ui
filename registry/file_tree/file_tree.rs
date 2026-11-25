//! rune-ui registry: file_tree 0.8.0. Copied into your app by `rune-ui add file_tree`; yours to change.
//! Needs: tree.
//!
//! A [`Tree`] of files and folders made from paths: folders first and sorted the way people sort names (`file2` before `file10`),
//! each file with an icon for its kind (code, text, config, image) and folders that open, and quiet figures per file (the `+12 −3`
//! of a change). It is a Tree in every other way: the same keys, focus, choice and lazy folders. The paths are data the app hands
//! in; nothing here reads the disk.
//!
//! ```ignore
//! let nodes = file_tree::nodes(&changes.iter().map(|c| FileEntry::new(&c.path).meta(format!("+{} −{}", c.added, c.removed))).collect::<Vec<_>>());
//! FileTree::new("changes", "Changed files", &nodes, &s.files, |s: &mut App| &mut s.files).on_activate(|s: &mut App, path: &str| s.show(path)).build(t)
//! ```

use std::cmp::Ordering;

use rune::{El, Icon};

use super::base::Theme;
use super::icons;
use super::tree::{Tree, TreeNode, TreeState};

/// One file: its `/`-separated path and its figures.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEntry {
    pub path: String,
    pub meta: Option<String>,
}

impl FileEntry {
    pub fn new(path: impl Into<String>) -> FileEntry {
        FileEntry { path: path.into(), meta: None }
    }

    pub fn meta(mut self, m: impl Into<String>) -> Self {
        self.meta = Some(m.into());
        self
    }
}

/// The icon for a file's kind, from its name.
pub fn file_icon(name: &str) -> &'static Icon {
    let lower = name.to_ascii_lowercase();
    let ext = lower.rsplit_once('.').map_or("", |(_, e)| e);
    match ext {
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "swift" | "c" | "h" | "cpp" | "java" | "kt" | "rb" | "sh" | "lua" | "html" | "css" | "sql" | "wgsl" => &icons::FILE_CODE,
        "md" | "txt" | "rst" | "csv" | "log" => &icons::FILE_TEXT,
        "toml" | "json" | "yaml" | "yml" | "lock" | "ini" | "cfg" | "conf" | "env" | "xml" => &icons::FILE_CONFIG,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "heic" => &icons::IMAGE,
        _ if lower.starts_with('.') || lower == "makefile" || lower == "dockerfile" => &icons::FILE_CONFIG,
        _ => &icons::FILE,
    }
}

/// Names as people sort them: case aside, runs of digits by their value (`file2` before `file10`).
pub fn natural(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(p), Some(q)) if p.is_ascii_digit() && q.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut n = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        n.push(c);
                        it.next();
                    }
                    n
                };
                let (m, n) = (take(&mut x), take(&mut y));
                let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                match m.len().cmp(&n.len()).then_with(|| m.cmp(n)) {
                    Ordering::Equal => {}
                    o => return o,
                }
            }
            (Some(p), Some(q)) => {
                match p.to_lowercase().cmp(q.to_lowercase()) {
                    Ordering::Equal => {}
                    o => return o,
                }
                x.next();
                y.next();
            }
        }
    }
}

/// The tree of `files`: a folder node for every folder on a path (its id the folder's path), a leaf for every file (its id the
/// file's path); folders first, then natural order.
pub fn nodes(files: &[FileEntry]) -> Vec<TreeNode> {
    fn insert(level: &mut Vec<TreeNode>, parts: &[&str], prefix: &str, f: &FileEntry) {
        let Some((name, rest)) = parts.split_first() else { return };
        let path = if prefix.is_empty() { (*name).to_owned() } else { format!("{prefix}/{name}") };
        if rest.is_empty() {
            let mut leaf = TreeNode::leaf(path, *name).icon(file_icon(name));
            leaf.meta = f.meta.clone();
            level.push(leaf);
            return;
        }
        let at = match level.iter().position(|n| n.id == path && n.is_branch()) {
            Some(i) => i,
            None => {
                level.push(TreeNode { lazy: false, ..TreeNode::branch(path.clone(), *name, Vec::new()).icons(&icons::FOLDER, &icons::FOLDER_OPEN) });
                level.len() - 1
            }
        };
        insert(&mut level[at].children, rest, &path, f);
    }
    fn sort(level: &mut [TreeNode]) {
        level.sort_by(|a, b| b.is_branch().cmp(&a.is_branch()).then_with(|| natural(&a.label, &b.label)));
        for n in level.iter_mut() {
            sort(&mut n.children);
        }
    }
    let mut top = Vec::new();
    for f in files {
        let parts: Vec<&str> = f.path.split('/').filter(|p| !p.is_empty()).collect();
        insert(&mut top, &parts, "", f);
    }
    sort(&mut top);
    top
}

/// A Tree of files: [`nodes`] in, Tree's keys, focus and choice.
pub struct FileTree<'a, S: 'static>(Tree<'a, S>);

impl<'a, S: 'static> FileTree<'a, S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, nodes: &'a [TreeNode], state: &TreeState, access: impl for<'b> Fn(&'b mut S) -> &'b mut TreeState + 'static) -> FileTree<'a, S> {
        FileTree(Tree::new(key, name, nodes, state, access))
    }

    pub fn multi_select(self, on: bool) -> Self {
        FileTree(self.0.multi_select(on))
    }

    pub fn max_height(self, h: f32) -> Self {
        FileTree(self.0.max_height(h))
    }

    pub fn on_select(self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        FileTree(self.0.on_select(f))
    }

    /// Return on a file, or a press on the chosen one: the app shows it (by its path).
    pub fn on_activate(self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        FileTree(self.0.on_activate(f))
    }

    /// A folder opened or closed (by its path).
    pub fn on_expand(self, f: impl Fn(&mut S, &str, bool) + 'static) -> Self {
        FileTree(self.0.on_expand(f))
    }

    pub fn build(self, t: &Theme) -> El {
        self.0.build(t)
    }
}
