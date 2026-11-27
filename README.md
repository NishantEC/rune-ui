# rune-ui

rune-ui does for Rune what Tailwind and shadcn/ui do for React. You get a style system, components that you copy into your app and then own, and themes stored as data. Rune is a GPU-rendered Rust UI framework.

MIT licensed. Rune is pinned to `53f6d36a` (a `dev` commit of 2026-11-25; see `RUNE_REV`). Rune has no releases, so every rune-ui
release names the one Rune rev it was built and tested against, and an app uses that same rev.

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
| `rune-ui update [<item>...]` | Merges the registry's changes into your edits in three ways, using the base copy it kept in `.rune-ui/base/`. Conflicts get markers and the command exits 1. Binary files (fonts) are never merged: see below. |
| `rune-ui registry hash` | For maintainers: refreshes the `sha256` values in every `item.toml`. |

The registry is embedded in the binary. `--registry <dir>` reads it from a folder instead.

**Binary files and `update`.** A file that is not valid UTF-8, or that its item marks `binary = true` (every font item marks its
`.ttf` files), is never merged as text. If your copy is still the registry's old bytes (or what the lock recorded), `update` replaces
it with the new bytes exactly; if you changed it, `update` names it, leaves it untouched, keeps that item at its old version and exits
1. The lock only records the registry's own bytes for an untouched file, and a file left with conflict markers keeps its old hash, so
neither can pass `diff --check`. (Before 0.8.0, `update` merged fonts as text and corrupted them; an app that ran it has its fonts
put back by the next `update`, because their lock hash then matches the corrupted copy.)

### Pinning a release

Release tags are annotated (`git tag -a vX.Y.Z`). Pin `rune-ui-style` and `rune-ui-themes` to one with `tag = "vX.Y.Z"`, which is what
`rune-ui init --git <url> --tag vX.Y.Z` writes:

```toml
rune-ui-style = { git = "https://github.com/NishantEC/rune-ui", tag = "vX.Y.Z" }
```

### Moving an app to a new Rune

The app and rune-ui must build against the same Rune rev, with the same patch table. When a rune-ui release moves the pin (0.7.0
moved it from `bc05f817` to `53f6d36a`):

1. In the app's `Cargo.toml`, change the `rev =` on `rune` and `rune-kit` to the one in rune-ui's `RUNE_REV`.
2. Replace the app's `[patch.crates-io]` table with rune-ui's (its root `Cargo.toml`), every line at the same rev. At `53f6d36a` it has
   eight crates: `winit`, `sctk-adwaita`, `parley_engine`, `wgpu-hal`, `naga`, `accesskit_windows`, `accesskit_macos` and
   `accesskit_atspi_common`. `rune-ui init` adds missing lines but never rewrites a line that is there, so edit the existing ones.
3. Move `rune-ui-style` and `rune-ui-themes` to the new tag (`tag = "v0.8.0"`; 0.8.0 keeps the `53f6d36a` pin).
4. Run `cargo build`. If Cargo warns that a patch "was not used in the crate graph" (or `Cargo.lock` lists it under
   `[[patch.unused]]`), the lock picked a newer crates.io release than Rune's vendored copy; lock it back:
   `cargo update -p naga@30.0.2 --precise 30.0.1`, and the same for `wgpu`, `wgpu-core`, `wgpu-types`, `wgpu-naga-bridge`,
   `naga-types`, `wgpu-core-deps-apple` and `wgpu-core-deps-windows-linux-android` (30.0.1 is what Rune's own lock tests).
5. Run `rune-ui update` to merge the registry's changes into your copies, then `rune-ui diff --check`, `cargo test` and a look at
   the app.

What changed for apps at `53f6d36a`, with no code change on rune-ui's side: inline code in Markdown (Response, the chat block) is now
drawn in the system monospaced face (Rune's mono slot now defaults to `ui-monospace`; before it fell back to the body font), the
streaming caret glyph comes from a different fallback font and sits shorter, and the soft shadow along Sheet and Drawer panel edges
differs by a few levels. Everything else in the gallery draws pixel for pixel as it did.

## The gallery

```sh
cargo run -p gallery                      # a window: every component in every state, light and dark, two densities
apps/gallery/scripts/snap.sh              # headless PNGs of every group and open overlay, both modes, into /tmp/rune-ui/ (taken 1 s in)
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
page is read from (`gallery::edit_theme`), so what you see is the tokens at work. Every control starts from, and Reset returns to, the
theme file's own values. Export hands a theme file valid against the schema to `GalleryState::on_export`, a callback the app supplies (the
page never writes files; with no callback there is no Export button, and Copy theme stays).

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

The agent set (0.6.0) has its own section below.

## Components (wave 3, 0.8.0)

| Group | Items |
| --- | --- |
| Actions | ActionBar (bulk actions on a selection), Toolbar (grouped controls, overflow into More) |
| Form | SegmentedControl (a sliding thumb on an interruptible spring) |
| Structure | Tree, FileTree |
| Agent | StackTrace, TestResults, Commit; PromptInput gains `@` mentions and `/` commands |

Keys, as the interaction tests in `apps/gallery/tests/wave3.rs` press them:

| Item | Keys |
| --- | --- |
| ActionBar | One Tab stop (a toolbar); Left and Right, Home and End inside; Escape clears the selection and focus goes to `return_focus`. Shown only while the count is above zero, rising from the bottom in the overlay layer; it works with DataTable's `selected`. |
| Toolbar | One Tab stop; Left and Right rove, Home and End; Space or Return presses. It measures itself (`ToolbarState`, kept by the app) and moves what does not fit into a More menu. |
| SegmentedControl | One Tab stop on the chosen segment; Left and Right choose (wrapping), Home and End. The thumb retargets mid-slide; a cut under Reduce Motion. |
| Tree, FileTree | One Tab stop on the tree's own row; Up and Down (stopping at the ends), Right opens or goes to the first child, Left closes or goes to the parent, Home and End, a letter jumps, Return opens a leaf or folds a branch. One choice follows the keys; with `multi_select` Space chooses and Shift with Up or Down extends. Lazy branches call `on_expand` and show a loading row. FileTree builds the nodes from paths: folders first, natural order, an icon per kind. |
| StackTrace | Frames are buttons (Return or Space): `on_frame` gets path, line and column; runs of library frames fold into one line; Copy puts the trace on the clipboard. |
| TestResults | Suites are fold lines (Return or Space), failed suites first and open; a failure shows its message and expected/received. |
| Commit | The line folds (Return or Space) to its files (each hands its path to `on_file`) and the diff; Copy hash. |
| PromptInput | With `.menu(..)` and `.mentions(..)` or `.commands(..)`, built with `build_in(t, layer)`: `@` after a space or `/` at the start opens a list over the composer; Up and Down, Return or Tab picks, Escape closes the list only. A mention is one token (a Rune pill) Backspace deletes whole; a command runs and inserts nothing; Return with nothing to pick does nothing. Without sources nothing changes. |

InputOTP (0.8.0) is now one hidden text field behind the slots, so it takes a pasted code: `InputOtp::new(key, name, &edit, access)`
takes the app's `TextEdit` (it used to take a `&str`); `on_change` and `on_complete` are as before.

Wave 2 added `recipes::series` (chart colours from the theme's roles), Label and Field's optional mark, `sheet::mount` (Sheet and
Drawer share it) and `GalleryState::slot` (a component's own state type in the gallery). Their behaviour is tested in the live path
(`apps/gallery/tests/wave2.rs`: real presses, pointer moves, drags and keys).

## Components (0.11.0)

| Group | Items |
| --- | --- |
| Structure | Split (the regions of a window with drag handles: a sidebar, the centre, a pane) |
| Also new in existing items | Tabs (pane variant, close, overflow menu), Toast (host and stack), Command (palette), SidebarItem (thread rows), Kbd (modifier glyphs), Textarea (mono), Dialog (scrolling body), icons |

## Components (0.10.0)

| Group | Items |
| --- | --- |
| Display | Portrait (a tile with a progress ring and a status dot) |
| Structure | Reorder (rows put in order by dragging or with Alt and the arrows) |
| Agent | Receipt (what a run leaves behind) |

## 0.9.0: what the first app needed

Everything here came from an agent app built on rune-ui that had to work round these components; each is additive or a visual fix,
except the breaks listed under "Upgrade notes" below. Tested in the live path in `apps/gallery/tests/wave4.rs`.

| Item | New |
| --- | --- |
| Message | `.reveal_actions(reveal, access)`: the action bar shows only while the message is hovered or holds focus. Hidden, it takes no height and leaves no gap; its buttons stay in the Tab order, and the bar shows when one has focus. The app keeps one `message::Reveal` per message. Rhythm: `TURN_GAP` (20 points) between turns, `BLOCK_GAP` (4) between the blocks of a turn and above the bar (was 24 and 8). |
| Context | `.face(Face::Ring)` (the default: a 20 point ring filling clockwise, the warning ink from 80%, the danger ink from 95%) or `Face::Bar`; `.percent(bool)`; `.note(words)`. The popover shows used and window, in, out, cached, the cost only when `Usage::cost` is `Some`, then the note. Return and Space open it, as a press does. |
| Tool | `.label(words)` (the whole line, cut in the middle when long; the whole of it is the line's name and is in the fold), `.note(words)` ("exit 1"), `.text(words)` (free words at the top of the fold). No time under one second. `Status::Denied`: a neutral shield and "Denied", not red. |
| Reasoning | "Thought" under one second; finished with no words, it draws nothing (no height, no gap). |
| FoldLine (`agent`) | `.indent(false)` sets the open body on the line's own left edge; `.truncate_middle(true)`. ChainOfThought passes `.indent(..)` through. `agent::took(secs)` is a duration only from one second. |
| Response | `.selectable(bool)` (unset: a finished answer is selectable, one still streaming is not); `.muted(bool)` draws the words in the second ink. |
| Confirmation | `.why(words)`: a line under the request. `.destructive(true)`: no Always allow, the confirm button in the danger material, Return answers nothing (a press or Space on a focused answer does), Escape denies. |
| Task | Every fact is optional (`Run::new(title, status).changes(..).checks(..).cost(..).tokens(..).secs(..).branch(..).note(..)`); tokens stand in for a missing cost; no checks, no "0 passed". |
| Sidebar | `SidebarItem::lead(el)`: a status mark in the icon's place, on the icon's box. `.list_row(position, size)` and `.description(words)`: an `option` with its place and selected state, Return opens it. `Sidebar::list(label, name, rows)` makes a group one `listbox` and numbers its rows. |
| RadioGroup | Stacked, an option's description wraps to the group's width (at least 240 points), not at 320. |
| Hit areas | The 40 point wrapper (`hit(<key>)`) has role `none`: it no longer reads as an unnamed button round the named control. `apps/gallery/tests/unnamed.rs` fails on any unnamed button in any story. |
| Disabled ink | Disabled Default (ink) and Destructive (danger) buttons keep their shape: the body lets the surface through, the rim, lit edge and contact shadow stay, the words dim (`recipes::DISABLED_WORDS`), and the control keeps its opacity. Other variants still fade. PromptInput's empty Send stays the raised material at rest with a faint arrow: a dimmed ink disc is a different look, so it was not folded into this. |

What Rune cannot do here at this rev:

- **Hover that changes layout without app state:** Rune's hover groups (`hover_group`, `group_opacity`) change paint only, so a bar
  that takes no room until hovered needs the app to keep a flag (`message::Reveal`), set from `on_hover_change` and
  `on_focus_within_change`.

### Upgrade notes (0.8 to 0.9)

| Break | Fix |
| --- | --- |
| `context::Usage` has `context` (tokens in the context now) and `cost: Option<f64>`; `Usage::used()` is gone | `Usage { input, output, cache, limit, cost }` → `Usage { context: input + output + cache, input, output, cache, limit, cost: Some(cost) }` |
| Context's default face is the ring | `.face(Face::Bar)` keeps the bar |
| `task::Run`'s facts are options (`changes`, `checks`, `cost`, `tokens`, `secs`, `branch`, `note`) | `Run { title, status, files, added, removed, passed, failed, cost, secs }` → `Run::new(title, status).changes(files, added, removed).checks(passed, failed).cost(cost).secs(secs)` |
| `agent::Status` has `Denied`; `Status::ALL` is `[Status; 7]` | add an arm for `Status::Denied` where you match on `Status` without a wildcard |
| `chat::ChatState` has `reveal` | add `reveal: Default::default()` to a struct literal (or use `ChatState::default()`) |
| A conversation's gaps are 20 and 4 points (were 24 and 8) | none needed; to keep the old rhythm, change `TURN_GAP` and `BLOCK_GAP` in your copies |

## 0.10.0: closing the gaps

What an agent app found missing, built as general parts; each is additive unless listed under "Upgrade notes (0.9 to 0.10)". Tested in
the live path in `apps/gallery/tests/wave5.rs`, pictured light and dark in the gallery.

| Item | New |
| --- | --- |
| Portrait (new) | `Portrait::new(key, name, size)` (20 to 112 points): `.image(&Pixels)`, `.face(El)` or the initials of `name`, on `.ground(colour)`. `.ringed(true)` reserves the ring's room in every state (`portrait::ringed(size)` is the outer size); `.state(Activity::Working)` turns a third of a circle round it (still under Reduce Motion), `NeedsYou`, `Failed`, `Done` and `Info` put a dot of the theme's meaning on the corner, ringed in `.under(surface)`. Ring and dot are made 4.5:1 and 3:1 against `under`. An `img` named `name` and described by the state in words (`.state_words(..)`); `.decorative(true)` hides it. |
| Theme file | Optional `color.identity` and `role.identity` (`bg`, `fg`, `solid`, `on-solid`), `number.scrim`, `base.keys-font`, and `type.<step>.{size, line-height, weight, tracking}`. `Theme::identity()` (falls back to `info`), `scrim()` / `scrim_soft()` (black at the theme's strength), `keycap` (the keys' text style), `shade(a)` / `glint(a)`. The default theme sets identity (a teal) and the scrim as the example. `docs/recipes.md` has the details. |
| Scrim | Dialog, AlertDialog, Sheet, Drawer and Command dim with black (`Theme::scrim()`), not the page ink at an alpha, so a dark page dims instead of fogging. |
| Message | `.header(El)`: a row that leads the turn (portrait, name, state), one `BLOCK_GAP` above the first block, keyed `<key>-header`. `.bubble(Bubble::Well)`: the person's words in the well material (16 across, 10 down, at most four fifths of the column); `Bubble::Quiet` is as before. |
| PromptInput | `.lead(El)`: beside the text, top-aligned, its own Tab stop before the text. `.chip(El)` (repeat): a wrapping row under the text, Tab stops after the text and before the toolbar. Unused, nothing changes. |
| SidebarItem | `.image(El)` and `.subtitle(words)`: a two-line row (52 points at the default density) with a title over a quiet second line. `.selected_style(Selected::Raised)`: the current row as a raised pebble, with its focus ring. `.trail(El)`: a trailing element after the count. `.button(true)`: role `button`. A count is read out ("Count 3", or `.count_label("3 unread")`) with the second line, after the name. |
| Tool, Confirmation, Reasoning | `Tool::marks(Marks::Neutral)` and `Confirmation::answered_marks(Marks::Neutral)`: a tick in the third ink for done, an 8 by 1.5 dash for denied, no green and no red; only a failure keeps colour. `Reasoning::icon(None)`: a hollow 7 point dot in place of the sparkle. `agent::mark_with`, `agent::hollow_dot`. |
| Receipt (new) | `Receipt::new(key, sentence)`: `.lead(El)`, `.totals(added, removed)`, `.files(&[ReceiptFile::new(path, added, removed)])`, `.max_files(5)` ("and N more"), `.note(words)`, `.branch(name)`, `.action(El)`, `.on_file(f)` (rows become buttons), `.settle(bool)`. A raised card, 6 points of padding, a 40 point header, files in a well in 32 point rows (paths cut in the middle, a zero side of a figure hidden), a 40 point foot. `receipt::figures` and `figures_words` for your own rows. |
| Diff | `.stats(added, removed)` for a folded or cut file, `.hide_zero(true)`, `.selectable(true)` (the words of the lines, one selection across them, signs left out), `.foot(El)` (a row under the lines). |
| Menus | `Row::image(El)` (also on `DropdownMenuItem` and `ContextMenuItem`): a picture or element leads a row in place of its icon. Shortcuts are drawn in `Theme::keycap`, as Kbd is. Every row says its place (`position_in_set`), a listbox's current option is selected, and the open surface is named after its list, so Select and menus read as "2 of 5". |
| Context | `.placement(Placement)` (above and to the right by default) and `.limit_assumed(true)` ("Window (assumed)" and a line saying so). |
| Task | `.secondary(words, f)` (repeat): raised actions beside the primary one. `agent::Status::Stopped` (a square, neutral, "Stopped"). |
| Confirmation (list) | `.in_list(position, size)` and `.escape_denies(false)`: one request among several, Escape left to the list. |
| Textarea | `.limit(n)`: a "12 / 500" counter at the foot (warning ink from 90%, danger over the limit, the well invalid), characters not bytes. `.hard_limit(true)` takes the excess out as it is typed or pasted. |
| Reorder (new) | `Reorder::new(key, name, pitch).rows(vec![..]).on_move(|s, from, to| ..)`: Alt with Up, Down, Home and End, or drag a row; Escape cancels a drag. `reorder::move_item`, `drop_index`, `key_target`. You keep the list. |

What Rune cannot do here at this rev:

- **A live drag preview in Reorder:** a dragged row does not travel under the pointer; it moves when released. Rune's `on_drag` reports
  the travel but a row has no per-frame offset without app state.
- **Dropping Rune's Ogg audio from an app without naming its features:** see "Licences" below.

### Upgrade notes (0.9 to 0.10)

Move `rune-ui-style` and `rune-ui-themes` to `v0.10.0` first, then `rune-ui update`. The copies of `popup`, `kbd`, `dialog`, `sheet`,
`command` and `portrait` need the new style crate (`Theme::keycap`, `Theme::scrim`).

| Break | Fix |
| --- | --- |
| `agent::Status` has `Stopped`; `Status::ALL` is `[Status; 8]` | add an arm for `Status::Stopped` where you match on `Status` without a wildcard |
| `popup::Row` has a public field `image` | build rows with `Row::new(..)`; a struct literal gets `image: None` |
| Style structs gained fields: `Palette::identity`, `Theme::scrim_strength` and `keycap`, `Base::keys_font`, `ThemeFile::type_steps` | only code that builds these by struct literal (not from a theme file) breaks: add the fields, or build from JSON |
| The modal scrim is black at `Theme::scrim()` (0.18 light, 0.55 dark in the default theme; was the page ink at 0.18 and 0.5) | none needed; set `number.scrim` in your theme file to tune it |
| A sidebar row with a count now says "Count N" to assistive technology; popup rows say their place | none needed; `.count_label(words)` words the count |
| The workspace takes Rune with `default-features = false` (see "Licences") | none for apps: `rune-ui init` still writes Rune with its default features |

## 0.11.0: the window around a conversation

An agent app is three columns (a sidebar of threads, the conversation, a tabbed pane), a palette on Cmd+K, toasts that say "Undo", and
dense two-line rows. These came from one app's list of what it could not build from 0.10.0; they are general parts, additive unless
listed under "Upgrade notes (0.10 to 0.11)". Tested in the live path in `apps/gallery/tests/wave6.rs`, pictured light and dark in the
gallery. Rune stays at `53f6d36a`.

| Item | New |
| --- | --- |
| Split (new) | `Split::new(key, name)` with `.left(Region)`, `.center(El)`, `.right(Region)`. A `Region` has `.width(points)` (the app keeps and saves it), `.min`, `.max`, `.max_fraction` (a right pane never past half the window), `.default` (double-click on its handle goes back to it), `.collapsed(bool)`, `.collapsible(true)` (a drag past the least by 48 points collapses it, a drag from the edge opens it), `.name`. The centre keeps `.center_min(320)`. `.on_resize(|s, Edge, width| ..)`, `.on_collapse(|s, Edge, hidden| ..)`. A handle is a `separator` named "Resize <name>" whose value is the width: Left and Right move it by `.step` (16; Shift four times), Home and End go to the least and most, Return collapses or opens. `split::clamped` is the bounds rule. Handles take one point of layout and a nine point grab area; they paint above both neighbours. A sidebar and a pane work at once. |
| Tabs | `.activation(Activation::Automatic \| Manual)`: Left, Right (wrapping), Home, End move; automatic chooses as it moves, manual moves focus and Return or Space chooses. Roving: the chosen tab is the strip's one Tab stop and is `selected`; the `tabpanel` is labelled by it and the tab `controls` it. `.on_close(f)`: a close button per tab ("Close <tab>", 18 points, drawn for the chosen tab and on hover or focus, out of the Tab order) and Cmd+W on a focused tab; `.permanent()` keeps the last tab you added. `.overflow(&TabsState, access)` + `.build_in(t, &mut layer)`: the strip and its tabs measure themselves; tabs that do not fit go into a "More tabs" menu (`tabs::fitting` is the rule), the chosen tab always stays. `TabsVariant::Pane`: compact (one control height, small words, semibold chosen tab, a 2 point line, no hairline). No motion: nothing to hold still under Reduce Motion. |
| Toast | `ToastSpec::new(title).description(..).tone(..).action(words, tag).timeout(secs).sticky()` (5 seconds, 8 with an action). `ToastHost` (`new`, `push`, `show(spec, tx)`, `dismiss`, `dismiss_latest`, `clear`, `items`, `paused`, `tick`, `next_due`, `poll(clock, tx)`, `set_hover`, `set_focus`, `focus_target(stack_key)`): you keep one in your state and call `poll` from the app's poll hook (`AppSpec::poll`). Timers stop while the pointer is over the stack or focus is inside it and carry on from where they were. `Toasts::new(key, &host, access).position(Position::BottomCenter \| BottomTrailing).limit(4).on_action(\|s, tag\| ..).mount(t, &mut layer)`: each toast is a polite `status` live region; the action dismisses it and tells you its tag; Escape on a focused toast dismisses it; toasts come in from the bottom edge (not under Reduce Motion). The key that reaches the stack is yours: send focus to `focus_target`. `Toaster::position` for stacks you build yourself. |
| Command | `CommandItem::subtitle`, `lead(El)`, `hint(words)`, `keywords(words)`; `Command::matching(Match::Fuzzy \| Words)`, `recents(&keys)` and `recent_label`, `loading(bool)`, `on_alt(f)` (Command+Return), `hint(keys, words)` (a footer), `limit(n)`, `top(points)`. `command::search(items, query, mode)` returns `Hit`s (item, matched title and subtitle positions, score), `command::fuzzy(query, text)` scores one text. Sections keep the order they first appear in your items and rows are ranked inside them; the letters that matched are semibold; rows are `option`s in `group`s with a heading, each saying its place; a row with a second line is `metrics.row` plus 16 high. While `loading`, the field spins and the rows stay; with none it says "Searching…". |
| SidebarItem | `.subtitle_el(El)`, `.unread(true)` (semibold in the first ink; "Unread" is read after the name), `.trail_swap(width, rest, hover)` (hover groups: the age shows, a menu button takes its place under the pointer or keyboard focus; pinned with `.state(Hover)` it shows the button), `.indent(levels)` (16 points a level). Row heights follow the theme's density (30 and 46 compact, 36 and 52 comfortable). |
| Kbd | ⌘ ⌥ ⇧ ⌃ ↩ are icons (`icons::KEY_*`), one cap each, spaced or run together; words stay words in the keycap face; the name is spoken ("Shift Command P"). `kbd::modifier`, `kbd::spoken`. |
| Textarea, Dialog, icons | `Textarea::mono(true)`. `Dialog::scroll_body(true)` (+ `.max_height(points)`): the panel shrinks to the window, the header and footer stay, the body scrolls, a hairline marks the footer. Icons: `PLUG`, `CHEVRONS_RIGHT`, `PANEL_LEFT`, `PANEL_RIGHT`, `KEY_COMMAND`, `KEY_OPTION`, `KEY_SHIFT`, `KEY_RETURN`, `KEY_CONTROL`. |

What Rune cannot do here at this rev:

- **Drag a tab to reorder it:** tabs keep the app's order; `Reorder` is for columns of rows of one pitch.
- **Toasts that wake themselves:** a toast needs the app's poll hook (`ToastHost::poll`), because Rune delivers the window's clock only to
  `AppSpec::poll` and to handlers.

### Upgrade notes (0.10 to 0.11)

Move `rune-ui-style` and `rune-ui-themes` to `v0.11.0` first, then `rune-ui update`. The style crate itself is unchanged; every item
now requires `>=<its minimum>, <0.12.0`.

| Break | Fix |
| --- | --- |
| `command::CommandItem` has public fields `subtitle`, `lead`, `hint`, `keywords` | build with `CommandItem::new(..)`; a struct literal gets `subtitle: None, lead: None, hint: None, keywords: None` |
| `CommandItem` is not `Send` or `Sync` | build items in the view (or a `thread_local!`) rather than a `static OnceLock` |
| Command matches fuzzily and ranks | `.matching(Match::Words)` for the old rule |
| Tabs: one Tab stop; `selected` not `checked`; needs `popup` | none; adjust tests that looked for `checked`; `rune-ui update` adds `popup` |
| Kbd's name speaks glyphs ("Command K") | none; adjust tests that read the name |

## Licences

rune-ui is MIT. Through Rune, an app also gets permissive crates (MIT, Apache-2.0, Zlib, BSD, ISC, Unicode, Unlicense) and, with Rune's
default features, two MPL-2.0 crates, `symphonia` and `opuscule` (Rune's Ogg audio decoders), which arrive through the `ogg-audio`
feature. From 0.10.0 rune-ui's own lock file has neither: the workspace depends on Rune with `default-features = false` and the
gallery names every default feature except `ogg-audio` (`accessibility`, `dbus`, `embedded-fonts`, `hunspell`, `metal-backend`, `svg`);
nothing else is lost (the other defaults only re-enable those). An app drops them the same way:

```toml
rune = { git = "https://github.com/anomalyco/rune", rev = "53f6d36a96439d357a6f2a2560383d7b9e9ec9c2", default-features = false, features = ["accessibility", "dbus", "embedded-fonts", "hunspell", "metal-backend", "svg"] }
```

Cargo unifies features, so an app that keeps Rune's defaults (what `rune-ui init` writes) still builds them. Fonts are OFL 1.1 (each
font item carries its licence).

## Agent set

The components an app needs to show an AI agent at work, built on the base components and the style system. Names follow Vercel's
AI Elements where one exists. Everything is data in and events out: plain structs and callbacks, no network, no async runtime.

| Item | What it is for |
| --- | --- |
| Conversation | The scroll area: pinned to the newest message, lets go when the reader scrolls up, then shows Jump to latest |
| Message | A person's quiet or well bubble at the right, or the agent's words on the page in a 720 point column; a header that leads the turn, avatar, attachments, actions |
| Response | The agent's words as Markdown (rune-kit's `markdown` feature), with a caret while streaming |
| PromptInput | The composer: growing text, attachment chips, a toolbar, Send that becomes Stop; Return, Shift+Return, Escape. With nothing to send, Send is a raised button at rest with a faint arrow (off, but still a button); with words it is ink. `.spellcheck(true)` marks misspelt words |
| ModelSelector | Models grouped by provider in a searchable panel from the overlay layer, with a tier or price per row |
| Attachment | A file or image chip with remove |
| Tool | One tool call as one quiet line that reads like a sentence (its own words, a note, a time from 1 s) that folds open to its input and output |
| Reasoning | "Thinking…" with a shimmer, then a "Thought for 12s" fold ("Thought" under a second; nothing at all when there are no words) |
| ChainOfThought | The "Worked for 2m 14s" fold: steps with marks, words and details |
| CodeBlock | Code with a language label, Copy, line numbers and sideways scroll; colour from Rune's built-in highlighter |
| Diff | A unified diff per file: a folding header with `+n −n` (or the app's figures), added and removed lines on their washes, selectable, with a foot |
| Receipt | What a run leaves behind: totals, the files with their own figures, the branch and actions |
| Terminal | A command, its output, its running state and exit status |
| Task | A run card: status pill, only the facts given (files, checks, cost or tokens, time, branch, a note), a primary action and secondary ones |
| Plan | A checklist: to do, doing, done, skipped |
| Queue | Messages waiting to be sent, moved with Alt and the arrows, removed with Delete |
| Sources, InlineCitation | A "Used 4 sources" fold of links; a citation chip with a hover card |
| Suggestion | A row of suggested prompts |
| Actions | Copy, Retry, Good, Bad under an answer, with tooltips |
| Confirmation | A tool asking permission: Allow once, Always allow, Deny, an optional why line, a destructive form; answered, one quiet line |
| Shimmer, Loader | Words with a sweeping band; three dots. Both still under Reduce Motion |
| StackTrace, TestResults, Commit | An error and its frames; a test run; one commit folding to its files and diff (wave 3) |
| Context | A context meter (a 20 point ring, or a bar) with a popover: used and window, in, out, cached, an optional cost and note |
| Branch | "2 / 3" between alternative answers |
| Checkpoint | A divider with Restore |
| `chat` (block) | All of it composed into one agent screen with placeholder content |

The shared pieces (a step's `Status`, the quiet `FoldLine`, monospaced type, words for durations and money) live in the `agent` base
item, which `rune-ui add` brings along. Response and the chat block need rune-kit's `markdown` feature; their `item.toml` says so
(`features = ["rune-kit/markdown"]`) and `rune-ui add` turns it on in the app's Cargo.toml.

A chat screen, put together:

```rust
let mut layer = Vec::new();
let turns = vec![
    Message::user("m1", "Asha").child(message::words(&t, "Why is the build slow?")).build(&t),
    Message::assistant("m2", "Assistant")
        .child(Reasoning::new("r1", false, 12.0, s.is_open("r1")).text(&s.thoughts).on_toggle(|s: &mut App| s.flip("r1")).build(&t))
        .child(Tool::new("t1", &s.call, s.is_open("t1")).on_toggle(|s: &mut App| s.flip("t1")).build(&t))
        .child(Response::new("a1", &s.answer).streaming(s.streaming).build(&t))
        .build(&t),
];
let chat = Conversation::new("chat", s.at_bottom).children(turns).on_at_bottom(|s: &mut App, at| s.at_bottom = at).build(&t);
let composer = PromptInput::new("composer", &s.draft, |s: &mut App| &mut s.draft).phase(s.phase)
    .on_submit(|s: &mut App| s.send()).on_stop(|s: &mut App| s.stop()).build(&t);
ui::base::overlay_root(col().w_full().h_full().child(col().grow(1.0).child(chat)).child(composer), layer)
```

What Rune cannot do here at this rev:

- **A blinking caret inside Markdown:** the streaming caret is a glyph at the end of the text, so it does not blink; rune-kit's
  word reveal is the motion instead.
- **Return on a button:** on macOS Rune sends Return to the window's default button and presses a focused button only on Space, so
  fold lines and the permission answers take Return themselves.
- **Shimmer on any surface:** the band is the page's colour, so the shimmer reads right on the page and on quiet surfaces, not on a
  saturated one.
- **The whole Agent tab in one picture:** headless pictures stop at 8,000 points, so `snap.sh` pictures the last items on their own
  (`RUNE_UI_ITEMS`).

## Reduce Motion

Builders only receive `&Theme`, so the theme carries the person's choice: `Theme::reduced_motion`, not in the theme file, set by the
app every frame from Rune:

```rust
let t = rune_ui_themes::theme(cx.system_dark()).with_reduced_motion(cx.is_reduced_motion());
// the gallery page: ui::gallery::page(json, ui::gallery::Appearance::of(cx.system_dark(), cx.is_reduced_motion()), …)
```

With it on, Shimmer draws its words with no band, Loader's dots and Spinner's arc (and every status mark and busy button) hold still,
streaming words in Response appear without fading in, SegmentedControl's thumb cuts instead of sliding, and overlays (the popup
lists, HoverCard, Drawer, ActionBar, PromptInput's lists) appear in place with no grow or slide; a short cross-fade is all that is
left, as macOS keeps one. State stays visible: busy flags, a fixed arc, the words. Skeleton never moved. `page(json, dark, …)` with a
plain `bool` still works (motion on). The registry test `entry_motion_is_off_under_reduced_motion` fails on any item transition that
ignores the flag.

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
- **Sliding indicators in Tabs and ToggleGroup:** they still land at once. SegmentedControl's thumb does slide (equal-width cells with
  the thumb in the chosen grid cell, so Rune springs its layout); Tabs keep content-sized tabs, whose widths are not known when the
  view is built.
- **Presence motion in tests:** Rune's element list does not report an entering element's presence transform, so the tests check
  overlays' still entry by their size, place and fade, and the registry test checks the source.
- **Moving between open menus by pointer:** a Menubar or NavigationMenu menu is a modal (`El::modal`), and Rune blocks all pointer
  input outside a modal, hover included (there is a `modal_wheel_outside` but nothing for pointer moves). Finding the title under the
  pointer is no longer the problem (`El::data` plus `Tx::element_at_in`); the move never arrives. So pointing at another title does
  not switch menus; a press closes the open one first (the keyboard moves between them).
- **Wrapping and textures:** no `text-wrap: balance`. Metal textures stop at 16,384 pixels, so headless pictures are at most 8,000 points tall at 2x.

## Known Rune issue on macOS: "Application Not Responding" in the Dock

At the pinned Rune rev (and still at upstream's tip on 2026-11-26), an app's Dock menu is often headed "Application Not Responding":
the app's own Dock items and Quit are missing and Force Quit is offered, although the app is fine. Rune makes a symbol image on a
short-lived launch thread before `NSApplication.shared`; AppKit registers a process with the Dock once, on whichever thread first
makes one, so the Dock's link can end up tied to a thread that has exited. It is a race: about four launches in five.

An app fixes it by doing one of these before `rune::run` / `run_windows`, and only when it opens a real window (not for a headless
run):

- Ask for a symbol image on the main thread first (keeps Rune's faster launch). With `objc2`:

  ```rust
  let name = objc2_foundation::NSString::from_str("info.circle");
  let _: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe {
      objc2::msg_send![objc2::class!(NSImage), imageWithSystemSymbolName: &*name, accessibilityDescription: None::<&objc2_foundation::NSString>]
  };
  ```

- Or set `RUNE_LAUNCH_PLAIN=1` in the process before Rune starts (turns off all of Rune's launch helpers). The gallery app does this.

Nothing reports the state to the app: the window server says it is responsive. Only the Dock menu shows it, so after a Rune bump,
launch the real app and read its Dock menu.

## Develop

```sh
export CARGO_TARGET_DIR=$HOME/Library/Caches/neko/build-rune-ui
cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets
RUNE_UI_BLESS=1 cargo test -p gallery --test snapshot   # after an intended layout change; review the snapshot diffs
cargo run -p rune-ui-cli -- registry hash               # after editing a registry file
```
