//! rune-ui registry: navigation_menu 0.5.1. Copied into your app by `rune-ui add navigation_menu`; yours to change. Needs: popup.
//!
//! A site's top navigation: plain links and titles that open a panel of links (a title over a line of description, in one or two
//! columns) on the **floating** material under the title, through the shared popup host. Titles are quiet until hovered; the open
//! one is selected and its chevron turns.
//!
//! Keys: Left and Right move along the bar (one Tab stop); Down, Return or Space open a panel with its first link focused; inside,
//! the arrows move between links, Left and Right at a panel's edge open the neighbouring panel, Escape closes and gives focus back to
//! the title, Return follows a link. A press outside closes. The app keeps which panel is open.
//!
//! ```ignore
//! NavigationMenu::new("site", "Main").panel("Learn", vec![NavLink::new("Introduction", "Reusable parts, styled by tokens")])
//!     .link("Docs", |s: &mut App| s.go("docs")).open(s.nav).on_open(|s: &mut App, i| s.nav = i).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Side, Tx, col, row, text};
use rune_ui_style::{Radius, weight};

use super::base::{Press, Shape, States, Theme, glyph, paint, paint_state, press};
use super::icons;
use super::popup::{INSET, mount};

/// A link in a panel.
pub struct NavLink<S: 'static> {
    pub title: String,
    pub description: String,
    pub press: Option<Press<S>>,
}

impl<S: 'static> NavLink<S> {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> NavLink<S> {
        NavLink { title: title.into(), description: description.into(), press: None }
    }

    pub fn on_press(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }
}

enum Item<S: 'static> {
    Link(String, Press<S>),
    Panel(String, Vec<NavLink<S>>, usize),
}

pub type OnOpen<S> = Rc<dyn Fn(&mut S, Option<usize>)>;

pub struct NavigationMenu<S: 'static> {
    key: String,
    name: String,
    items: Vec<Item<S>>,
    open: Option<usize>,
    on_open: Option<OnOpen<S>>,
}

impl<S: 'static> NavigationMenu<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> NavigationMenu<S> {
        NavigationMenu { key: key.into(), name: name.into(), items: Vec::new(), open: None, on_open: None }
    }

    /// A title that opens a panel of links in one column (two past four links).
    pub fn panel(mut self, title: impl Into<String>, links: Vec<NavLink<S>>) -> Self {
        let cols = if links.len() > 4 { 2 } else { 1 };
        self.items.push(Item::Panel(title.into(), links, cols));
        self
    }

    /// A plain link in the bar.
    pub fn link(mut self, title: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.items.push(Item::Link(title.into(), press(f)));
        self
    }

    /// The open panel, by its place in the bar (the app keeps this).
    pub fn open(mut self, at: Option<usize>) -> Self {
        self.open = at;
        self
    }

    pub fn on_open(mut self, f: impl Fn(&mut S, Option<usize>) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// The key of item `i` in the bar.
    pub fn item_key(key: &str, i: usize) -> String {
        format!("{key}-{i}-trigger")
    }

    /// The key of link `j` in panel `i`.
    pub fn link_key(key: &str, i: usize, j: usize) -> String {
        format!("{key}-{i}-link{j}")
    }

    /// The bar in place; the open panel goes into `layer` (mounted at the view's root: `base::overlay_root`).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let k = self.key.clone();
        let panels: Vec<usize> = self.items.iter().enumerate().filter_map(|(i, it)| matches!(it, Item::Panel(..)).then_some(i)).collect();
        let on_open = self.on_open.clone().unwrap_or_else(|| Rc::new(|_: &mut S, _| {}));
        let h = t.control.md;
        let r = Radius::fit(t.radius.md, h);
        let mut bar = row().key(k.clone()).role("navigation").label(self.name.clone());
        let mut list = row().key(format!("{k}-list")).role("toolbar").label(self.name.clone()).focus_group().gap(t.space.at(1.0));
        for (i, it) in self.items.iter().enumerate() {
            let open = self.open == Some(i);
            let mut b = row().key(Self::item_key(&k, i)).focusable(true).focus_selected(i == 0).h(h).px(t.metrics.pad_md - 4.0).gap(t.space.at(1.0)).center().radius(r).round_as(t);
            b = paint_state(b, &t.mat.row, States { selected: open, ..States::default() });
            match it {
                Item::Link(title, f) => {
                    let f = f.clone();
                    b = b.role("link").label(title.clone()).child(text(title.clone()).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).a11y_hidden()).on_press(move |s: &mut S| f(s));
                }
                Item::Panel(title, links, _) => {
                    let first = (!links.is_empty()).then(|| Self::link_key(&k, i, 0));
                    let f = on_open.clone();
                    b = b.role("button").label(title.clone()).expanded(open).child(text(title.clone()).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).a11y_hidden()).child(glyph(&icons::CHEVRON_DOWN, t.control.icon_sm, t.ink_muted()).rotate(if open { std::f32::consts::PI } else { 0.0 }));
                    b = b.on_press_tx(move |s: &mut S, tx: &mut Tx| {
                        f(s, if open { None } else { Some(i) });
                        if let (false, Some(r)) = (open, &first) {
                            tx.focus_when_shown(r.clone(), true);
                        }
                    });
                }
            }
            list = list.child(b);
        }
        let (f, prefix, panels2) = (on_open.clone(), format!("{k}-"), panels.clone());
        list = list.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let at = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(|r| r.strip_suffix("-trigger")).and_then(|i| i.parse::<usize>().ok());
            let opens = matches!(&ev.key, Key::Down) || matches!(&ev.key, Key::Text(c) if c == " ");
            match at {
                Some(i) if opens && panels2.contains(&i) => {
                    f(s, Some(i));
                    tx.focus_when_shown(format!("{prefix}{i}-link0"), true);
                    true
                }
                _ => false,
            }
        });
        bar = bar.child(list);
        if let Some(i) = self.open
            && let Some(Item::Panel(title, links, cols)) = self.items.into_iter().nth(i)
        {
            let shut = on_open.clone();
            let close = press(move |s: &mut S| shut(s, None));
            let width = if cols == 2 { 520.0 } else { 300.0 };
            let lr = Radius::inner(t.radius.xl, INSET);
            let n = links.len();
            let mut grid = row().key(format!("{k}-{i}-links")).role("list").label(title.clone()).focus_group().w_full().flex_wrap();
            for (j, l) in links.into_iter().enumerate() {
                let mut e = col().key(Self::link_key(&k, i, j)).role("link").label(l.title.clone()).description(l.description.clone()).focusable(true).w_frac(1.0 / cols as f32).padding(t.space.at(3.0)).gap(t.space.at(1.0)).radius(lr).round_as(t).focus_highlight(t.c.selected);
                e = e.child(text(l.title).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).a11y_hidden()).child(text(l.description).font(t.ty.sm).color(t.ink_muted()).wrap().max_lines(2).min_w(0.0).a11y_hidden());
                let c = close.clone();
                let p = l.press;
                e = e.on_press(move |s: &mut S| {
                    if let Some(p) = &p {
                        p(s);
                    }
                    c(s);
                });
                grid = grid.child(e);
            }
            // Left at the first column or Right at the last opens the neighbouring panel.
            let (f, pfx) = (on_open.clone(), format!("{k}-{i}-link"));
            let kk = k.clone();
            let surface = paint(col().key(format!("{k}-{i}-content")).role("dialog").label(title).w(width).padding(INSET).radius(t.radius.xl).round_as(t), &t.mat.floating.base).child(grid).on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
                let Some(j) = tx.focused().and_then(|f| f.strip_prefix(&pfx)).and_then(|j| j.parse::<usize>().ok()) else { return false };
                let (edge_left, edge_right) = (j % cols == 0, j % cols == cols - 1 || j + 1 == n);
                let dir: isize = match ev.key {
                    Key::Left if edge_left => -1,
                    Key::Right if edge_right => 1,
                    Key::Left | Key::Right => {
                        let to = if ev.key == Key::Left { j - 1 } else { j + 1 };
                        tx.focus(format!("{pfx}{to}"), true);
                        return true;
                    }
                    Key::Up if j >= cols => {
                        tx.focus(format!("{pfx}{}", j - cols), true);
                        return true;
                    }
                    Key::Down if j + cols < n => {
                        tx.focus(format!("{pfx}{}", j + cols), true);
                        return true;
                    }
                    Key::Up | Key::Down => return true,
                    _ => return false,
                };
                let at = panels.iter().position(|p| *p == i).unwrap_or(0) as isize;
                let next = panels[(at + dir).rem_euclid(panels.len() as isize) as usize];
                f(s, Some(next));
                tx.focus_when_shown(Self::link_key(&kk, next, 0), true);
                true
            });
            let trigger = Self::item_key(&k, i);
            mount(t, layer, &format!("{k}-{i}"), surface, &trigger, Side::Bottom, &trigger, close);
        }
        bar
    }
}
