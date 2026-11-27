//! rune-ui's gallery app: a window around the registry's `gallery` block (`registry/gallery/gallery.rs`), the same page an app gets
//! with `rune-ui add gallery`. `build.rs` compiles every registry item in place, laid out as an app's `src/ui/`, so every component and
//! its stories are built and pictured here first.
//!
//! `cargo run -p gallery` opens the window. `apps/gallery/scripts/snap.sh` renders the scenes headless to PNGs.

include!(concat!(env!("OUT_DIR"), "/registry.rs"));

use std::sync::atomic::{AtomicBool, Ordering};

use rune::{App as RuneApp, AppSpec, Cx, El, el};
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
    static ITEMS: Vec<StoryItem<App>> = ui::stories::all();
}

/// The view: the gallery page, scrolling in a window, whole in a picture; its overlays always at the root, over everything.
pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    // Light or dark, and Reduce Motion, from the system: the page writes Reduce Motion into its theme (`Theme::reduced_motion`).
    let app = ui::gallery::Appearance::of(cx.system_dark(), cx.is_reduced_motion());
    let json = rune_ui_themes::DEFAULT;
    let t = ui::gallery::theme_for(json, app, &s.g);
    // Rune's own hover tooltips, in the theme's look.
    cx.help_look(ui::tooltip::look(&t));
    let mut layer = Vec::new();
    let page = ITEMS.with(|items| ui::gallery::page(json, app, &s.g, access, items, &mut layer));
    if HEADLESS.load(Ordering::Relaxed) {
        // A picture is as tall as its content (at least a window's height, so an open list has room); the overlays are still the root's.
        el().key("canvas").w_full().min_h(840.0).fill(t.c.page.bg).child(page).children(layer)
    } else {
        // The page fills the window (its components scroll beside the theme editor); overlays at the root, over everything.
        ui::base::overlay_root(page, layer)
    }
}

/// Scenes that show an overlay open: (scene, item, overlay key).
pub const OPEN_SCENES: [(&str, &str, &str); 24] = [
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
    ("open-date-picker", "date_picker", "date-demo"),
    ("open-menubar", "menubar", "menubar-demo-0"),
    ("open-navigation-menu", "navigation_menu", "nav-demo-1"),
    ("open-hover-card", "hover_card", "hover-demo"),
    ("open-drawer", "drawer", "drawer-demo"),
    ("open-chart-bar", "chart", "chart-bar"),
    ("open-chart-donut", "chart", "chart-donut"),
    ("open-model-selector", "model_selector", "model-demo"),
    ("open-context", "context", "context-demo"),
    ("open-context-free", "context", "context-free"),
    ("open-inline-citation", "inline_citation", "cite-demo"),
    ("open-prompt-model", "prompt_input", "prompt-model"),
    ("open-dropdown-people", "dropdown_menu", "dropdown-people"),
    ("open-context-assumed", "context", "context-assumed"),
];

/// Sets up a scene by name: a group, `materials`, `all`, `item-<name>`, or one of [`OPEN_SCENES`].
pub fn apply_scene(s: &mut App, name: &str) {
    // `<scene>-compact`: the same scene at the compact density.
    let name = match name.strip_suffix("-compact") {
        Some(n) => {
            s.g.compact = Some(true);
            n
        }
        None => name,
    };
    // `<scene>-round`: the same scene with round corners instead of continuous ones.
    let name = match name.strip_suffix("-round") {
        Some(n) => {
            s.g.editor.corners = Some(rune_ui_style::Corners::Round);
            n
        }
        None => name,
    };
    // `<scene>-<font>`: the same scene in an installed font (`typography-geist`, `form-ibm-plex-sans`).
    // The longest name that fits wins, so `-geist-mono` is Geist Mono, not `mono`.
    let font = rune_ui_style::fonts::all().into_iter().filter_map(|f| name.strip_suffix(&format!("-{}", f.name)).map(|n| (n, f.name))).max_by_key(|(_, f)| f.len());
    let name = match font {
        Some((n, font)) => {
            s.g.editor.font = Some(font);
            n
        }
        None => name,
    };
    if name == "tuned" {
        // The editor at work: another neutral, a teal accent, a larger round radius, a flatter material, stronger shadows.
        s.g.editor = ui::gallery::Editor { neutral: 4, accent: Some(ui::gallery::ACCENTS[2].to_owned()), radius: Some(12.0), corners: Some(rune_ui_style::Corners::Round), texture: Some(0.4), shadow: 1.6, ..Default::default() };
        s.g.show = "actions".to_owned();
        return;
    }
    if name == "chat-open" || name == "chat-streaming" {
        // The chat block with its folds open and the model selector open, or with an answer arriving.
        s.g.only = Some("chat".to_owned());
        let c = s.g.slot_mut::<ui::chat::ChatState>("chat");
        if name == "chat-open" {
            for k in ["chat-thinking", "chat-time", "chat-steps"] {
                c.flip(k);
            }
            c.picking = true;
        } else {
            c.draft = rune::TextEdit::new("And the test suite?");
            c.send();
        }
        return;
    }
    if name == "open-toast-host" || name == "open-toast-centre" {
        // The toast host with two messages showing, one with an action; the centre one at the bottom centre.
        s.g.only = Some("toast".to_owned());
        if name == "open-toast-centre" {
            s.g.flags.insert("toast-centre".to_owned(), true);
        }
        let host = s.g.slot_mut::<ui::toast::ToastHost>("toast-host");
        host.push(ui::toast::ToastSpec::new("Archived 3 threads").tone(rune_ui_style::Tone::Success).action("Undo", "undo-archive"));
        host.push(ui::toast::ToastSpec::new("Thread settled").description("Fix the login redirect").action("Undo", "undo-settle"));
        return;
    }
    if let Some(scene) = WAVE3_SCENES.iter().find(|n| **n == name) {
        wave3_scene(s, scene);
        return;
    }
    if let Some((_, item, key)) = OPEN_SCENES.iter().find(|(n, _, _)| *n == name) {
        s.g.only = Some((*item).to_owned());
        s.g.open = Some((*key).to_owned());
        if *item == "context_menu" {
            s.g.point = Some(rune::PointerPress { x: 260.0, y: 330.0 });
        }
        if *item == "toast" {
            s.g.toasts = vec!["toast-demo".to_owned()];
        }
    } else if let Some(item) = name.strip_prefix("item-") {
        s.g.only = Some(item.to_owned());
    } else {
        s.g.show = name.to_owned();
    }
}

/// Wave 3's open states, for headless pictures.
pub const WAVE3_SCENES: [&str; 6] = ["open-toolbar-more", "open-prompt-mention", "open-prompt-command", "open-stack-trace", "open-tree", "segmented-moved"];

fn wave3_scene(s: &mut App, name: &str) {
    match name {
        "open-toolbar-more" => {
            s.g.only = Some("toolbar".to_owned());
            s.g.slot_mut::<ui::toolbar::ToolbarState>("toolbar-narrow").more_open = true;
        }
        "open-prompt-mention" | "open-prompt-command" => {
            s.g.only = Some("prompt_input".to_owned());
            let words = if name == "open-prompt-mention" { "Ask @" } else { "/re" };
            let e = s.g.edit_mut("prompt-menu");
            *e = rune::TextEdit::new(words);
            e.focused = true;
        }
        "open-stack-trace" => {
            s.g.only = Some("stack_trace".to_owned());
            s.g.flags.insert("trace-libs".to_owned(), true);
        }
        "open-tree" => {
            s.g.only = Some("tree".to_owned());
            let st = s.g.slot_mut::<ui::tree::TreeState>("tree-demo");
            for id in ["plans", "launch", "people", "archive"] {
                st.expanded.insert(id.to_owned());
            }
            st.selected.insert("checklist".to_owned());
            st.cursor = Some("checklist".to_owned());
        }
        _ => {
            s.g.only = Some("segmented_control".to_owned());
            s.g.choose("seg-demo", "2");
            s.g.choose("seg-view", "3");
        }
    }
}

macro_rules! scenes {
    ($($n:literal),* $(,)?) => { &[$(($n, |s: &mut App| apply_scene(s, $n))),*] };
}

/// Named states for headless pictures (`--scene`); one item alone with `RUNE_UI_ITEM=<name>`.
/// A named scene and how it sets up the state.
pub type Scene = (&'static str, fn(&mut App));

pub const SCENES: &[Scene] = scenes!["actions", "form", "overlays", "structure", "display", "materials", "materials-round", "typography", "typography-geist", "typography-ibm-plex-sans", "typography-figtree", "typography-manrope", "typography-dm-sans", "typography-plus-jakarta-sans", "typography-jetbrains-mono", "form-geist", "actions-geist", "all", "actions-compact", "form-compact", "structure-compact", "overlays-compact", "display-compact", "form-round", "structure-round", "tuned", "open-select", "open-combobox", "open-context-menu", "open-alert-dialog", "open-toast", "open-dropdown-menu", "open-popover", "open-dialog", "open-sheet", "open-command", "open-date-picker", "open-menubar", "open-navigation-menu", "open-hover-card", "open-drawer", "open-chart-bar", "open-chart-donut", "agent", "agent-compact", "agent-round", "agent-geist", "agent-serif", "chat-open", "chat-streaming", "open-model-selector", "open-context", "open-context-free", "open-inline-citation", "open-prompt-model", "open-dropdown-people", "open-context-assumed", "open-toolbar-more", "open-prompt-mention", "open-prompt-command", "open-stack-trace", "open-tree", "segmented-moved", "open-tree-compact", "segmented-moved-round", "open-stack-trace-geist", "open-toolbar-more-compact", "open-toast-host", "open-toast-centre"];

static HEADLESS: AtomicBool = AtomicBool::new(false);

/// Marks this process as rendering pictures (the page is drawn whole, not in a scroll view).
pub fn set_headless(on: bool) {
    HEADLESS.store(on, Ordering::Relaxed);
}

/// Writes an exported theme file to `~/Downloads/rune-ui-theme.json` (the temporary folder without one) and says where.
pub fn export_theme(json: &str) -> String {
    let dir = std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join("Downloads")).filter(|d| d.is_dir()).unwrap_or_else(std::env::temp_dir);
    let path = dir.join("rune-ui-theme.json");
    match std::fs::write(&path, json) {
        Ok(()) => format!("Wrote {}", path.display()),
        Err(e) => format!("Could not write {}: {e}", path.display()),
    }
}

fn init() -> App {
    let mut s = App::default();
    s.g.scroll = !HEADLESS.load(Ordering::Relaxed);
    // The window app's Export: the theme file goes to ~/Downloads (the page itself never writes files).
    s.g.on_export = Some(std::rc::Rc::new(export_theme));
    if let Ok(i) = std::env::var("RUNE_UI_ITEM") {
        apply_scene(&mut s, &format!("item-{i}"));
    }
    s
}

/// The app's poll hook: the toast host starts, pauses and ends its timers from the window's clock.
fn poll(s: &mut App, clock: f64, tx: &mut rune::Tx) -> bool {
    s.g.slot_mut::<ui::toast::ToastHost>("toast-host").poll(clock, tx)
}

pub fn run() {
    set_headless(std::env::args().any(|a| a == "--snap" || a == "--tree"));
    // The font items: registered with Rune and made nameable by the theme (the editor's Font control lists them).
    let fonts = rune_ui_style::fonts::install(ui::FONTS);
    let spec = AppSpec { poll: Some(poll), scenes: SCENES, fonts, min_size: Some((720.0, 480.0)), ..AppSpec::new("rune-ui gallery", (1320.0, 900.0), init, view) };
    let app = RuneApp::new(spec);
    if HEADLESS.load(Ordering::Relaxed) {
        app.fit_content_height().run()
    } else {
        // Rune at this rev makes a symbol image on a launch thread, and AppKit's once-per-process Dock registration can land on that
        // thread: the Dock then heads the app's menu "Application Not Responding" (README, "Known Rune issue on macOS"). Rune's own
        // switch keeps the launch on the main thread; this small tool does not need the faster launch.
        // SAFETY: no other thread exists yet (nothing has been spawned before `run`), so nothing reads the environment concurrently.
        #[cfg(target_os = "macos")]
        unsafe {
            std::env::set_var("RUNE_LAUNCH_PLAIN", "1");
        }
        app.run()
    }
}
