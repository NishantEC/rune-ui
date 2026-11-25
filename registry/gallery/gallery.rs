//! rune-ui registry: gallery 0.2.0 (a block). Copied into your app by `rune-ui add gallery`; yours to change.
//!
//! A page that shows every component your app has added, in every state: group tabs, each component's stories (its variants × states),
//! a light/dark switch and a density switch, and a materials sheet that builds each recipe one layer at a time. It is made from the
//! same stories as rune-ui's own gallery app (`apps/gallery` is a window around this page).
//!
//! **Stories register themselves.** Each component's stories live in `src/ui/stories/<name>.rs` and expose `item::<S>()`; the CLI keeps
//! `src/ui/stories/mod.rs` listing the ones you have added (`stories::all()`), so a component you add appears here and one you have not
//! added does not. Stories are copied only once this block is installed.
//!
//! Mounting it in an app:
//!
//! ```ignore
//! struct App { gallery: ui::gallery::GalleryState, /* … */ }
//! // in the view, where the page goes:
//! ui::gallery::page(&theme_file, cx.system_dark(), &s.gallery, |s: &mut App| &mut s.gallery, &ui::stories::all())
//! ```

use rune::{El, TextEdit, col, el, row, text};
use rune_ui_style::{Layer, LayerKind, Material, Recipe, State as Look, Theme, ThemeFile, paint, weight};
use std::collections::HashMap;

/// How a story reaches the gallery's state inside the app's state.
pub type Access<S> = for<'a> fn(&'a mut S) -> &'a mut GalleryState;

/// What a story draws with.
pub struct Ctx<'a, S: 'static> {
    /// The gallery's state (what the live specimens hold).
    pub g: &'a GalleryState,
    pub t: &'a Theme,
    /// Overlays (menus, dialogs) mount here.
    pub layer: &'a mut Vec<El>,
    pub access: Access<S>,
}

/// One story: a name and what it draws.
pub struct Story<S: 'static> {
    pub name: &'static str,
    pub build: fn(&mut Ctx<S>) -> El,
}

/// A component's stories, as its `stories/<name>.rs` registers them.
pub struct StoryItem<S: 'static> {
    pub name: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    pub stories: Vec<Story<S>>,
}

impl<S: 'static> StoryItem<S> {
    pub fn new(name: &'static str, title: &'static str, group: &'static str) -> StoryItem<S> {
        StoryItem { name, title, group, stories: Vec::new() }
    }

    pub fn story(mut self, name: &'static str, build: fn(&mut Ctx<S>) -> El) -> Self {
        self.stories.push(Story { name, build });
        self
    }
}

/// The groups, in page order: (key, title, what the group shows).
pub const GROUPS: [(&str, &str, &str); 5] = [
    ("actions", "Actions", "Buttons and toggles: ink, raised, outline, ghost"),
    ("form", "Form", "Wells hold values; tracks hold a raised thumb"),
    ("overlays", "Menus and overlays", "The floating material: a smooth shadow stack with its hairline baked in"),
    ("structure", "Structure", "One surface divided into rows, not stacks of cards"),
    ("display", "Display", "Colour is state, at low alpha, never alone"),
];

/// The gallery's state. Keep one in your app's state and hand the page a way to reach it.
pub struct GalleryState {
    /// The group tab: a group key, `materials`, or `all`.
    pub show: String,
    /// One component alone (its name), over the group tab.
    pub only: Option<String>,
    /// Light or dark; `None` follows the app.
    pub dark: Option<bool>,
    /// The compact density (the theme's `density` knob at 0.875).
    pub compact: bool,
    /// The overlay open now (a menu, a dialog, a select's list).
    pub open: Option<String>,
    pub presses: u32,
    pub flags: HashMap<String, bool>,
    pub values: HashMap<String, f32>,
    pub choices: HashMap<String, String>,
    pub edits: HashMap<String, TextEdit>,
    pub combos: HashMap<String, rune_kit::ComboboxState>,
    empty: TextEdit,
}

impl Default for GalleryState {
    fn default() -> Self {
        let mut s = GalleryState { show: "actions".to_owned(), only: None, dark: None, compact: false, open: None, presses: 0, flags: HashMap::new(), values: HashMap::new(), choices: HashMap::new(), edits: HashMap::new(), combos: HashMap::new(), empty: TextEdit::new("") };
        for (k, v) in [("name", "Placeholder project"), ("budget", "12.50"), ("query", "theme"), ("secret", "sample-pass"), ("command-miss", "zzz"), ("notes", "A few words about the placeholder project, long enough to wrap onto a second line.")] {
            s.edits.insert(k.to_owned(), TextEdit::new(v));
        }
        for k in ["switch-on", "check-on", "toggle-bold", "accordion-0", "collapsible-open", "sidebar-projects"] {
            s.flags.insert(k.to_owned(), true);
        }
        s
    }
}

impl GalleryState {
    pub fn flag(&self, k: &str) -> bool {
        self.flags.get(k).copied().unwrap_or(false)
    }
    pub fn toggle(&mut self, k: &str) {
        let v = !self.flag(k);
        self.flags.insert(k.to_owned(), v);
    }
    pub fn value(&self, k: &str, default: f32) -> f32 {
        self.values.get(k).copied().unwrap_or(default)
    }
    pub fn set_value(&mut self, k: &str, v: f32) {
        self.values.insert(k.to_owned(), v);
    }
    pub fn choice(&self, k: &str, default: &str) -> String {
        self.choices.get(k).cloned().unwrap_or_else(|| default.to_owned())
    }
    /// A choice that is an index.
    pub fn index(&self, k: &str, default: usize) -> usize {
        self.choices.get(k).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
    pub fn choose(&mut self, k: &str, v: &str) {
        self.choices.insert(k.to_owned(), v.to_owned());
    }
    pub fn edit(&self, k: &str) -> &TextEdit {
        self.edits.get(k).unwrap_or(&self.empty)
    }
    pub fn edit_mut(&mut self, k: &str) -> &mut TextEdit {
        self.edits.entry(k.to_owned()).or_insert_with(|| TextEdit::new(""))
    }
    pub fn combo(&self, k: &str) -> &rune_kit::ComboboxState {
        self.combos.get(k).unwrap_or_else(|| blank_combo())
    }
    pub fn combo_mut(&mut self, k: &str) -> &mut rune_kit::ComboboxState {
        self.combos.entry(k.to_owned()).or_default()
    }
    pub fn is_open(&self, k: &str) -> bool {
        self.open.as_deref() == Some(k)
    }
    pub fn toggle_open(&mut self, k: &str) {
        self.open = if self.is_open(k) { None } else { Some(k.to_owned()) };
    }
    pub fn close(&mut self, k: &str) {
        if self.is_open(k) {
            self.open = None;
        }
    }
}

fn blank_combo() -> &'static rune_kit::ComboboxState {
    thread_local! { static B: &'static rune_kit::ComboboxState = Box::leak(Box::default()); }
    B.with(|b| *b)
}

/// A specimen with its caption under it.
pub fn spec(t: &Theme, caption: &str, e: El) -> El {
    col().gap(t.space.at(2.0)).items_start().child(e).child(text(caption.to_owned()).font(t.ty.xs).color(t.ink_faint()).a11y_hidden())
}

/// Specimens side by side, wrapping.
pub fn specs(t: &Theme, items: Vec<El>) -> El {
    row().w_full().gap(t.space.at(6.0)).items_start().flex_wrap().children(items)
}

/// A labelled line: the label at the left, the content after it.
pub fn labelled(t: &Theme, label: &str, e: El) -> El {
    row().gap(t.space.at(6.0)).items_start().child(col().w(96.0).pt(t.space.at(1.5)).child(text(label.to_owned()).font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted()))).child(e)
}

/// A quiet pill button for the page's own controls (it draws from materials, so it needs no other item).
fn pill<S: 'static>(t: &Theme, key: String, words: &str, on: bool, f: impl Fn(&mut S) + 'static) -> El {
    let m = if on { &t.mat.raised } else { &t.mat.row };
    let face = row().key(key).role("button").label(words.to_owned()).checked(on).focusable(true).h(t.control.sm).px(t.space.at(3.0)).center().radius(t.radius.md).continuous_corners();
    rune_ui_style::paint_state(face, m, Default::default()).child(text(words.to_owned()).font(weight(t.ty.sm, if on { t.ty.medium } else { t.ty.regular })).color(if on { t.ink() } else { t.ink_muted() }).a11y_hidden()).on_press(f)
}

/// One recipe built up a layer at a time, then each state's result.
fn layers(t: &Theme, name: &str, m: &Material, w: f32, h: f32, r: f32) -> El {
    let base = &m.base;
    let mut items = Vec::new();
    let mut so_far = Recipe::new();
    for (i, l) in base.layers.iter().enumerate() {
        so_far = so_far.layer(l.clone());
        if l.kind() == LayerKind::Shadow && base.layers.get(i + 1).is_some_and(|n| n.kind() == LayerKind::Shadow) {
            continue;
        }
        let word = match l {
            Layer::Fill(_) => "fill",
            Layer::Gradient { .. } => "+ gradient",
            Layer::Linear { .. } => "+ linear",
            Layer::InnerTopEdge { .. } => "+ lit top edge",
            Layer::InnerShadow(_) => "+ inner shadow",
            Layer::InnerRim { .. } => "+ inner rim",
            Layer::Shadow(s) if s.spread > 0.0 && s.blur == 0.0 => "+ baked ring",
            Layer::Shadow(_) => "+ drop shadows",
            Layer::Ring { .. } => "+ ring",
            Layer::Border { .. } => "+ border",
            Layer::Wash(_) => "+ wash",
            Layer::Scale(_) => "+ scale",
            Layer::Opacity(_) => "+ opacity",
            Layer::BackdropBlur(_) => "+ blur",
            Layer::Grain { .. } => "+ grain (skipped)",
        };
        items.push(spec(t, word, paint(el().size(w, h).radius(r).continuous_corners(), &so_far)));
    }
    items.push(el().w(1.0).h(h).fill(t.c.line));
    for s in [Look::Hover, Look::Pressed, Look::FocusVisible, Look::Selected, Look::Invalid, Look::Disabled] {
        if m.delta(s).is_some_and(|d| !d.is_empty()) {
            items.push(spec(t, s.word(), paint(el().size(w, h).radius(r).continuous_corners(), &m.state(s))));
        }
    }
    labelled(t, name, specs(t, items))
}

/// The materials sheet.
pub fn materials(t: &Theme) -> El {
    let mut c = col().key("materials").w_full().gap(t.space.at(6.0));
    for (name, m) in t.mat.all() {
        let (w, h, r) = match name {
            "floating" | "slab" => (120.0, 64.0, t.radius.xl),
            "thumb" => (24.0, 24.0, 12.0),
            "chip" => (72.0, 24.0, 12.0),
            _ => (96.0, t.control.lg, rune_ui_style::Radius::fit(t.radius.lg, t.control.lg)),
        };
        c = c.child(layers(t, name, m, w, h, r));
    }
    c.child(labelled(t, "atmosphere", spec(t, "fill + grain (skipped: no grain primitive in Rune)", paint(el().size(240.0, 64.0).radius(t.radius.xl), &t.mat.atmosphere))))
}

/// The theme the page draws in: the app's file, in the page's appearance and density.
pub fn theme_for(file: &ThemeFile, app_dark: bool, g: &GalleryState) -> Theme {
    let mut f = file.clone();
    if g.compact {
        f.base.density = 0.875;
    }
    Theme::from_file(&f, g.dark.unwrap_or(app_dark))
}

/// One component's block: its title and each story.
pub fn item_block<S: 'static>(c: &mut Ctx<S>, item: &StoryItem<S>) -> El {
    let t = c.t;
    let mut b = col().key(format!("item-{}", item.name)).w_full().gap(t.space.at(4.0)).child(row().gap(t.space.at(2.0)).items_end().child(text(item.title).font(t.ty.lg).color(t.ink()).role("heading").level(3)).child(text(item.name).font(t.ty.xs).color(t.ink_faint())));
    for st in &item.stories {
        let body = (st.build)(c);
        b = b.child(col().w_full().gap(t.space.at(2.0)).child(text(st.name).font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted())).child(body));
    }
    b
}

/// The page: the header with its controls, then the chosen group's components (or the materials sheet), and the overlays of whatever is
/// open, laid over the page.
pub fn page<S: 'static>(file: &ThemeFile, app_dark: bool, g: &GalleryState, access: Access<S>, items: &[StoryItem<S>]) -> El {
    let t = theme_for(file, app_dark, g);
    let mut layer = Vec::new();
    let mut tabs = row().key("gallery-groups").role("tablist").label("Groups").gap(t.space.at(1.0)).flex_wrap();
    let mut shown: Vec<(&str, &str)> = GROUPS.iter().filter(|(k, _, _)| items.iter().any(|i| i.group == *k)).map(|(k, n, _)| (*k, *n)).collect();
    shown.push(("materials", "Materials"));
    shown.push(("all", "All"));
    for (k, n) in shown {
        let key = k.to_owned();
        tabs = tabs.child(pill::<S>(&t, format!("gallery-group-{k}"), n, g.show == k && g.only.is_none(), move |s: &mut S| {
            let g = access(s);
            g.show = key.clone();
            g.only = None;
            g.open = None;
        }));
    }
    let dark = g.dark.unwrap_or(app_dark);
    let controls = row().gap(t.space.at(1.0)).child(pill::<S>(&t, "gallery-light".to_owned(), "Light", !dark, move |s: &mut S| access(s).dark = Some(false))).child(pill::<S>(&t, "gallery-dark".to_owned(), "Dark", dark, move |s: &mut S| access(s).dark = Some(true))).child(el().w(t.space.at(3.0))).child(pill::<S>(&t, "gallery-comfortable".to_owned(), "Comfortable", !g.compact, move |s: &mut S| access(s).compact = false)).child(pill::<S>(&t, "gallery-compact".to_owned(), "Compact", g.compact, move |s: &mut S| access(s).compact = true));
    let head = col().w_full().gap(t.space.at(3.0)).child(row().w_full().items_end().gap(t.space.at(3.0)).child(text("Components").font(t.ty.xxl).color(t.ink()).role("heading").level(1)).child(text(format!("{} theme · {} · {} components", t.name, if dark { "dark" } else { "light" }, items.len())).font(t.ty.base).color(t.ink_muted())).child(rune::spacer()).child(controls)).child(tabs);
    let mut body = col().key("gallery").w_full().padding(t.space.at(10.0)).gap(t.space.at(10.0)).child(head);
    let mut c = Ctx { g, t: &t, layer: &mut layer, access };
    for (key, title, what) in GROUPS {
        let list: Vec<&StoryItem<S>> = items.iter().filter(|i| i.group == key && match &g.only { Some(o) => o == i.name, None => g.show == "all" || g.show == key }).collect();
        if list.is_empty() {
            continue;
        }
        let mut sec = col().key(format!("group-{key}")).w_full().gap(t.space.at(8.0)).child(col().gap(t.space.at(1.0)).child(text(title).font(t.ty.xl).color(t.ink()).role("heading").level(2)).child(text(what).font(t.ty.base).color(t.ink_muted())));
        for item in list {
            sec = sec.child(item_block(&mut c, item));
        }
        body = body.child(sec);
    }
    if g.only.is_none() && (g.show == "materials" || g.show == "all") {
        body = body.child(col().key("group-materials").w_full().gap(t.space.at(6.0)).child(text("Materials").font(t.ty.xl).color(t.ink()).role("heading").level(2)).child(materials(&t)));
    }
    el().key("gallery-page").w_full().fill(t.c.page.bg).child(col().w_full().child(body)).children(layer)
}
