//! rune-ui registry: gallery 0.5.1 (a block). Copied into your app by `rune-ui add gallery`; yours to change.
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
//! let mut layer = Vec::new();
//! let page = ui::gallery::page(&theme_file, cx.system_dark(), &s.gallery, |s: &mut App| &mut s.gallery, &ui::stories::all(), &mut layer);
//! ui::base::overlay_root(scrolling(page), layer) // at the view's root: overlays float above, anchored, out of the layout
//! ```

use super::base::Shape;
use rune::{El, TextEdit, col, el, row, text};
use rune::TextStyle;
use rune_ui_style::{Corners, Font, Layer, LayerKind, Material, Recipe, State as Look, Theme, paint, tabular, weight};
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
    /// The compact density (the theme's `density` knob at `COMPACT`, 0.8; comfortable is the theme's own, 1.0).
    pub compact: bool,
    /// The theme editor's values, written into the theme file before it is read.
    pub editor: Editor,
    /// Lay the page out to fill its window with the content scrolling beside the editor (an app's window), or whole (a picture).
    pub scroll: bool,
    /// Where the editor last exported the theme.
    pub exported: Option<String>,
    /// The overlay open now (a menu, a dialog, a select's list).
    pub open: Option<String>,
    /// Where a secondary press opened the context menu.
    pub point: Option<rune::PointerPress>,
    /// Toasts showing now (their keys).
    pub toasts: Vec<String>,
    pub presses: u32,
    pub flags: HashMap<String, bool>,
    pub values: HashMap<String, f32>,
    pub choices: HashMap<String, String>,
    pub edits: HashMap<String, TextEdit>,
    /// A component's own state type (a data table's sort and selection), by key: [`GalleryState::slot`].
    pub slots: HashMap<String, Box<dyn std::any::Any>>,
    empty: TextEdit,
}

impl Default for GalleryState {
    fn default() -> Self {
        let mut s = GalleryState { show: "actions".to_owned(), only: None, dark: None, compact: false, editor: Editor::default(), scroll: true, exported: None, open: None, point: None, toasts: Vec::new(), presses: 0, flags: HashMap::new(), values: HashMap::new(), choices: HashMap::new(), edits: HashMap::new(), slots: HashMap::new(), empty: TextEdit::new("") };
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
    /// A copy of the state kept under `k` (its default until something is kept).
    pub fn slot<T: std::any::Any + Clone + Default>(&self, k: &str) -> T {
        self.slots.get(k).and_then(|b| b.downcast_ref::<T>()).cloned().unwrap_or_default()
    }
    /// The state kept under `k`, made on first use.
    pub fn slot_mut<T: std::any::Any + Default>(&mut self, k: &str) -> &mut T {
        let b = self.slots.entry(k.to_owned()).or_insert_with(|| Box::new(T::default()));
        if !b.is::<T>() {
            *b = Box::new(T::default());
        }
        b.downcast_mut::<T>().expect("a slot of this type")
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

/// A specimen with its caption under it.
pub fn spec(t: &Theme, caption: &str, e: El) -> El {
    col().gap(t.space.at(2.0)).items_start().child(e).child(text(caption.to_owned()).font(t.ty.xs).color(t.ink_faint()).a11y_hidden())
}

/// Specimens side by side, wrapping.
pub fn specs(t: &Theme, items: Vec<El>) -> El {
    row().w_full().gap(t.metrics.stack + 4.0).items_start().flex_wrap().children(items)
}

/// A labelled line: the label at the left, the content after it.
pub fn labelled(t: &Theme, label: &str, e: El) -> El {
    row().w_full().gap(t.space.at(6.0)).items_start().child(col().w(96.0).min_w(96.0).pt(t.space.at(1.5)).child(text(label.to_owned()).font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted()))).child(e.grow(1.0).shrink().min_w(0.0))
}

/// A quiet pill button for the page's own controls (it draws from materials, so it needs no other item).
fn pill<S: 'static>(t: &Theme, key: String, words: &str, on: bool, f: impl Fn(&mut S) + 'static) -> El {
    pill_in::<S>(t, key, words, on, t.ty.sm, f)
}

/// A pill whose words are drawn in `style` (the font control shows each font in its own face).
fn pill_in<S: 'static>(t: &Theme, key: String, words: &str, on: bool, style: TextStyle, f: impl Fn(&mut S) + 'static) -> El {
    let m = if on { &t.mat.raised } else { &t.mat.row };
    let face = row().key(key).role("button").label(words.to_owned()).checked(on).focusable(true).h(t.control.sm).px(t.space.at(3.0)).center().radius(t.radius.md).round_as(t);
    rune_ui_style::paint_state(face, m, Default::default()).child(text(words.to_owned()).font(weight(style, if on { t.ty.medium } else { t.ty.regular })).color(if on { t.ink() } else { t.ink_muted() }).a11y_hidden()).on_press(f)
}

/// `style` in `font`, at its size adjustment (to half points).
fn in_font(style: TextStyle, font: &Font) -> TextStyle {
    TextStyle { family: font.family, size: (style.size * font.size_adjust * 2.0).round() / 2.0, ..style }
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
        items.push(spec(t, word, paint(el().size(w, h).radius(r).round_as(t), &so_far)));
    }
    items.push(el().w(1.0).h(h).fill(t.c.line));
    for s in [Look::Hover, Look::Pressed, Look::FocusVisible, Look::Selected, Look::Invalid, Look::Disabled] {
        if m.delta(s).is_some_and(|d| !d.is_empty()) {
            items.push(spec(t, s.word(), paint(el().size(w, h).radius(r).round_as(t), &m.state(s))));
        }
    }
    labelled(t, name, specs(t, items))
}

/// `e` rounded at `r` in a corner style, whatever the theme's: the corner specimen compares the two.
fn shaped(e: El, corners: Corners, r: f32) -> El {
    let e = e.radius(r);
    if corners == Corners::Continuous { e.continuous_corners() } else { e }
}

/// The corner style, judged side by side: a large tile round and continuous at the same nominal radius (continuous drawn on its
/// larger ladder, as the theme draws it), the continuous curve without that gain (why it is there), both outlines laid over each other,
/// and a tile and a button in this theme's own style.
pub fn corner_shapes(t: &Theme) -> El {
    const W: f32 = 240.0;
    const H: f32 = 160.0;
    const R: f32 = 28.0;
    let ink = t.c.primary.bg;
    let tile = |corners: Corners, r: f32| shaped(el().size(W, H), corners, r).fill(ink).a11y_hidden();
    let outline = |corners: Corners, r: f32, color| shaped(el().absolute(0.0, 0.0).size(W, H), corners, r).border(1.5, color);
    let overlaid = el().size(W, H).a11y_hidden().child(outline(Corners::Round, R, t.c.danger.solid)).child(outline(Corners::Continuous, R * rune_ui_style::CONTINUOUS_GAIN, ink));
    let themed = paint(el().key("corner-theme").role("img").label("A tile in this theme's corner style").size(W, H).radius(t.radius.xxl).round_as(t), &Recipe::new().fill(ink));
    let button_r = rune_ui_style::Radius::fit(t.radius.md, t.control.md);
    let button = paint(el().key("corner-theme-button").role("img").label("A button in this theme's corner style").size(120.0, t.control.md).radius(button_r).round_as(t), &Recipe::new().fill(ink));
    let gain = rune_ui_style::CONTINUOUS_GAIN;
    specs(t, vec![
        spec(t, &format!("Round, {R:.0}"), tile(Corners::Round, R)),
        spec(t, &format!("Continuous, {R:.0} (drawn at {:.0}, {gain}x)", R * gain), tile(Corners::Continuous, R * gain)),
        spec(t, &format!("Continuous at {R:.0} without the gain: within 0.4 pt of round"), tile(Corners::Continuous, R)),
        spec(t, "Both outlines: round in red, continuous in ink", overlaid),
        spec(t, &format!("This theme: {}, xxl ({:.1})", if t.corners == Corners::Continuous { "continuous" } else { "round" }, t.radius.xxl), themed),
        spec(t, &format!("A button: md ({button_r:.1})"), button),
    ])
}

/// Type set in the theme's font, to compare fonts: the headings, reading text, the small sizes, figures proportional and tabular, the
/// three weights, a button and an input, then one line in every installed font.
pub fn typography(t: &Theme) -> El {
    let f = &t.font;
    let line = |style: TextStyle, words: &str, color| text(words.to_owned()).font(style).color(color);
    let named = |label: &str, e: El| labelled(t, label, e);
    let about = format!("{}{}, size-adjust {:.2}, line height {:.2}. Rune draws variable fonts at each weight; font smoothing is the window's (Rune's `font_smoothing`), not the theme's.", f.title, if f.mono { " (monospaced)" } else { "" }, f.size_adjust, f.line_height);
    let headings = col().gap(t.space.at(2.0)).child(line(t.ty.xxl, "Plan the next quarter", t.ink()).key("typography-heading").role("heading").level(3).label("Plan the next quarter")).child(line(t.ty.xl, "Weekly review", t.ink())).child(line(t.ty.lg, "Waiting for approval", t.ink()));
    let body = col().gap(t.space.at(2.0)).max_w(560.0)
        .child(line(t.ty.md, "Plans, tickets and reviews live in one place. Invite the team, set a goal, and see where every piece of work stands before the Monday meeting.", t.ink()).wrap().min_w(0.0))
        .child(line(t.ty.base, "Assigned to you · Due in 3 days · 12 comments", t.ink_muted()))
        .child(line(t.ty.sm, "Last edited 2 minutes ago by a placeholder teammate", t.ink_muted()))
        .child(line(t.ty.xs, "BETA · VERSION 2.4.1", t.ink_faint()));
    let money = ["$1,204.50", "$987.10", "$11,111.11", "$40.00"];
    let column = |style: TextStyle, caption: &str| {
        let mut c = col().w(120.0).items_end().gap(t.space.at(1.0));
        for m in money {
            c = c.child(line(style, m, t.ink()));
        }
        spec(t, caption, c)
    };
    let figures = specs(t, vec![column(t.ty.md, "Proportional"), column(tabular(t.ty.md), "Tabular ('tnum'): columns line up")]);
    let weights = row().gap(t.space.at(6.0)).flex_wrap().child(line(weight(t.ty.lg, t.ty.regular), "Regular 400", t.ink())).child(line(weight(t.ty.lg, t.ty.medium), "Medium 500", t.ink())).child(line(weight(t.ty.lg, t.ty.semibold), "Semibold 600", t.ink()));
    let button = |m: &Material, words: &str, color| rune_ui_style::paint_state(row().h(t.control.md).px(t.metrics.pad_md).center().radius(rune_ui_style::Radius::fit(t.radius.md, t.control.md)).round_as(t), m, Default::default()).child(line(weight(t.ty.base, t.ty.medium), words, color));
    let input = paint(row().w(260.0).h(t.control.md).px(t.metrics.pad_md - 2.0).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, t.control.md)).round_as(t), &t.mat.well.base).child(line(t.ty.base, "Placeholder project", t.ink()));
    let controls = row().gap(t.space.at(3.0)).center_y().flex_wrap().child(button(&t.mat.ink, "Create project", t.c.primary.fg)).child(button(&t.mat.raised, "Cancel", t.ink())).child(input);
    let mut every = col().gap(t.space.at(3.0));
    for other in rune_ui_style::fonts::all() {
        every = every.child(row().gap(t.space.at(4.0)).items_center().child(line(t.ty.sm, other.title, t.ink_muted()).w(140.0)).child(line(in_font(t.ty.md, &other), "Revenue grew 18% to $4,210,980 in Q3", t.ink())));
    }
    col().key("typography").w_full().gap(t.space.at(8.0))
        .child(named("font", line(t.ty.base, &about, t.ink_muted()).wrap().max_w(560.0).min_w(0.0)))
        .child(named("headings", headings))
        .child(named("text", body))
        .child(named("figures", figures))
        .child(named("weights", weights))
        .child(named("controls", controls))
        .child(named("every font", every))
}

/// The materials sheet.
pub fn materials(t: &Theme) -> El {
    let mut c = col().key("materials").w_full().gap(t.space.at(6.0)).child(labelled(t, "corner shape", corner_shapes(t)));
    for (name, m) in t.mat.all() {
        let (w, h, r) = match name {
            "floating" | "slab" => (120.0, 64.0, t.radius.xl),
            "thumb" => (24.0, 24.0, 12.0),
            "chip" => (72.0, 24.0, 12.0),
            _ => (96.0, t.control.lg, rune_ui_style::Radius::fit(t.radius.lg, t.control.lg)),
        };
        c = c.child(layers(t, name, m, w, h, r));
    }
    c.child(labelled(t, "atmosphere", spec(t, "fill + grain (skipped: no grain primitive in Rune)", paint(el().size(240.0, 64.0).radius(t.radius.xl).round_as(t), &t.mat.atmosphere))))
}

/// The theme editor's values. Each one writes a key of the theme file (`edit_theme`), which is then read like any theme: the editor
/// proves the tokens drive everything.
#[derive(Clone, Debug, PartialEq)]
pub struct Editor {
    /// Index into [`NEUTRALS`].
    pub neutral: usize,
    /// The accent as `#rrggbb` (`None`: the theme's own).
    pub accent: Option<String>,
    /// `base.radius`, in points (`None`: the theme's own).
    pub radius: Option<f32>,
    pub corners: Corners,
    /// `base.font`: a font's name (built in or installed, `rune_ui_style::fonts`).
    pub font: &'static str,
    /// `base.type-scale`.
    pub type_scale: f32,
    /// Multiplies each mode's `elevation`.
    pub shadow: f32,
    /// Each mode's `texture`: flat (0) to tactile (1).
    pub texture: f32,
}

impl Default for Editor {
    fn default() -> Self {
        Editor { neutral: 0, accent: None, radius: None, corners: Corners::Continuous, font: "inter", type_scale: 1.0, shadow: 1.0, texture: 1.0 }
    }
}

/// The base neutral families: (name, chroma, hue). Index 0 keeps the theme's own ramp.
pub const NEUTRALS: [(&str, f32, f32); 5] = [("Theme", 0.0, 0.0), ("Grey", 0.0, 0.0), ("Stone", 0.010, 75.0), ("Zinc", 0.008, 265.0), ("Slate", 0.020, 255.0)];

/// Accent swatches.
pub const ACCENTS: [&str; 6] = ["#2f6fe4", "#7c4dea", "#0f9488", "#d9590f", "#d6336c", "#3f3f46"];

fn ramp_like(ramp: &mut rune_ui_style::serde_json::Value, chroma: f32, hue: f32, keep_ends: bool) {
    let Some(steps) = ramp.as_object_mut() else { return };
    for (step, v) in steps.iter_mut() {
        if step.starts_with('$') {
            continue;
        }
        let Some(old) = v["$value"].as_str().and_then(rune::color::hex) else { continue };
        if keep_ends && (step == "0" || step == "1000") {
            continue;
        }
        let l = old.to_oklab()[0];
        // Chroma fades toward black and white so the ends stay neutral and in gamut.
        let c = chroma * (1.0 - (2.0 * l - 1.0).powi(2)).max(0.0).sqrt();
        let hex = rune::oklch(l, c, hue).hex();
        v["$value"] = rune_ui_style::serde_json::Value::from(&hex[..7]);
    }
}

/// The theme file with the editor's values written in.
pub fn edit_theme(json: &str, g: &GalleryState) -> rune_ui_style::serde_json::Value {
    use rune_ui_style::serde_json::Value;
    let mut v: Value = rune_ui_style::serde_json::from_str(json).unwrap_or(Value::Null);
    let e = &g.editor;
    if e.neutral > 0
        && let Some((_, c, h)) = NEUTRALS.get(e.neutral)
    {
        ramp_like(&mut v["color"]["neutral"], *c, *h, true);
    }
    if let Some(accent) = e.accent.as_deref().and_then(rune::color::hex) {
        let lab = accent.to_oklab();
        ramp_like(&mut v["color"]["blue"], lab[1].hypot(lab[2]) * 1.15, lab[2].atan2(lab[1]).to_degrees(), false);
    }
    if let Some(r) = e.radius {
        v["base"]["radius"]["$value"] = Value::from(r);
    }
    v["base"]["corners"]["$value"] = Value::from(if e.corners == Corners::Continuous { "continuous" } else { "round" });
    v["base"]["font"]["$value"] = Value::from(e.font);
    v["base"]["type-scale"]["$value"] = Value::from(e.type_scale);
    if g.compact {
        v["base"]["density"]["$value"] = Value::from(rune_ui_style::COMPACT);
    }
    for m in ["light", "dark"] {
        let el = v["modes"][m]["number"]["elevation"]["$value"].as_f64().unwrap_or(1.0);
        v["modes"][m]["number"]["elevation"]["$value"] = Value::from(el * f64::from(e.shadow));
        v["modes"][m]["number"]["texture"]["$value"] = Value::from(e.texture);
    }
    v
}

/// The theme the page draws in: the app's theme file with the editor's values, in the page's appearance.
pub fn theme_for(json: &str, app_dark: bool, g: &GalleryState) -> Theme {
    let edited = edit_theme(json, g).to_string();
    Theme::from_json(&edited, g.dark.unwrap_or(app_dark)).or_else(|_| Theme::from_json(json, g.dark.unwrap_or(app_dark))).expect("the app's theme file parses")
}


// ---- the theme editor ----

fn heading(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(weight(t.ty.xs, t.ty.medium)).color(t.ink_faint())
}

/// A labelled control in the editor.
fn field(t: &Theme, label: &str, control: El) -> El {
    col().w_full().gap(t.metrics.label_gap).child(heading(t, label)).child(control)
}

/// One of a few, as pills that wrap.
fn choices<S: 'static>(t: &Theme, key: &str, words: &[&str], at: usize, access: Access<S>, set: fn(&mut GalleryState, usize)) -> El {
    let mut r = row().key(key.to_owned()).role("radiogroup").label(key.to_owned()).w_full().gap(t.space.at(1.0)).flex_wrap();
    for (i, w) in words.iter().enumerate() {
        r = r.child(pill::<S>(t, format!("{key}-{i}"), w, i == at, move |s: &mut S| set(access(s), i)));
    }
    r
}

/// The installed fonts as pills, each in its own face.
fn font_choices<S: 'static>(t: &Theme, at: &str, access: Access<S>) -> El {
    let mut r = row().key("editor-font").role("radiogroup").label("Font").w_full().gap(t.space.at(1.0)).flex_wrap();
    for f in rune_ui_style::fonts::all() {
        let name = f.name;
        r = r.child(pill_in::<S>(t, format!("editor-font-{name}"), f.title, name == at, in_font(t.ty.sm, &f), move |s: &mut S| access(s).editor.font = name));
    }
    r
}

/// A value in a range: a well track, an ink fill, a thumb; drag or press anywhere on it.
fn slider<S: 'static>(t: &Theme, key: &str, value: f32, min: f32, max: f32, words: String, access: Access<S>, set: fn(&mut GalleryState, f32)) -> El {
    let frac = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let track = paint(row().w_full().h(6.0).radius(3.0), &t.mat.well.base).child(paint(el().w_frac(frac).h(6.0).radius(3.0), &Recipe::new().fill(t.c.primary.bg)));
    let thumb = row().absolute(0.0, 2.0).w_full().h(16.0).child(el().w_frac(frac).h(16.0)).child(paint(el().size(16.0, 16.0).radius(8.0).ml(-16.0 * frac), &t.mat.thumb.base));
    let bar = col().key(key.to_owned()).role("slider").label(key.to_owned()).range_value(f64::from(value), f64::from(min), f64::from(max)).focusable(true).grow(1.0).h(20.0).justify_center().child(track).child(thumb).on_drag(move |s: &mut S, d: &mut rune::Drag, _: &mut rune::Tx| {
        set(access(s), min + (d.x / d.w.max(1.0)).clamp(0.0, 1.0) * (max - min));
    });
    row().w_full().gap(t.space.at(3.0)).center_y().child(bar).child(text(words).font(rune_ui_style::tabular(t.ty.sm)).color(t.ink_muted()).w(40.0))
}

/// The editor: every control writes the theme file the whole page is drawn from.
pub fn editor_panel<S: 'static>(t: &Theme, json: &str, app_dark: bool, g: &GalleryState, access: Access<S>) -> El {
    let e = &g.editor;
    let neutral_names: Vec<&str> = NEUTRALS.iter().map(|n| n.0).collect();
    let mut swatches = row().key("editor-accent").role("radiogroup").label("Accent").gap(t.space.at(1.5)).flex_wrap();
    for (i, hex) in ACCENTS.iter().enumerate() {
        let on = e.accent.as_deref() == Some(*hex);
        let color = rune::color::hex(hex).unwrap_or(t.c.accent.fg);
        let mut dot = row().key(format!("editor-accent-{i}")).role("radio").label((*hex).to_owned()).checked(on).focusable(true).size(22.0, 22.0).radius(11.0).fill(color).center();
        if on {
            dot = dot.box_shadow(rune::Shadow { spread: 2.0, color: t.c.page.bg, ..Default::default() }).box_shadow(rune::Shadow { spread: 3.5, color, ..Default::default() });
        }
        let pick = (*hex).to_owned();
        swatches = swatches.child(dot.on_press(move |s: &mut S| access(s).editor.accent = Some(pick.clone())));
    }
    let hex_field = paint(row().w_full().h(t.control.sm).px(t.space.at(2.0)).center_y().radius(t.radius.md).round_as(t), &t.mat.well.base).child(
        rune::text_input(g.edit("editor-hex"), t.ty.sm).key("editor-hex").single_line().grow(1.0).min_w(0.0).color(t.ink()).placeholder("#rrggbb, then Return", t.ink_faint()).label("Accent hex").on_edit(rune::edit_access(move |s: &mut S| access(s).edit_mut("editor-hex"))).on_submit(move |s: &mut S, _| {
            let g = access(s);
            let typed = g.edit("editor-hex").text.trim().to_owned();
            let typed = if typed.starts_with('#') { typed } else { format!("#{typed}") };
            if typed.len() == 7 && rune::color::hex(&typed).is_some() {
                g.editor.accent = Some(typed);
            }
        }),
    );
    let dark = g.dark.unwrap_or(app_dark);
    let export_json = rune_ui_style::serde_json::to_string_pretty(&edit_theme(json, g)).unwrap_or_default();
    let copy_json = export_json.clone();
    let buttons = row().w_full().gap(t.space.at(1.5)).flex_wrap()
        .child(pill::<S>(t, "editor-reset".to_owned(), "Reset", false, move |s: &mut S| {
            let g = access(s);
            g.editor = Editor::default();
            g.compact = false;
            g.dark = None;
            g.exported = None;
        }))
        .child(pill::<S>(t, "editor-copy".to_owned(), "Copy theme", false, |_: &mut S| {}).on_press_tx(move |_: &mut S, tx: &mut rune::Tx| tx.copy(copy_json.clone())))
        .child(pill::<S>(t, "editor-export".to_owned(), "Export", false, move |s: &mut S| {
            let dir = std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join("Downloads")).filter(|d| d.is_dir()).unwrap_or_else(std::env::temp_dir);
            let path = dir.join("rune-ui-theme.json");
            access(s).exported = Some(match std::fs::write(&path, &export_json) {
                Ok(()) => format!("Wrote {}", path.display()),
                Err(e) => format!("Could not write {}: {e}", path.display()),
            });
        }));
    let mut c = col().key("gallery-editor").role("region").label("Theme editor").w(300.0).padding(t.metrics.inset).gap(t.metrics.stack)
        .child(text("Theme").font(t.ty.lg).color(t.ink()).role("heading").level(2))
        .child(field(t, "Light or dark", choices::<S>(t, "editor-mode", &["Light", "Dark"], usize::from(dark), access, |g, i| g.dark = Some(i == 1))))
        .child(field(t, "Base neutral", choices::<S>(t, "editor-neutral", &neutral_names, e.neutral, access, |g, i| g.editor.neutral = i)))
        .child(field(t, "Accent", col().w_full().gap(t.space.at(2.0)).child(swatches).child(hex_field)))
        .child(field(t, "Radius", slider::<S>(t, "editor-radius", t.base_radius, 0.0, 16.0, format!("{:.0}", t.base_radius), access, |g, v| g.editor.radius = Some(v.round()))))
        .child(field(t, "Corners", choices::<S>(t, "editor-corners", &["Continuous", "Round"], usize::from(e.corners == Corners::Round), access, |g, i| g.editor.corners = if i == 0 { Corners::Continuous } else { Corners::Round })))
        .child(field(t, "Font", font_choices::<S>(t, e.font, access)))
        .child(field(t, "Type scale", slider::<S>(t, "editor-type", e.type_scale, 0.85, 1.25, format!("{:.2}", e.type_scale), access, |g, v| g.editor.type_scale = (v * 20.0).round() / 20.0)))
        .child(field(t, "Density", choices::<S>(t, "editor-density", &["Comfortable", "Compact"], usize::from(g.compact), access, |g, i| g.compact = i == 1)))
        .child(field(t, "Shadow strength", slider::<S>(t, "editor-shadow", e.shadow, 0.0, 2.5, format!("{:.1}", e.shadow), access, |g, v| g.editor.shadow = (v * 10.0).round() / 10.0)))
        .child(field(t, "Material: flat to tactile", slider::<S>(t, "editor-texture", e.texture, 0.0, 1.0, format!("{:.0}%", e.texture * 100.0), access, |g, v| g.editor.texture = (v * 20.0).round() / 20.0)))
        .child(buttons);
    if let Some(note) = &g.exported {
        c = c.child(text(note.clone()).font(t.ty.xs).color(t.ink_muted()).wrap().min_w(0.0));
    }
    c
}

/// One component's block: its title and each story.
pub fn item_block<S: 'static>(c: &mut Ctx<S>, item: &StoryItem<S>) -> El {
    let t = c.t;
    let mut b = col().key(format!("item-{}", item.name)).w_full().gap(t.metrics.stack - 4.0).child(row().gap(t.space.at(2.0)).items_end().child(text(item.title).font(t.ty.lg).color(t.ink()).role("heading").level(3)).child(text(item.name).font(t.ty.xs).color(t.ink_faint())));
    for st in &item.stories {
        let body = (st.build)(c);
        b = b.child(col().w_full().gap(t.space.at(2.0)).child(text(st.name).font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted())).child(body));
    }
    b
}

/// The page: the header with its controls, then the chosen group's components (or the materials sheet), and the overlays of whatever is
/// open, laid over the page.
///
/// Overlays (menus, select lists, popovers, dialogs, sheets, toasts) are pushed into `layer`, never laid out inside the page: the app
/// mounts them at its root, after its content, with `base::overlay_root(content, layer)` (that is, `rune_kit::root`). Only there do they
/// float over everything, stay anchored to their triggers through scrolling, and leave the page's layout alone.
pub fn page<S: 'static>(json: &str, app_dark: bool, g: &GalleryState, access: Access<S>, items: &[StoryItem<S>], layer: &mut Vec<El>) -> El {
    let t = theme_for(json, app_dark, g);
    let mut tabs = row().key("gallery-groups").role("tablist").label("Groups").gap(t.space.at(1.0)).flex_wrap();
    let mut shown: Vec<(&str, &str)> = GROUPS.iter().filter(|(k, _, _)| items.iter().any(|i| i.group == *k)).map(|(k, n, _)| (*k, *n)).collect();
    shown.push(("typography", "Typography"));
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
    let head = col().w_full().gap(t.space.at(3.0)).child(row().w_full().items_end().gap(t.space.at(3.0)).child(text("Components").font(t.ty.xxl).color(t.ink()).role("heading").level(1)).child(text(format!("{} theme · {} · {} components", t.name, if dark { "dark" } else { "light" }, items.len())).font(t.ty.base).color(t.ink_muted()))).child(tabs);
    let mut body = col().key("gallery").w_full().padding(t.metrics.section - 16.0).gap(t.metrics.section - 16.0).child(head);
    let mut c = Ctx { g, t: &t, layer, access };
    for (key, title, what) in GROUPS {
        let list: Vec<&StoryItem<S>> = items.iter().filter(|i| i.group == key && match &g.only { Some(o) => o == i.name, None => g.show == "all" || g.show == key }).collect();
        if list.is_empty() {
            continue;
        }
        let mut sec = col().key(format!("group-{key}")).w_full().gap(t.metrics.section - 16.0).child(col().gap(t.space.at(1.0)).child(text(title).font(t.ty.xl).color(t.ink()).role("heading").level(2)).child(text(what).font(t.ty.base).color(t.ink_muted())));
        for item in list {
            sec = sec.child(item_block(&mut c, item));
        }
        body = body.child(sec);
    }
    if g.only.is_none() && (g.show == "typography" || g.show == "all") {
        body = body.child(col().key("group-typography").w_full().gap(t.space.at(6.0)).child(text("Typography").font(t.ty.xl).color(t.ink()).role("heading").level(2)).child(typography(&t)));
    }
    if g.only.is_none() && (g.show == "materials" || g.show == "all") {
        body = body.child(col().key("group-materials").w_full().gap(t.space.at(6.0)).child(text("Materials").font(t.ty.xl).color(t.ink()).role("heading").level(2)).child(materials(&t)));
    }
    let panel = paint(editor_panel::<S>(&t, json, app_dark, g, access), &t.mat.slab.base).border_sides([0.0, 0.0, 0.0, 1.0], t.c.line);
    if g.scroll {
        // A window: the components scroll; the editor stays.
        let main = col().key("gallery-scroll").grow(1.0).min_w(0.0).h_full().scroll(rune::ScrollSpec { stick_bottom: false }).child(body);
        row().key("gallery-page").w_full().h_full().fill(t.c.page.bg).child(main).child(col().h_full().scroll(rune::ScrollSpec { stick_bottom: false }).child(panel))
    } else {
        row().key("gallery-page").w_full().fill(t.c.page.bg).items_stretch().child(col().grow(1.0).min_w(0.0).child(body)).child(panel)
    }
}
