//! rune-ui registry: menubar 0.6.0. Copied into your app by `rune-ui add menubar`; yours to change. Needs: popup.
//!
//! A row of menus (File, Edit, View) on a quiet slab, each opening rune-ui's popup list under its title: the **floating** material,
//! rows with icons, shortcuts, checks, separators and labels, a highlight that follows the pointer and the keys.
//!
//! Keys: Left and Right move between titles (one Tab stop for the bar); Down, Return or Space open a menu with its first row
//! focused; inside, Up and Down move (wrapping), a typed letter jumps, Left and Right open the neighbouring menu (wrapping), Escape
//! closes and gives focus back to the title, Return chooses. A press outside closes. The app keeps which menu is open.
//!
//! Pointer: press a title to open it; while one is open, a press anywhere else closes it first (the shared popup's scrim takes the
//! press), so moving between open menus is a keyboard move or a second press.
//!
//! ```ignore
//! Menubar::new("app-menu", "Application").menu(Menu::new("File").row(Row::new("New").shortcut("⌘N").on_select(|s: &mut App| s.new())))
//!     .open(s.menu).on_open(|s: &mut App, m| s.menu = m).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Side, Tx, col, row, text};
use rune_ui_style::{Radius, weight};

use super::base::{Shape, States, Theme, paint, paint_state, press};
use super::popup::{Entry, INSET, Popup, Row, Mount, mount};

/// One menu: its title and its entries.
pub struct Menu<S: 'static> {
    pub title: String,
    pub entries: Vec<Entry<S>>,
    pub disabled: bool,
}

impl<S: 'static> Menu<S> {
    pub fn new(title: impl Into<String>) -> Menu<S> {
        Menu { title: title.into(), entries: Vec::new(), disabled: false }
    }

    pub fn row(mut self, r: Row<S>) -> Self {
        self.entries.push(Entry::Row(r));
        self
    }

    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    pub fn label(mut self, words: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(words.into()));
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }
}

/// The key the popup gives a menu's first row that can be chosen.
pub fn first_row<S>(popup_key: &str, entries: &[Entry<S>]) -> Option<String> {
    entries.iter().enumerate().find_map(|(i, e)| match e {
        Entry::Row(r) if !r.disabled => Some(r.key.clone().unwrap_or_else(|| format!("{popup_key}-item{}", i + 1))),
        _ => None,
    })
}

pub type OnOpen<S> = Rc<dyn Fn(&mut S, Option<usize>)>;

pub struct Menubar<S: 'static> {
    key: String,
    name: String,
    menus: Vec<Menu<S>>,
    open: Option<usize>,
    on_open: Option<OnOpen<S>>,
}

impl<S: 'static> Menubar<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Menubar<S> {
        Menubar { key: key.into(), name: name.into(), menus: Vec::new(), open: None, on_open: None }
    }

    pub fn menu(mut self, m: Menu<S>) -> Self {
        self.menus.push(m);
        self
    }

    /// The open menu (the app keeps this).
    pub fn open(mut self, at: Option<usize>) -> Self {
        self.open = at;
        self
    }

    /// A menu opened (`Some`) or closed (`None`).
    pub fn on_open(mut self, f: impl Fn(&mut S, Option<usize>) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// The key of menu `i`'s title.
    pub fn trigger_key(key: &str, i: usize) -> String {
        format!("{key}-{i}-trigger")
    }

    /// The bar in place; the open menu goes into `layer` (mounted at the view's root: `base::overlay_root`).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let k = self.key.clone();
        let n = self.menus.len();
        let live: Vec<bool> = self.menus.iter().map(|m| !m.disabled).collect();
        let firsts: Vec<Option<String>> = self.menus.iter().enumerate().map(|(i, m)| first_row(&format!("{k}-{i}"), &m.entries)).collect();
        let on_open = self.on_open.clone().unwrap_or_else(|| Rc::new(|_: &mut S, _| {}));
        // The next menu that can open, `dir` steps round from `i`.
        let step = {
            let live = live.clone();
            move |i: usize, dir: isize| (1..=n).map(|d| (i as isize + dir * d as isize).rem_euclid(n as isize) as usize).find(|j| live[*j]).unwrap_or(i)
        };
        let h = t.control.sm;
        let r = Radius::fit(t.radius.md, h);
        let mut bar = row().key(k.clone()).role("menubar").label(self.name.clone()).focus_group().gap(t.space.at(0.5)).padding(INSET).radius(Radius::inner(t.radius.md, -INSET)).round_as(t);
        bar = paint(bar, &t.mat.slab.base);
        for (i, m) in self.menus.iter().enumerate() {
            let open = self.open == Some(i);
            let mut b = row().key(Self::trigger_key(&k, i)).role("menuitem").label(m.title.clone()).expanded(open).focusable(!m.disabled).focus_selected(i == 0).h(h).px(t.metrics.pad_sm - 2.0).center().radius(r).round_as(t);
            b = paint_state(b, &t.mat.row, States { selected: open, disabled: m.disabled, ..States::default() });
            b = b.child(text(m.title.clone()).font(weight(t.ty.base, t.ty.medium)).color(if m.disabled { t.ink_faint() } else { t.ink() }).a11y_hidden());
            if m.disabled {
                b = b.disabled(true);
            } else {
                let (f, first) = (on_open.clone(), firsts[i].clone());
                b = b.on_press_tx(move |s: &mut S, tx: &mut Tx| {
                    f(s, if open { None } else { Some(i) });
                    if let (false, Some(r)) = (open, &first) {
                        tx.focus_when_shown(r.clone(), true);
                    }
                });
            }
            bar = bar.child(b);
        }
        // On a title: Down, Return or Space open its menu.
        let (f, firsts2, prefix) = (on_open.clone(), firsts.clone(), format!("{k}-"));
        bar = bar.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let at = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(|r| r.strip_suffix("-trigger")).and_then(|i| i.parse::<usize>().ok());
            match (&ev.key, at) {
                (Key::Down | Key::Enter, Some(i)) => {
                    f(s, Some(i));
                    if let Some(r) = &firsts2[i] {
                        tx.focus_when_shown(r.clone(), true);
                    }
                    true
                }
                (Key::Text(c), Some(i)) if c == " " => {
                    f(s, Some(i));
                    if let Some(r) = &firsts2[i] {
                        tx.focus_when_shown(r.clone(), true);
                    }
                    true
                }
                _ => false,
            }
        });
        if let Some(i) = self.open.filter(|i| *i < n && live[*i]) {
            let menu = self.menus.into_iter().nth(i).expect("open menu");
            let pk = format!("{k}-{i}");
            let mut popup = Popup::<S>::new(pk.clone(), "menu", menu.title.clone());
            popup.min_w = 200.0;
            popup.entries = menu.entries;
            let shut = on_open.clone();
            let close = press(move |s: &mut S| shut(s, None));
            let surface = popup.surface(t, close.clone());
            // Left and Right open the neighbouring menu with its first row focused.
            let (f, firsts) = (on_open.clone(), firsts.clone());
            let holder = col().key(format!("{pk}-menu")).child(surface).on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
                let dir = match ev.key {
                    Key::Left => -1,
                    Key::Right => 1,
                    _ => return false,
                };
                let j = step(i, dir);
                f(s, Some(j));
                if let Some(r) = &firsts[j] {
                    tx.focus_when_shown(r.clone(), true);
                }
                true
            });
            let trigger = Self::trigger_key(&k, i);
            mount(t, layer, holder, Mount { key: &pk, anchor: &trigger, side: Side::Bottom, return_to: &trigger }, close);
        }
        bar
    }
}
