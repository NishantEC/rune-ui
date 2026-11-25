//! rune-ui registry: tree 0.8.0. Copied into your app by `rune-ui add tree`; yours to change.
//!
//! An outline of nested rows: a chevron that turns on each branch, an optional icon, the words, quiet figures at the right. The app
//! passes the nodes ([`TreeNode`]) and keeps a [`TreeState`] (open branches, the selection, the row the keyboard is on); the tree
//! reports what changed through it and through its callbacks.
//!
//! * **Focus and selection differ.** The keyboard's row has the focus ring; chosen rows have the selected fill. With one choice
//!   (the default) the choice follows the keys; with [`Tree::multi_select`] the keys only move, Space chooses or unchooses the row,
//!   and Shift with Up or Down extends the choice.
//! * **Keys.** The tree is one Tab stop, landing on the last row it was on. Up and Down move (stopping at the ends), Right opens a
//!   closed branch or moves to its first child, Left closes an open branch or moves to the parent, Home and End go to the first and
//!   last row, a typed letter moves to the next row starting with it, Return opens a leaf (`on_activate`) or folds a branch.
//! * **Lazy children.** A node made with [`TreeNode::lazy`] has no children yet: opening it calls `on_expand`, and until the app
//!   fills its children it shows a quiet loading row.
//!
//! ```ignore
//! Tree::new("outline", "Project", &nodes, &s.tree, |s: &mut App| &mut s.tree).on_activate(|s: &mut App, id: &str| s.open(id))
//!     .on_expand(|s: &mut App, id: &str, open| if open { s.load(id) }).build(t)
//! ```

use std::collections::BTreeSet;
use std::rc::Rc;

use rune::{El, Icon, Key, KeyEvent, Tx, col, el, row, text};
use rune_ui_style::{Radius, tabular};

use super::base::{Shape, States, Theme, glyph, paint_state, spinner_in};
use super::icons;

/// One row and what is under it.
#[derive(Clone, Debug, Default)]
pub struct TreeNode {
    /// Unique in the tree.
    pub id: String,
    pub label: String,
    pub icon: Option<&'static Icon>,
    /// The icon while open (a branch: an open folder).
    pub icon_open: Option<&'static Icon>,
    /// Quiet figures at the right ("+12 −3", "4").
    pub meta: Option<String>,
    pub children: Vec<TreeNode>,
    /// A branch whose children come later: opening it asks the app for them.
    pub lazy: bool,
    pub disabled: bool,
}

impl TreeNode {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> TreeNode {
        TreeNode { id: id.into(), label: label.into(), ..TreeNode::default() }
    }

    pub fn branch(id: impl Into<String>, label: impl Into<String>, children: Vec<TreeNode>) -> TreeNode {
        TreeNode { children, ..TreeNode::leaf(id, label) }
    }

    /// A branch with no children yet: `on_expand` asks for them.
    pub fn lazy(id: impl Into<String>, label: impl Into<String>) -> TreeNode {
        TreeNode { lazy: true, ..TreeNode::leaf(id, label) }
    }

    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    pub fn icons(mut self, closed: &'static Icon, open: &'static Icon) -> Self {
        self.icon = Some(closed);
        self.icon_open = Some(open);
        self
    }

    pub fn meta(mut self, m: impl Into<String>) -> Self {
        self.meta = Some(m.into());
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn is_branch(&self) -> bool {
        self.lazy || !self.children.is_empty()
    }
}

/// What the tree remembers: open branches, chosen rows and the keyboard's row (by id).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeState {
    pub expanded: BTreeSet<String>,
    pub selected: BTreeSet<String>,
    pub cursor: Option<String>,
}

/// A row as shown: its node, depth, parent and whether it is a loading placeholder.
#[derive(Clone, Debug)]
pub struct Visible<'a> {
    pub node: &'a TreeNode,
    pub depth: u32,
    pub parent: Option<&'a str>,
}

/// The rows that show, top to bottom: every node whose ancestors are all open.
pub fn visible<'a>(nodes: &'a [TreeNode], expanded: &BTreeSet<String>) -> Vec<Visible<'a>> {
    fn walk<'a>(nodes: &'a [TreeNode], depth: u32, parent: Option<&'a str>, expanded: &BTreeSet<String>, out: &mut Vec<Visible<'a>>) {
        for n in nodes {
            out.push(Visible { node: n, depth, parent });
            if n.is_branch() && expanded.contains(&n.id) {
                walk(&n.children, depth + 1, Some(&n.id), expanded, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(nodes, 0, None, expanded, &mut out);
    out
}

pub type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut TreeState>;
pub type OnId<S> = Rc<dyn Fn(&mut S, &str)>;
pub type OnExpand<S> = Rc<dyn Fn(&mut S, &str, bool)>;

pub struct Tree<'a, S: 'static> {
    key: String,
    name: String,
    nodes: &'a [TreeNode],
    state: TreeState,
    access: Access<S>,
    multi: bool,
    max_h: f32,
    select: Option<OnId<S>>,
    activate: Option<OnId<S>>,
    expand: Option<OnExpand<S>>,
}

/// One row's flat facts, kept for the key handler.
#[derive(Clone)]
struct Line {
    id: String,
    label: String,
    parent: Option<String>,
    branch: bool,
    open: bool,
    first_child: Option<String>,
    live: bool,
}

impl<'a, S: 'static> Tree<'a, S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, nodes: &'a [TreeNode], state: &TreeState, access: impl for<'b> Fn(&'b mut S) -> &'b mut TreeState + 'static) -> Tree<'a, S> {
        Tree { key: key.into(), name: name.into(), nodes, state: state.clone(), access: Rc::new(access), multi: false, max_h: 0.0, select: None, activate: None, expand: None }
    }

    /// Several rows can be chosen: Space chooses, Shift with the arrows extends, a press flips one row.
    pub fn multi_select(mut self, on: bool) -> Self {
        self.multi = on;
        self
    }

    /// At most this tall, then the rows scroll (0: no limit).
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_h = h;
        self
    }

    /// The choice changed: the row chosen (or flipped, with several).
    pub fn on_select(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.select = Some(Rc::new(f));
        self
    }

    /// Return on a leaf, or a press on a leaf that was already chosen.
    pub fn on_activate(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.activate = Some(Rc::new(f));
        self
    }

    /// A branch opened (`true`) or closed: load a lazy node's children here.
    pub fn on_expand(mut self, f: impl Fn(&mut S, &str, bool) + 'static) -> Self {
        self.expand = Some(Rc::new(f));
        self
    }

    pub fn row_key(key: &str, id: &str) -> String {
        format!("{key}-row-{id}")
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let st = &self.state;
        let rows = visible(self.nodes, &st.expanded);
        let lines: Vec<Line> = rows.iter().map(|v| Line {
            id: v.node.id.clone(),
            label: v.node.label.to_lowercase(),
            parent: v.parent.map(str::to_owned),
            branch: v.node.is_branch(),
            open: st.expanded.contains(&v.node.id),
            first_child: v.node.children.first().map(|c| c.id.clone()),
            live: !v.node.disabled,
        }).collect();
        // Tab lands on the keyboard's row, or the first chosen one, or the first row.
        let landing = st.cursor.clone().filter(|c| lines.iter().any(|l| &l.id == c && l.live)).or_else(|| lines.iter().find(|l| l.live && st.selected.contains(&l.id)).map(|l| l.id.clone())).or_else(|| lines.iter().find(|l| l.live).map(|l| l.id.clone()));
        let h = t.metrics.row - 4.0;
        let indent = t.control.icon + t.space.at(1.0);
        let mut body = col().key(format!("{k}-rows")).w_full().gap(1.0);
        for v in &rows {
            let n = v.node;
            let open = st.expanded.contains(&n.id);
            let chosen = st.selected.contains(&n.id);
            let key = Self::row_key(&k, &n.id);
            let mut line = row().key(key.clone()).role("treeitem").label(n.label.clone()).level(v.depth + 1).checked(chosen).focusable(!n.disabled).focus_selected(landing.as_deref() == Some(n.id.as_str())).w_full().h(h).pl(t.space.at(1.5) + v.depth as f32 * indent).pr(t.space.at(2.0)).gap(t.space.at(1.5)).center_y().radius(Radius::fit(t.radius.md, h)).round_as(t);
            if n.is_branch() {
                line = line.expanded(open);
            }
            line = paint_state(line, &t.mat.row, States { selected: chosen, disabled: n.disabled, ..States::default() });
            let ink = if n.disabled { t.ink_faint() } else if chosen { t.ink() } else { t.ink_muted() };
            line = line.child(if n.is_branch() { glyph(if open { &icons::CHEVRON_DOWN } else { &icons::CHEVRON_RIGHT }, t.control.icon_sm, t.ink_faint()) } else { el().size(t.control.icon_sm, t.control.icon_sm) });
            if let Some(i) = if open { n.icon_open.or(n.icon) } else { n.icon } {
                line = line.child(glyph(i, t.control.icon, ink));
            }
            line = line.child(text(n.label.clone()).font(t.ty.base).color(if n.disabled { t.ink_faint() } else { t.ink() }).truncate().grow(1.0).shrink().min_w(0.0).a11y_hidden());
            if let Some(m) = &n.meta {
                line = line.child(text(m.clone()).font(tabular(t.ty.sm)).color(t.ink_faint()).a11y_hidden());
            }
            if n.disabled {
                line = line.disabled(true);
            } else {
                let (acc, id, branch, multi) = (self.access.clone(), n.id.clone(), n.is_branch(), self.multi);
                let (select, activate, expand) = (self.select.clone(), self.activate.clone(), self.expand.clone());
                line = line.on_press(move |s: &mut S| {
                    let st = acc(s);
                    let was = st.selected.contains(&id);
                    st.cursor = Some(id.clone());
                    if multi {
                        if !st.selected.remove(&id) {
                            st.selected.insert(id.clone());
                        }
                    } else {
                        st.selected = BTreeSet::from([id.clone()]);
                    }
                    let opened = if branch { Some(!st.expanded.contains(&id)) } else { None };
                    if let Some(o) = opened {
                        if o { st.expanded.insert(id.clone()); } else { st.expanded.remove(&id); }
                    }
                    if let Some(f) = &select {
                        f(s, &id);
                    }
                    match opened {
                        Some(o) => {
                            if let Some(f) = &expand {
                                f(s, &id, o);
                            }
                        }
                        None if was && !multi => {
                            if let Some(f) = &activate {
                                f(s, &id);
                            }
                        }
                        None => {}
                    }
                });
            }
            body = body.child(line);
            if n.lazy && open && n.children.is_empty() {
                body = body.child(row().key(format!("{key}-loading")).role("status").label("Loading").w_full().h(h).pl(t.space.at(1.5) + (v.depth + 1) as f32 * indent + t.control.icon_sm + t.space.at(1.5)).gap(t.space.at(1.5)).center_y().child(spinner_in(t, t.control.icon_sm, t.ink_faint(), "Loading")).child(text("Loading…").font(t.ty.sm).color(t.ink_faint()).a11y_hidden()));
            }
        }
        let mut tree = col().key(k.clone()).role("tree").label(self.name.clone()).multiselectable(self.multi).focus_group().w_full();
        tree = if self.max_h > 0.0 { tree.child(col().key(format!("{k}-scroll")).w_full().max_h(self.max_h).scroll(rune::ScrollSpec { stick_bottom: false }).child(body)) } else { tree.child(body) };
        let (acc, multi) = (self.access.clone(), self.multi);
        let (select, activate, expand) = (self.select, self.activate, self.expand);
        let prefix = format!("{k}-row-");
        tree.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let Some(at) = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(|id| lines.iter().position(|l| l.id == id)) else { return false };
            if ev.mods.cmd || ev.mods.ctrl || ev.mods.alt {
                return false;
            }
            let here = &lines[at];
            let live: Vec<usize> = (0..lines.len()).filter(|i| lines[*i].live).collect();
            let next = |d: isize| -> Option<usize> {
                let p = live.iter().position(|i| *i == at)? as isize + d;
                (p >= 0).then(|| live.get(p as usize).copied()).flatten()
            };
            // Where the keys move to, and whether that moves the choice too.
            let to: Option<usize> = match &ev.key {
                Key::Down => next(1),
                Key::Up => next(-1),
                Key::Home => live.first().copied(),
                Key::End => live.last().copied(),
                Key::Right if here.branch && !here.open => {
                    acc(s).expanded.insert(here.id.clone());
                    if let Some(f) = &expand {
                        f(s, &here.id, true);
                    }
                    return true;
                }
                Key::Right if here.branch => here.first_child.as_ref().and_then(|c| lines.iter().position(|l| &l.id == c)),
                Key::Left if here.branch && here.open => {
                    acc(s).expanded.remove(&here.id);
                    if let Some(f) = &expand {
                        f(s, &here.id, false);
                    }
                    return true;
                }
                Key::Left => here.parent.as_ref().and_then(|p| lines.iter().position(|l| &l.id == p)),
                Key::Enter if here.branch => {
                    let st = acc(s);
                    let open = !st.expanded.remove(&here.id);
                    if open {
                        st.expanded.insert(here.id.clone());
                    }
                    if let Some(f) = &expand {
                        f(s, &here.id, open);
                    }
                    return true;
                }
                Key::Enter => {
                    if let Some(f) = &activate {
                        f(s, &here.id);
                    }
                    return true;
                }
                Key::Text(c) if c == " " => {
                    let st = acc(s);
                    if multi {
                        if !st.selected.remove(&here.id) {
                            st.selected.insert(here.id.clone());
                        }
                    } else {
                        st.selected = BTreeSet::from([here.id.clone()]);
                    }
                    if let Some(f) = &select {
                        f(s, &here.id);
                    }
                    return true;
                }
                Key::Text(c) if c.chars().count() == 1 && c != " " => {
                    let c = c.to_lowercase();
                    let n = live.len();
                    let p = live.iter().position(|i| *i == at).unwrap_or(0);
                    (1..=n).map(|d| live[(p + d) % n]).find(|i| lines[*i].label.starts_with(&c))
                }
                _ => return false,
            };
            let Some(to) = to else { return true };
            let id = lines[to].id.clone();
            let st = acc(s);
            st.cursor = Some(id.clone());
            let extend = multi && ev.mods.shift && matches!(ev.key, Key::Up | Key::Down);
            let chose = if extend {
                st.selected.insert(here.id.clone());
                st.selected.insert(id.clone());
                true
            } else if !multi {
                st.selected = BTreeSet::from([id.clone()]);
                true
            } else {
                false
            };
            if chose && let Some(f) = &select {
                f(s, &id);
            }
            tx.focus(format!("{prefix}{id}"), true);
            true
        })
    }
}
