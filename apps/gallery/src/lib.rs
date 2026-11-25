//! The rune-ui gallery: every component in every state, light and dark, and a materials sheet that builds each recipe one layer at a
//! time. States are pinned (`Look`) so a picture shows hover, press and focus without a pointer. Placeholder data only.
//!
//! `cargo run -p rune-ui-gallery` opens the window; `apps/gallery/scripts/snap.sh` renders every scene headless to PNGs.

pub mod ui;

use std::sync::atomic::{AtomicBool, Ordering};

use rune::{App, AppSpec, Cx, El, TextEdit, col, el, row, text};
use rune_ui_style::{Layer, LayerKind, Look, Material, Recipe, Theme, paint, weight};

use ui::badge::{Badge, Mark, StatusPill, Tone};
use ui::base::Size;
use ui::button::{Button, Variant};
use ui::card::{self, Card, SectionHeader};
use ui::chip::Chip;
use ui::icons;
use ui::segmented::Segmented;
use ui::select::Select;
use ui::switch::Switch;
use ui::text_field::{SearchField, TextField};

/// Which part of the gallery is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Group {
    #[default]
    All,
    Buttons,
    Fields,
    Selects,
    Toggles,
    Chips,
    Cards,
    Materials,
}

impl Group {
    pub const PARTS: [Group; 7] = [Group::Buttons, Group::Fields, Group::Selects, Group::Toggles, Group::Chips, Group::Cards, Group::Materials];

    pub fn word(self) -> &'static str {
        match self {
            Group::All => "All",
            Group::Buttons => "Buttons",
            Group::Fields => "Text fields",
            Group::Selects => "Select",
            Group::Toggles => "Switch and segmented",
            Group::Chips => "Chips and badges",
            Group::Cards => "Cards",
            Group::Materials => "Materials",
        }
    }
}

/// The gallery's state: what the live specimens hold.
pub struct State {
    pub group: Group,
    pub presses: u32,
    pub on: bool,
    pub seg: usize,
    pub filter: bool,
    pub name: TextEdit,
    pub budget: TextEdit,
    pub query: TextEdit,
    pub empty: TextEdit,
    pub levels: Vec<String>,
    pub level: String,
    /// The select whose list is open.
    pub open: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        State {
            group: Group::All,
            presses: 0,
            on: true,
            seg: 1,
            filter: true,
            name: TextEdit::new("Placeholder project"),
            budget: TextEdit::new("12.50"),
            query: TextEdit::new("theme"),
            empty: TextEdit::new(""),
            levels: ["Careful", "Balanced", "Hands-off"].map(str::to_owned).to_vec(),
            level: "Balanced".to_owned(),
            open: None,
        }
    }
}

impl State {
    fn toggle_select(&mut self, key: &str) {
        self.open = if self.open.as_deref() == Some(key) { None } else { Some(key.to_owned()) };
    }
}

/// A specimen with its caption under it.
fn spec(t: &Theme, caption: &str, e: El) -> El {
    col().gap(t.space.at(2.0)).items_start().child(e).child(text(caption.to_owned()).font(t.ty.xs).color(t.c.ink_faint))
}

/// A labelled line of specimens.
fn line(t: &Theme, label: &str, items: Vec<El>) -> El {
    let mut r = row().gap(t.space.at(6.0)).items_start().child(col().w(104.0).pt(t.space.at(1.5)).child(text(label.to_owned()).font(weight(t.ty.sm, t.ty.medium)).color(t.c.ink_muted)));
    for i in items {
        r = r.child(i);
    }
    r
}

fn section(t: &Theme, g: Group, description: &str, body: El) -> El {
    col().key(format!("group-{}", g.word().to_lowercase().replace(' ', "-"))).w_full().gap(t.space.at(2.0)).child(SectionHeader::new(g.word()).description(description).build(t)).child(body)
}

fn buttons(t: &Theme) -> El {
    let mut c = col().gap(t.space.at(5.0));
    for v in Variant::ALL {
        let mk = |k: &str| Button::<State>::new(v, format!("btn-{}-{k}", v.word().to_lowercase()), "Save changes").on_press(|s: &mut State| s.presses += 1);
        let mut items: Vec<El> = Look::ALL.iter().map(|l| spec(t, l.word(), mk(l.word()).look(*l).build(t))).collect();
        items.push(spec(t, "Disabled", mk("disabled").disabled(true).build(t)));
        items.push(spec(t, "Loading", mk("loading").loading(true).build(t)));
        c = c.child(line(t, v.word(), items));
    }
    let sizes: Vec<El> = Size::ALL.iter().map(|z| spec(t, z.word(), Button::<State>::secondary(format!("btn-size-{}", z.word()), "Continue").size(*z).trailing_icon(&icons::ARROW_RIGHT).build(t))).collect();
    c = c.child(line(t, "Sizes", sizes));
    c.child(line(
        t,
        "Shapes",
        vec![
            spec(t, "Icon and words", Button::<State>::primary("btn-icon", "New item").icon(&icons::PLUS).build(t)),
            spec(t, "Icon only", Button::<State>::ghost("btn-icon-only", "Add").icon_only(&icons::PLUS).build(t)),
            spec(t, "Selected", Button::<State>::secondary("btn-selected", "Pinned").selected(true).build(t)),
            spec(t, "Ghost selected", Button::<State>::ghost("btn-ghost-selected", "Board").selected(true).build(t)),
            spec(t, "Destructive", Button::<State>::destructive("btn-delete", "Delete").icon(&icons::TRASH).build(t)),
        ],
    ))
}

fn fields(s: &State, t: &Theme) -> El {
    let w = 220.0;
    let f = |key: &str| TextField::<State>::new(key.to_owned(), &s.name, |s: &mut State| &mut s.name).width(w);
    col().gap(t.space.at(5.0))
        .child(line(t, "Text field", vec![
            spec(t, "Default", f("field-rest").build(t)),
            spec(t, "Focused", f("field-focused").look(Look::Focused).build(t)),
            spec(t, "Placeholder", TextField::<State>::new("field-empty", &s.empty, |s: &mut State| &mut s.empty).placeholder("Project name").width(w).build(t)),
            spec(t, "Disabled", f("field-disabled").disabled(true).build(t)),
        ]))
        .child(line(t, "With words", vec![
            spec(t, "Label and help", TextField::<State>::new("field-budget", &s.budget, |s: &mut State| &mut s.budget).label("Budget").prefix("$").suffix("per day").help("Stops work that spends more").width(w).build(t)),
            spec(t, "Error", TextField::<State>::new("field-error", &s.empty, |s: &mut State| &mut s.empty).label("Name").placeholder("Project name").error(Some("A name is needed".to_owned())).width(w).build(t)),
            spec(t, "Clearable", TextField::<State>::new("field-clear", &s.name, |s: &mut State| &mut s.name).clearable().width(w).build(t)),
        ]))
        .child(line(t, "Search", vec![
            spec(t, "Empty", SearchField::<State>::new("search-empty", &s.empty, |s: &mut State| &mut s.empty).placeholder("Find a setting").hint("⌘F").width(w).build(t)),
            spec(t, "With text", SearchField::<State>::new("search-query", &s.query, |s: &mut State| &mut s.query).width(w).build(t)),
            spec(t, "Small", SearchField::<State>::new("search-sm", &s.empty, |s: &mut State| &mut s.empty).size(Size::Sm).width(180.0).build(t)),
        ]))
}

fn selects(s: &State, t: &Theme, layer: &mut Vec<El>) -> El {
    let mk = |key: &str| {
        let (k1, k2) = (key.to_owned(), key.to_owned());
        Select::<State>::new(key.to_owned(), "Starting level", &s.levels)
            .current(&s.level)
            .width(180.0)
            .open(s.open.as_deref() == Some(key))
            .on_toggle(move |s: &mut State| s.toggle_select(&k1))
            .on_close(move |s: &mut State| {
                if s.open.as_deref() == Some(k2.as_str()) {
                    s.open = None;
                }
            })
            .on_choose(|s: &mut State, v: &str| {
                s.level = v.to_owned();
                s.open = None;
            })
    };
    col().gap(t.space.at(5.0)).child(line(t, "Select", vec![
        spec(t, "Default", mk("select-rest").build(t, layer)),
        spec(t, "Focused", mk("select-focused").look(Look::Focused).build(t, layer)),
        spec(t, "Placeholder", Select::<State>::new("select-empty", "Model", &s.levels).placeholder("Choose a level").width(180.0).build(t, layer)),
        spec(t, "Error", mk("select-error").error(true).build(t, layer)),
        spec(t, "Disabled", mk("select-disabled").disabled(true).build(t, layer)),
    ]))
    // Room under the specimens for an open list.
    .child(el().h(if s.open.is_some() { 120.0 } else { 0.0 }))
}

fn toggles(s: &State, t: &Theme) -> El {
    let sw = |key: &str, on: bool| Switch::<State>::new(key.to_owned(), "Sync", on).on_toggle(|s: &mut State| s.on = !s.on);
    let opts: [(&str, Option<&'static rune::Icon>); 3] = [("List", None), ("Board", None), ("Calendar", None)];
    let seg = |key: &str| Segmented::new(key.to_owned(), "View", &opts, s.seg);
    col().gap(t.space.at(5.0))
        .child(line(t, "Switch", vec![
            spec(t, "Off", sw("switch-off", false).build(t)),
            spec(t, "On", sw("switch-on", true).build(t)),
            spec(t, "Focused", sw("switch-focused", s.on).look(Look::Focused).build(t)),
            spec(t, "Disabled off", sw("switch-disabled-off", false).disabled(true).build(t)),
            spec(t, "Disabled on", sw("switch-disabled-on", true).disabled(true).build(t)),
        ]))
        .child(line(t, "With words", vec![col().w(320.0).child(sw("switch-line", s.on).label("Sync in the background").help("Keeps placeholder data fresh").build(t))]))
        .child(line(t, "Segmented", vec![
            spec(t, "Default", seg("seg-rest").build(t, |s: &mut State, i| s.seg = i)),
            spec(t, "Hover on Board", seg("seg-hover").look(0, Look::Hover).build(t, |s: &mut State, i| s.seg = i)),
            spec(t, "Focused", seg("seg-focused").look(1, Look::Focused).build(t, |s: &mut State, i| s.seg = i)),
            spec(t, "Small", seg("seg-sm").size(Size::Sm).build(t, |s: &mut State, i| s.seg = i)),
            spec(t, "Disabled", seg("seg-disabled").disabled(true).build(t, |s: &mut State, i| s.seg = i)),
        ]))
}

fn chips(s: &State, t: &Theme) -> El {
    let filter = |key: &str, words: &str| Chip::<State>::new(key.to_owned(), words.to_owned()).on_press(|s: &mut State| s.filter = !s.filter);
    let tones: Vec<El> = Tone::ALL.iter().map(|tone| spec(t, tone.word(), StatusPill::new(format!("pill-{}", tone.word()), *tone, tone.word()).build(t))).collect();
    let counts: Vec<El> = Tone::ALL.iter().map(|tone| spec(t, tone.word(), Badge::count(format!("count-{}", tone.word()), 3).tone(*tone).build(t))).collect();
    col().gap(t.space.at(5.0))
        .child(line(t, "Tag", vec![
            spec(t, "Plain", Chip::<State>::new("tag-plain", "docs").build(t)),
            spec(t, "With icon", Chip::<State>::new("tag-icon", "release").icon(&icons::TAG).build(t)),
            spec(t, "Removable", Chip::<State>::new("tag-remove", "draft").on_remove(|_: &mut State| {}).build(t)),
        ]))
        .child(line(t, "Filter chip", vec![
            spec(t, "Default", filter("chip-rest", "Open").build(t)),
            spec(t, "Hover", filter("chip-hover", "Open").look(Look::Hover).build(t)),
            spec(t, "Pressed", filter("chip-pressed", "Open").look(Look::Pressed).build(t)),
            spec(t, "Focused", filter("chip-focused", "Open").look(Look::Focused).build(t)),
            spec(t, "Selected", filter("chip-selected", "Open").selected(s.filter).build(t)),
            spec(t, "Disabled", filter("chip-disabled", "Open").disabled(true).build(t)),
        ]))
        .child(line(t, "Status pill", tones))
        .child(line(t, "Marks", vec![
            spec(t, "Working", StatusPill::new("pill-working", Tone::Info, "Working").mark(Mark::Working).build(t)),
            spec(t, "Draft", StatusPill::new("pill-draft", Tone::Neutral, "Draft").mark(Mark::Hollow).build(t)),
        ]))
        .child(line(t, "Count", counts))
        .child(line(t, "Loud and dot", vec![
            spec(t, "Solid", Badge::count("count-solid", 120).solid().build(t)),
            spec(t, "Dot", Badge::dot("dot").build(t)),
        ]))
}

fn cards(s: &State, t: &Theme) -> El {
    let sw = Switch::<State>::new("card-sync", "Sync", s.on).on_toggle(|s: &mut State| s.on = !s.on).build(t);
    let more = Button::<State>::secondary("card-manage", "Manage").size(Size::Sm).build(t);
    let pill = StatusPill::new("card-pill", Tone::Success, "Connected").build(t);
    let slab = Card::slab("card-slab").rows(vec![
        card::line(t, "card-row-sync", "Sync in the background", Some("Keeps placeholder data fresh"), Some(sw)),
        card::line(t, "card-row-account", "Placeholder account", Some("Signed in as someone@example.com"), Some(pill)),
        card::line(t, "card-row-data", "Stored data", Some("12 MB on this computer"), Some(more)),
    ]);
    let raised = Card::raised("card-raised")
        .child(col().gap(t.space.at(1.5)).child(text("A result that stands out").font(t.ty.lg).color(t.c.ink)).child(text("Raised cards lift a little; slab cards sit flat on the page and are split by hairlines.").font(t.ty.base).color(t.c.ink_muted).wrap().min_w(0.0)))
        .child(row().pt(t.space.at(4.0)).gap(t.space.at(2.0)).child(Button::<State>::primary("card-open", "Open").build(t)).child(Button::<State>::ghost("card-dismiss", "Dismiss").build(t)));
    row().gap(t.space.at(8.0)).items_start().child(col().w(440.0).child(SectionHeader::new("General").description("A slab of rows").build(t)).child(slab.build(t))).child(col().w(360.0).pt(t.space.at(12.0)).child(raised.build(t)))
}

/// One recipe built up a layer at a time: what each layer adds.
fn layers(t: &Theme, name: &str, m: &Material, w: f32, h: f32, r: f32) -> El {
    let rest = &m.rest;
    let mut items = Vec::new();
    let mut so_far = Recipe::new();
    for (i, l) in rest.layers.iter().enumerate() {
        so_far = so_far.layer(l.clone());
        // Consecutive drop shadows are one step (a smooth stack).
        let next_is_drop = rest.layers.get(i + 1).is_some_and(|n| n.kind() == LayerKind::Shadow);
        if l.kind() == LayerKind::Shadow && next_is_drop {
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
    let states: Vec<El> = [("Pressed", &m.pressed), ("Focused", &m.focused), ("Disabled", &m.disabled), ("Selected", &m.selected)]
        .into_iter()
        .filter(|(_, rr)| *rr != rest)
        .map(|(word, rr)| spec(t, word, paint(el().size(w, h).radius(r).continuous_corners(), rr)))
        .collect();
    let mut all = items;
    if !states.is_empty() {
        all.push(el().w(1.0).h(h).fill(t.c.line));
        all.extend(states);
    }
    line(t, name, all)
}

fn materials(t: &Theme) -> El {
    let (w, h, r) = (96.0, t.control.lg, rune_ui_style::Radius::fit(t.radius.lg, t.control.lg));
    let mut c = col().gap(t.space.at(6.0));
    for (name, m) in t.mat.all() {
        let (mw, mh, mr) = match name {
            "floating" => (120.0, 64.0, t.radius.lg),
            "slab" => (120.0, 64.0, t.radius.xl),
            "thumb" => (24.0, 24.0, 12.0),
            "chip" => (72.0, 24.0, 12.0),
            _ => (w, h, r),
        };
        c = c.child(layers(t, name, m, mw, mh, mr));
    }
    c.child(line(t, "atmosphere", vec![spec(t, "fill + grain (skipped: Rune has no grain primitive)", paint(el().size(240.0, 64.0).radius(t.radius.xl), &t.mat.atmosphere))]))
}

/// The gallery's one view.
pub fn view(s: &State, cx: &mut Cx<State>) -> El {
    let t = rune_ui_themes::theme(cx.system_dark());
    let mut layer = Vec::new();
    let wanted = |g: Group| s.group == Group::All || s.group == g;
    let mut body = col().key("gallery").w_full().padding(t.space.at(10.0)).gap(t.space.at(12.0)).child(
        col().gap(t.space.at(1.0)).child(text("rune-ui").font(t.ty.xxl).color(t.c.ink).role("heading").level(1)).child(text(format!("Every component in every state · {} theme · {}", t.name, s.group.word())).font(t.ty.base).color(t.c.ink_muted)),
    );
    for g in Group::PARTS.into_iter().filter(|g| wanted(*g)) {
        let (what, part) = match g {
            Group::Buttons => ("Ink, raised, ghost and danger materials; pinned states, sizes and shapes", buttons(&t)),
            Group::Fields => ("Wells: a tint pressed into the page, a ring on focus", fields(s, &t)),
            Group::Selects => ("A well trigger over rune-kit's listbox", selects(s, &t, &mut layer)),
            Group::Toggles => ("Wells holding a raised thumb", toggles(s, &t)),
            Group::Chips => ("Colour is state, at low alpha, never alone", chips(s, &t)),
            Group::Cards => ("One surface divided into rows", cards(s, &t)),
            Group::Materials => ("Each recipe, one layer at a time, then its states", materials(&t)),
            Group::All => unreachable!(),
        };
        body = body.child(section(&t, g, what, part));
    }
    let page = col().key("page").w_full().fill(t.c.page).child(body);
    // A picture is as tall as the gallery (the window fits its content); a window scrolls.
    let page = if HEADLESS.load(Ordering::Relaxed) { page } else { col().key("scroll").w_full().h_full().fill(t.c.page).clip_rounded(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(page) };
    rune_kit::root(page, layer)
}

/// Named states for headless pictures (`--scene`).
pub const SCENES: &[(&str, fn(&mut State))] = &[
    ("all", |_| {}),
    ("buttons", |s| s.group = Group::Buttons),
    ("fields", |s| s.group = Group::Fields),
    ("selects", |s| s.group = Group::Selects),
    ("select-open", |s| {
        s.group = Group::Selects;
        s.open = Some("select-rest".to_owned());
    }),
    ("toggles", |s| s.group = Group::Toggles),
    ("chips", |s| s.group = Group::Chips),
    ("cards", |s| s.group = Group::Cards),
    ("materials", |s| s.group = Group::Materials),
];

/// Whether this run renders pictures (`--snap`, `--tree`) rather than opening a window.
static HEADLESS: AtomicBool = AtomicBool::new(false);

pub fn run() {
    let headless = std::env::args().any(|a| a == "--snap" || a == "--tree");
    HEADLESS.store(headless, Ordering::Relaxed);
    let spec = AppSpec { scenes: SCENES, min_size: Some((640.0, 240.0)), ..AppSpec::new("rune-ui gallery", (1280.0, 900.0), State::default, view) };
    let app = App::new(spec);
    if headless { app.fit_content_height().run() } else { app.run() }
}
