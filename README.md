# rune-ui

rune-ui does for Rune what Tailwind and shadcn/ui do for React. You get a style system, components that you copy into your app and then own, and themes stored as data. Rune is a GPU-rendered Rust UI framework.

MIT licensed. It lives locally for now and has no remote. Rune is pinned to `bc05f817` (see `RUNE_REV`).

## The layers

| # | Layer | Here | Like |
| --- | --- | --- | --- |
| 0 | Rune | git dependency, pinned | React |
| 1 | Behaviour | Rune's `rune-kit`, same rev | Radix |
| 2 | Style | `crates/rune-ui-style` | Tailwind |
| 3 | Components | `registry/` | shadcn/ui |
| 4 | Themes | `crates/rune-ui-themes` | shadcn themes |
| 5 | Apps | your `src/ui/` | |

`ARCHITECTURE.md` is the rule book. It covers the behaviour/paint seam, the three token tiers, how recipes work, and how items are tested. `docs/principles.md` sets out the design theory, and `docs/recipes.md` is the reference for every material, state, and token.

## Use it in an app

```sh
cargo run -p rune-ui-cli -- init --app ../my-app          # deps, Rune's [patch.crates-io] table, src/ui/{base,icons}.rs, rune-ui.lock
cargo run -p rune-ui-cli -- add button select dialog --app ../my-app
cargo run -p rune-ui-cli -- diff --check --app ../my-app  # CI: exit 1 if a copy drifted from rune-ui.lock
```

Then add `mod ui;` to your crate and build from the copies:

```rust
let t = rune_ui_themes::theme(cx.system_dark());
ui::button::Button::new("save", "Save").on_press(|s: &mut App| s.save()).build(&t)
```

### What an app must mount: one overlay host at the root

Anything that opens over the page (Select, Combobox, DropdownMenu, ContextMenu, Popover, Command, Dialog, AlertDialog, Sheet, Toast)
does not lay itself out where it is built. It pushes its surface into a `layer: &mut Vec<El>` that the view collects, and the view mounts
that layer **once, at its root, after the content**:

```rust
fn view(s: &App, cx: &mut Cx<App>) -> El {
    let mut layer = Vec::new();
    let content = col().key("scroll").w_full().h_full().scroll(rune::ScrollSpec { stick_bottom: false })
        .child(page(s, &t, &mut layer));           // components get `&mut layer`
    ui::base::overlay_root(content, layer)          // = rune_kit::root: overlays over everything
}
```

Mounted there, overlays are out of the layout: they never move or scroll what is under them, they stay anchored to their trigger as the
page scrolls, flip above a trigger near the window's bottom, stay inside the window, close on Escape and on a press outside, and give
focus back to their trigger. Dialogs, alert dialogs, sheets and the command palette add a scrim and trap focus (`El::modal`); give them
`.return_focus(trigger_key)`. Mounted anywhere else (inside the scroll view, inside a row) they would be laid out in place. Tooltips are
Rune's own (`El::help`) and need no layer.

### The CLI

| Command | What it does |
| --- | --- |
| `rune-ui init [--source <repo> \| --git <url> --tag <tag>] [--dry-run]` | Adds `rune`, `rune-kit`, `rune-ui-style` and `rune-ui-themes`, plus Rune's patch table, which Cargo does not inherit. Copies `base` and `icons`, then writes the lock. |
| `rune-ui add <item>... [--dry-run] [--force]` | Copies the items and everything they need into `src/ui/` and regenerates `src/ui/mod.rs`. It never overwrites a file it did not write, or one you changed, unless you pass `--force`. Two items writing the same path is an error. |
| `rune-ui list` | Lists every item, with its version, type and group, and the status of your copy: ok, changed, or has an update. |
| `rune-ui diff [<item>...] [--check]` | Shows a unified diff between the registry and your copy. With `--check`, exits 1 on drift. |
| `rune-ui update [<item>...]` | Merges the registry's changes into your edits in three ways, using the base copy it kept in `.rune-ui/base/`. Conflicts get markers and the command exits 1. |
| `rune-ui registry hash` | For maintainers: refreshes the `sha256` values in every `item.toml`. |

The registry is embedded in the binary. `--registry <dir>` reads it from a folder instead.

## The gallery

```sh
cargo run -p gallery                      # a window: every component in every state, light and dark, two densities
apps/gallery/scripts/snap.sh              # headless PNGs of every group and open overlay, both modes, into /tmp/rune-ui/
RUNE_UI_ITEMS="button switch" apps/gallery/scripts/snap.sh   # single items as well
```

The gallery is itself a registry item: the `gallery` block. Any app can add it and mount the same page:

```sh
rune-ui add gallery
```

```rust
struct App { gallery: ui::gallery::GalleryState, /* ... */ }

// where the page goes (Neko: its Components route, `neko-rune --open components`, ⌘K → Developer)
// pass your own theme file's JSON in place of rune_ui_themes::DEFAULT
let mut layer = Vec::new();
let page = ui::gallery::page(rune_ui_themes::DEFAULT, cx.system_dark(), &s.gallery, |s: &mut App| &mut s.gallery, &ui::stories::all(), &mut layer);
ui::base::overlay_root(page, layer) // the page fills its region and scrolls itself, beside the theme editor
```

The page has a **theme editor** on its right (about 300 points): light or dark, base neutral (theme, grey, stone, zinc, slate), accent
(swatches or a hex), radius, corner style (continuous or round), font (Inter, system, serif, mono and every installed font item, each
shown in its own face), type scale, density, shadow
strength, and material texture (flat to tactile), with Reset, Copy theme and Export. Every control writes a key of the theme file the
page is read from (`gallery::edit_theme`), so what you see is the tokens at work, and Export writes a theme file valid against the schema.

Stories register themselves. Each item's `stories.rs` is copied to `src/ui/stories/<name>.rs` once the gallery block is installed, and once everything the story shows is installed too. `rune-ui add` keeps `src/ui/stories/mod.rs` listing them. So a component you add appears on the page, and one you have not added does not. For Rune's own hover tooltips to match, also add `tooltip` and call `cx.help_look(ui::tooltip::look(&theme))` in your view.

## Components (wave 1)

| Group | Items |
| --- | --- |
| Actions | Button, ButtonGroup, Toggle, ToggleGroup |
| Form | Input, Textarea, Label, Field, Checkbox, RadioGroup, Switch, Slider, Select, Combobox |
| Menus and overlays | DropdownMenu, ContextMenu, Popover, Tooltip, Dialog, AlertDialog, Sheet, Command |
| Structure | Tabs, Accordion, Collapsible, Card, Separator, ScrollArea, Table, Sidebar, Breadcrumb |
| Display | Badge, Avatar, Skeleton, Spinner, Progress, Kbd, Alert, Empty, Toast |
| Blocks | Gallery |
| Fonts | Inter, Geist, Manrope, DM Sans, Figtree, Plus Jakarta Sans, IBM Plex Sans, Geist Mono, JetBrains Mono |

## Components (wave 2)

| Group | Items |
| --- | --- |
| Form | Calendar, DatePicker, InputOTP, Form, InputGroup |
| Menus and overlays | Menubar, NavigationMenu, HoverCard, Drawer |
| Structure | DataTable, Pagination, Resizable, Carousel, AspectRatio, Item |
| Display | Chart (line, bar, area, donut), Typography |

Wave 2 added `recipes::series` (chart colours from the theme's roles), Label and Field's optional mark, `sheet::mount` (Sheet and
Drawer share it) and `GalleryState::slot` (a component's own state type in the gallery). Their behaviour is tested in the live path
(`apps/gallery/tests/wave2.rs`: real presses, pointer moves, drags and keys).

## Fonts

A theme's `base.font` is a name: `inter`, `system` (SF), `serif` (New York, then Georgia) and `mono` (SF Mono) are built in, and a `font`
registry item adds one. Each item copies the font file, its SIL Open Font License and a `Font` constant into `src/ui/<item>/`, and
`rune-ui add` lists it in `ui::FONTS`. Hand them to Rune when the app starts:

```rust
let spec = AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), ..AppSpec::new("My app", (1200.0, 800.0), init, view) };
```

Rune registers fonts only from bytes given at start-up (`AppSpec::fonts`), and a non-empty list replaces its built-in Inter, so every font
item brings `font_inter` (the same Inter 4.1 files Rune embeds). A theme naming a font the app has not installed fails to load and lists the
names it could use. Variable fonts work: Rune sets `wght` to the weight asked for and `opsz` to the point size.

| Item | Name | Licence | Size | Size adjust |
| --- | --- | --- | --- | --- |
| `font_inter` | `inter` | OFL 1.1 | 1.2 MB (400, 500, 600) | 1.00 |
| `font_geist` | `geist` | OFL 1.1 | 165 KB, variable | 1.03 |
| `font_manrope` | `manrope` | OFL 1.1 | 161 KB, variable | 1.00 |
| `font_dm_sans` | `dm-sans` | OFL 1.1 | 235 KB, variable | 1.04 |
| `font_figtree` | `figtree` | OFL 1.1 | 61 KB, variable | 1.08 |
| `font_plus_jakarta_sans` | `plus-jakarta-sans` | OFL 1.1 | 172 KB, variable | 1.02 |
| `font_ibm_plex_sans` | `ibm-plex-sans` | OFL 1.1 | 525 KB, variable | 1.05 |
| `font_geist_mono` | `geist-mono` | OFL 1.1 | 168 KB, variable | 1.03 |
| `font_jetbrains_mono` | `jetbrains-mono` | OFL 1.1 | 183 KB, variable | 1.00 |

Each item's `item.toml` records the source URL and font version. Size adjust matches each face's x-height to Inter's, which the type ramp
was tuned in (sizes land on half points). Font smoothing is Rune's window setting (`App::font_smoothing`, `On` on macOS by default; `Off`
draws like `-webkit-font-smoothing: antialiased`), not a theme token. The gallery's Typography tab sets headings, reading text, small
sizes, proportional and tabular figures, the three weights, a button and an input in the theme's font, then one line in every font.

## Density

The theme's `density` knob has two named settings: **Comfortable** (1.0, the default: 36 point controls, 16 point side padding, 14 point
words, 36 point rows, 20 points between fields) and **Compact** (0.8: 30 point controls, 12 point padding, 13 point words, 30 point
rows, 12 between fields). Control heights, paddings, gaps, row heights, insets and the type ramp all follow it; the gallery page switches
between them. `docs/recipes.md` has the full table.

## Themes

A theme is one JSON file with a `light` mode and a `dark` mode, checked against `crates/rune-ui-themes/theme.schema.json`. Unknown keys are errors. To make your own, copy `themes/default.json`, change values, and run `cargo test -p rune-ui-themes`. The tests check contrast for every role, light/dark parity, and agreement with the schema. While developing, `rune_ui_themes::load(path, dark)` reloads a file from disk.

## What Rune cannot draw yet

- **Grain:** there is no grain primitive at this rev. Recipes carry `Layer::Grain`, the painter records it as skipped, and the gallery says so. `rune-fx` (same rev) is a separate wgpu effects renderer and not an `El` layer, so adopting it means writing a compositor.
- **Hover lift:** a chip cannot lift 1 point on hover. There is no hover offset; it would take app state per chip.
- **Saturated glass:** backdrop blur has no `saturate()`, and Rune allows only a few blurs per frame.
- **Animated focus ring:** the focus ring appears at once; it cannot fade or scale in.
- **Spring timing:** press is a spring, not a fixed 100 ms ease.
- **Sliding indicators:** the chosen segment or tab lands at once instead of sliding. Widths are not known when the view is built.
- **Moving between open menus by pointer:** a Menubar or NavigationMenu menu sits over the shared popup's scrim, which takes the next
  press, so pointing at another title does not switch menus; a press closes the open one first (the keyboard moves between them).
- **Paste outside a text field:** Rune hands a paste only to a text field or the view's root (`on_paste` is honoured there), so InputOTP
  does not take a pasted code at this rev.
- **Wrapping and textures:** no `text-wrap: balance`. Metal textures stop at 16,384 pixels, so headless pictures are at most 8,000 points tall at 2x.

## Develop

```sh
export CARGO_TARGET_DIR=$HOME/Library/Caches/neko/build-rune-ui
cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets
RUNE_UI_BLESS=1 cargo test -p gallery --test snapshot   # after an intended layout change; review the snapshot diffs
cargo run -p rune-ui-cli -- registry hash               # after editing a registry file
```
