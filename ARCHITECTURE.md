# rune-ui architecture

rune-ui is to Rune what Tailwind plus shadcn/ui are to React: a style system, components you copy into your app and own, and themes
as data. This file is the rule book. When code and this file disagree, fix one of them in the same change.

## The layers

| # | Layer | Where | Owns |
| --- | --- | --- | --- |
| 0 | Rune | git dependency, pinned (`RUNE_REV`) | views, layout, GPU drawing, text, motion, focus, accessibility tree |
| 1 | Behaviour | Rune's `rune-kit`, same rev | keys, focus, dismissal, placement, typeahead, text editing, listboxes |
| 2 | Style | `crates/rune-ui-style` | tokens, recipes, the resolver, the painter, `to_kit()` |
| 3 | Components | `registry/<item>/` | copy-in source (one file per item), stories, an accessibility check, a layout snapshot |
| 4 | Themes | `crates/rune-ui-themes` | theme files (light and dark modes) and their schema |
| 5 | Apps | yours | your copies in `src/ui/`, your theme |

Arrows only point down. A component reads the style layer and Rune; it never reads another app, never names a colour, and never
reaches into a theme file.

## The behaviour/paint seam

rune-kit is a **styled** kit (it ships its own look), not a headless one. rune-ui uses its behaviour and replaces its look:

1. **Heavy components wrap rune-kit through `Theme::to_kit()`.** Select, Combobox, DropdownMenu, ContextMenu, and anything with a
   listbox, typeahead or nested menus is built by rune-kit and handed a `rune_kit::Kit` made from our tokens: our floating surface,
   our shadows, no border, our type. The component then repaints the parts rune-kit lets the caller replace (a select's trigger).
2. **Overlays use Rune's and rune-kit's primitives, painted by us.** Dialog, AlertDialog, Sheet, Popover, Tooltip, Toast and Command
   draw their own panels from the floating material and get their behaviour from `El::modal` (focus trap, Escape, inert background),
   `rune_kit::scrim` (press outside) and `rune_kit::popover::anchored` (placement beside a trigger, kept in the window).
3. **Text editing is Rune's `text_input`** (caret, selection, IME, undo, accessibility), the same editor rune-kit builds on. Input and
   Textarea paint the well round it.
4. **Simple components are painted directly** from recipes: Button, Toggle, Checkbox, RadioGroup, Switch, Slider, Tabs, Badge, Card,
   Separator, Skeleton, Progress, Kbd, Avatar, Alert, Empty, Breadcrumb, Sidebar rows.
5. **Accessibility lives with behaviour.** Every interactive element sets a role, a name, its state (`checked`, `expanded`, `disabled`,
   `busy`, range values) and is focusable only when live. Icon-only controls require a name. Each item's accessibility check proves it.

If rune-kit later splits into headless behaviour plus a theme, step 1 becomes a direct call and nothing above it changes.

## Tokens: three tiers

1. **Ramps** (reference): raw colours by ramp and step (`neutral.50`, `blue.600`). Never read by components.
2. **Roles** (semantic): what a colour is for. Every surface role has a matching text colour (`page.bg` / `page.fg`, `primary.bg` /
   `primary.fg`), as shadcn pairs `background` with `foreground`. Meanings (`info`, `success`, `warning`, `danger`) also carry a solid
   colour and its text. State layers (hover, pressed, selected) are the page text colour at a token alpha, not separate colours.
3. **Component tokens** (rare): the few values a material needs that no role names (a raised gradient's two ends, the ink's rim).

Three base knobs scale everything else: `spacing` (the grid unit), `radius` (the middle of the radius ladder) and `density`
(multiplies control heights and paddings). A theme file has one `light` and one `dark` mode with the same keys, and is checked
against `crates/rune-ui-themes/theme.schema.json`. **Unknown keys are errors**, never ignored.

## Recipes

A recipe is typed data, never code paths in a component:

- **Layers** (`Layer`): fill, gradient, lit top edge, inner shadow, inner rim, drop shadow, ring, border, wash, scale, opacity,
  backdrop blur, grain. Bottom of the stack first.
- **Materials** (`Material`): a base recipe and a **delta for each state** of the closed set `Rest, Hover, Pressed, FocusVisible,
  Disabled, Selected, Invalid, Loading, Open`.
- **Parts** (`Parts`): a component's named parts, `root` (a material) and `label` and `icon` (a tint per state).
- **Variants** are enums (`ButtonVariant`, `Size`), `#[non_exhaustive]`; a compound variant is a `match` arm in `recipes.rs`.
- **One resolver** (`resolve`) decides what is drawn, always in this order: the base; `selected` on top of it; then the one
  interaction state that wins, **disabled > loading > pressed > hover > rest**; then `invalid`, `open` and the `focus` ring on top.
  A disabled control shows no focus ring, invalid ring or open look.
- **One painter** (`paint`): `plan(recipe)` lists the Rune calls (`Op`), `apply` makes them. Live hover, press and focus are handed
  to Rune (`apply_live`) so they animate at Rune's rate; a gallery pins them instead.
- **Escape hatches**: a component takes `.recipe(|parts| …)` to change its parts before painting (Button does today; others add it
  as they need it), and every component file is yours once copied.

## Components (the registry)

- Names are shadcn/ui's (`Button`, `DropdownMenu`, `AlertDialog`, `ScrollArea`), so people find them.
- One builder per component: `Name::new(key, …)` or a variant constructor, at most about twelve chained choices, `.build(t)` (or
  `.build(t, layer)` for anything that opens over the page). Handlers are generic over the app's state (`on_press(|s: &mut S| …)`).
- Keys are required. Icon-only controls require a name. A disabled or loading control is not focusable and never runs its handler.
- Every pressable thing has a 40 point hit area (`base::hit_area`) without changing layout.
- A component reads only `Theme` (tokens, materials, recipes). The guard test fails on a hex colour or a raw `Color::rgb` in
  `registry/`.

Each item is a folder:

```
registry/button/
  item.toml      schema, name, version, group, requires (style range, Rune rev), dependencies (registry items, crates, cargo
                 features such as "rune-kit/markdown", which `rune-ui add` turns on), files with sha256
  button.rs      the source the CLI copies into your src/ui/
  stories.rs     the states the gallery shows (the gallery is generated from these)
  a11y.rs        the roles, names and states the accessibility test checks
  snapshot.txt   the layout snapshot (keys, roles, rects) the snapshot test compares
```

A `font` item is a folder too: the font file (variable where the family has one), its `OFL.txt`, and `<item>.rs` holding one
`rune_ui_style::Font` constant, all copied to `src/ui/<item>/`; `item.toml`'s `[font]` table records the source URL and version.
`rune-ui add` lists installed fonts in `ui::FONTS`, which the app passes to `rune_ui_style::fonts::install` for `AppSpec::fonts`.

Stories are copied too (to `src/ui/stories/<name>.rs`), but only once the `gallery` block is installed and everything a story shows is
installed; `rune-ui add` keeps `src/ui/stories/mod.rs` listing them, so the gallery page shows exactly the components an app has. The
gallery block (`registry/gallery/`) is the page; `apps/gallery` is a window around it, generated from the registry by its `build.rs`.

Two items writing the same path is an error. Versions are semver; a change to a file changes its sha256, and the registry test fails
until `item.toml` is updated (`rune-ui registry hash`).

## Distribution

- `rune-ui-style` and `rune-ui-themes` are dependencies (git, pinned by tag once published; a path while local).
- Components are copied in by the CLI, which records each file's version and hash in `rune-ui.lock`.
- `rune-ui init` writes Rune's `[patch.crates-io]` table, because Cargo does not inherit a dependency's patches.
- Rune is pinned to one rev everywhere. Bumping it is a release of rune-ui.

## Tests

- **Style**: each material's recipe paints the expected layers; the resolver's precedence; contrast of every role's text on its
  surface; light/dark parity; unknown theme keys fail; the schema and the parser agree.
- **Items**: an accessibility check (roles, names, states) and a layout snapshot per item; a guard that components read tokens only;
  hashes in `item.toml` match the files.
- **Gallery**: every story renders in both modes; pixel checks on a few materials; `apps/gallery/scripts/snap.sh` writes PNGs.
