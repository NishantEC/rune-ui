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
let file = rune_ui_style::ThemeFile::parse(rune_ui_themes::DEFAULT).unwrap(); // or your own theme file
ui::gallery::page(&file, cx.system_dark(), &s.gallery, |s: &mut App| &mut s.gallery, &ui::stories::all())
```

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

Wave 2 will add Calendar, DatePicker, Carousel, Chart, Menubar, NavigationMenu, Pagination, Resizable, HoverCard, InputOTP, DataTable, Drawer and Form.

## Themes

A theme is one JSON file with a `light` mode and a `dark` mode, checked against `crates/rune-ui-themes/theme.schema.json`. Unknown keys are errors. To make your own, copy `themes/default.json`, change values, and run `cargo test -p rune-ui-themes`. The tests check contrast for every role, light/dark parity, and agreement with the schema. While developing, `rune_ui_themes::load(path, dark)` reloads a file from disk.

## What Rune cannot draw yet

- **Grain:** there is no grain primitive at this rev. Recipes carry `Layer::Grain`, the painter records it as skipped, and the gallery says so. `rune-fx` (same rev) is a separate wgpu effects renderer and not an `El` layer, so adopting it means writing a compositor.
- **Hover lift:** a chip cannot lift 1 point on hover. There is no hover offset; it would take app state per chip.
- **Saturated glass:** backdrop blur has no `saturate()`, and Rune allows only a few blurs per frame.
- **Animated focus ring:** the focus ring appears at once; it cannot fade or scale in.
- **Spring timing:** press is a spring, not a fixed 100 ms ease.
- **Sliding indicators:** the chosen segment or tab lands at once instead of sliding. Widths are not known when the view is built.
- **Wrapping and textures:** no `text-wrap: balance`. Metal textures stop at 16,384 pixels, so headless pictures are at most 8,000 points tall at 2x.

## Develop

```sh
export CARGO_TARGET_DIR=$HOME/Library/Caches/neko/build-rune-ui
cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets
RUNE_UI_BLESS=1 cargo test -p gallery --test snapshot   # after an intended layout change; review the snapshot diffs
cargo run -p rune-ui-cli -- registry hash               # after editing a registry file
```
