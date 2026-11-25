//! rune-ui's gallery app: a window around the registry's `gallery` block (`registry/gallery/gallery.rs`), the same page an app gets
//! with `rune-ui add gallery`. `build.rs` compiles every registry item in place, laid out as an app's `src/ui/`, so every component and
//! its stories are built and pictured here first.
//!
//! `cargo run -p gallery` opens the window. `apps/gallery/scripts/snap.sh` renders the scenes headless to PNGs.

include!(concat!(env!("OUT_DIR"), "/registry.rs"));

use std::sync::atomic::{AtomicBool, Ordering};

use rune::{App as RuneApp, AppSpec, Cx, El, col};
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

/// The view: the gallery page, scrolling in a window, whole in a picture.
pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = cx.system_dark();
    let file = FILE.with(Clone::clone);
    let t = ui::gallery::theme_for(&file, dark, &s.g);
    // Rune's own hover tooltips, in the theme's look.
    cx.help_look(ui::tooltip::look(&t));
    let page = ITEMS.with(|items| ui::gallery::page(&file, dark, &s.g, access, items));
    if HEADLESS.load(Ordering::Relaxed) { page } else { col().key("scroll").w_full().h_full().fill(t.c.page.bg).clip_rounded(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(page) }
}

/// Scenes that show an overlay open: (scene, item, overlay key).
pub const OPEN_SCENES: [(&str, &str, &str); 7] = [
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
    if let Some((_, item, key)) = OPEN_SCENES.iter().find(|(n, _, _)| *n == name) {
        s.g.only = Some((*item).to_owned());
        s.g.open = Some((*key).to_owned());
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
pub const SCENES: &[(&str, fn(&mut App))] = scenes!["actions", "form", "overlays", "structure", "display", "materials", "all", "open-select", "open-combobox", "open-dropdown-menu", "open-popover", "open-dialog", "open-sheet", "open-command"];

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
