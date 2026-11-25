//! rune-ui's gallery app: a window around the registry's `gallery` block (`registry/gallery/gallery.rs`), the same page an app gets
//! with `rune-ui add gallery`. `build.rs` compiles every registry item in place, laid out as an app's `src/ui/`, so every component and
//! its stories are built and pictured here first.
//!
//! `cargo run -p gallery` opens the window. `apps/gallery/scripts/snap.sh` renders the scenes headless to PNGs.

include!(concat!(env!("OUT_DIR"), "/registry.rs"));

use std::sync::atomic::{AtomicBool, Ordering};

use rune::{App as RuneApp, AppSpec, Cx, El, col, el};
use rune_ui_style::ThemeFile;
use ui::gallery::{GalleryState, StoryItem};

/// The app's state: only the gallery's.
#[derive(Default)]
pub struct App {
    pub g: GalleryState,
}

fn access(s: &mut App) -> &mut GalleryState {
    &mut s.g
}

thread_local! {
    static FILE: ThemeFile = ThemeFile::parse(rune_ui_themes::DEFAULT).expect("the default theme is valid (tested)");
    static ITEMS: Vec<StoryItem<App>> = ui::stories::all();
}

/// The view: the gallery page, scrolling in a window, whole in a picture; its overlays always at the root, over everything.
pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = cx.system_dark();
    let file = FILE.with(Clone::clone);
    let t = ui::gallery::theme_for(&file, dark, &s.g);
    // Rune's own hover tooltips, in the theme's look.
    cx.help_look(ui::tooltip::look(&t));
    let mut layer = Vec::new();
    let page = ITEMS.with(|items| ui::gallery::page(&file, dark, &s.g, access, items, &mut layer));
    if HEADLESS.load(Ordering::Relaxed) {
        // A picture is as tall as its content (at least a window's height, so an open list has room); the overlays are still the root's.
        el().key("canvas").w_full().min_h(840.0).fill(t.c.page.bg).child(page).children(layer)
    } else {
        ui::base::overlay_root(col().key("scroll").w_full().h_full().fill(t.c.page.bg).scroll(rune::ScrollSpec { stick_bottom: false }).child(page), layer)
    }
}

/// Scenes that show an overlay open: (scene, item, overlay key).
pub const OPEN_SCENES: [(&str, &str, &str); 10] = [
    ("open-context-menu", "context_menu", "context-demo"),
    ("open-alert-dialog", "alert_dialog", "alert-demo"),
    ("open-toast", "toast", "toast-demo"),
    ("open-select", "select", "select-demo"),
    ("open-combobox", "combobox", "combobox-demo"),
    ("open-dropdown-menu", "dropdown_menu", "dropdown-demo"),
    ("open-popover", "popover", "popover-demo"),
    ("open-dialog", "dialog", "dialog-demo"),
    ("open-sheet", "sheet", "sheet-demo"),
    ("open-command", "command", "command-demo"),
];

/// Sets up a scene by name: a group, `materials`, `all`, `item-<name>`, or one of [`OPEN_SCENES`].
pub fn apply_scene(s: &mut App, name: &str) {
    // `<scene>-compact`: the same scene at the compact density.
    let name = match name.strip_suffix("-compact") {
        Some(n) => {
            s.g.compact = true;
            n
        }
        None => name,
    };
    if let Some((_, item, key)) = OPEN_SCENES.iter().find(|(n, _, _)| *n == name) {
        s.g.only = Some((*item).to_owned());
        s.g.open = Some((*key).to_owned());
        if *item == "context_menu" {
            s.g.point = Some(rune::PointerPress { x: 260.0, y: 330.0 });
        }
        if *item == "toast" {
            s.g.toasts = vec!["toast-demo".to_owned()];
        }
        if *item == "combobox" {
            let _ = s.g.combo_mut(key).step(rune_kit::ComboboxEvent::Toggle { current: None });
        }
    } else if let Some(item) = name.strip_prefix("item-") {
        s.g.only = Some(item.to_owned());
    } else {
        s.g.show = name.to_owned();
    }
}

macro_rules! scenes {
    ($($n:literal),* $(,)?) => { &[$(($n, |s: &mut App| apply_scene(s, $n))),*] };
}

/// Named states for headless pictures (`--scene`); one item alone with `RUNE_UI_ITEM=<name>`.
/// A named scene and how it sets up the state.
pub type Scene = (&'static str, fn(&mut App));

pub const SCENES: &[Scene] = scenes!["actions", "form", "overlays", "structure", "display", "materials", "all", "actions-compact", "form-compact", "structure-compact", "open-select", "open-combobox", "open-context-menu", "open-alert-dialog", "open-toast", "open-dropdown-menu", "open-popover", "open-dialog", "open-sheet", "open-command"];

static HEADLESS: AtomicBool = AtomicBool::new(false);

/// Marks this process as rendering pictures (the page is drawn whole, not in a scroll view).
pub fn set_headless(on: bool) {
    HEADLESS.store(on, Ordering::Relaxed);
}

fn init() -> App {
    let mut s = App::default();
    if let Ok(i) = std::env::var("RUNE_UI_ITEM") {
        apply_scene(&mut s, &format!("item-{i}"));
    }
    s
}

pub fn run() {
    set_headless(std::env::args().any(|a| a == "--snap" || a == "--tree"));
    let spec = AppSpec { scenes: SCENES, min_size: Some((720.0, 480.0)), ..AppSpec::new("rune-ui gallery", (1320.0, 900.0), init, view) };
    let app = RuneApp::new(spec);
    if HEADLESS.load(Ordering::Relaxed) { app.fit_content_height().run() } else { app.run() }
}
