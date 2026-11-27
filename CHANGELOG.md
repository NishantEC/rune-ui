# Changelog

The README's release sections (`0.9.0: what the first app needed`, `0.10.0: closing the gaps`, `0.11.0: the window around a
conversation`) carry the full tables of options and keys; this file is the short version for people upgrading. rune-ui is pinned to
one Rune rev per release (`RUNE_REV`); 0.10.0 and 0.11.0 keep `53f6d36a`.

## 0.11.0

The parts of a three-column agent window (a sidebar, a conversation, a tabbed pane), a command palette, toasts with Undo and denser
sidebar rows. General and app-neutral; tested in the live path and pictured light and dark.

### New

- **Split** (new item): the regions of a window with drag handles. Widths are points the app keeps (so it can save them), with a
  least and a most, an optional share of the window, a floor for the centre, double-click to reset, arrow keys on a focused handle
  and collapse. `Split::new(key, name).left(Region::new(el).width(w).min(208.0).max(360.0).default(256.0)).center(el).right(..)
  .on_resize(|s, edge, w| ..).on_collapse(|s, edge, hidden| ..).build(t)`
- **Tabs**: Left, Right, Home and End with automatic or manual activation (`.activation(Activation::Manual)`); the chosen tab is the
  one Tab stop and says it is selected; the panel is labelled by it. `.on_close(|s, i| ..)` gives each tab a close button and
  Cmd+W (`.permanent()` exempts the last tab added). `.overflow(&state, access)` with `build_in(t, &mut layer)` folds tabs that do not
  fit into a "More tabs" menu and keeps the chosen one in the strip (`TabsState`). `TabsVariant::Pane` is a compact header for a
  side pane.
- **Toast**: `ToastHost` (plain data the app owns: `show`, `push`, `dismiss`, `poll`) times toasts, stops the timers while the
  pointer is over the stack or focus is inside, and `Toasts::new(key, &host, access).position(Position::BottomCenter)
  .on_action(|s, tag| ..).mount(t, &mut layer)` draws them (one action named by a tag, Escape dismisses, `focus_target` for the
  app's shortcut). `Toaster::position`.
- **Command**: fuzzy matching with the matched letters semibold (`search`, `fuzzy`, `Match`), rows with a leading element, a
  second line, a hint and a shortcut (`CommandItem::lead`, `subtitle`, `hint`, `keywords`), `recents(&keys)`, `loading(true)`,
  Command+Return as an alternate choice (`on_alt`), a footer of key hints (`hint(keys, words)`), `limit`, `top`.
- **SidebarItem** (the two-line list row): `subtitle_el(El)` (a second line of your own parts), `unread(true)`, `trail_swap(width,
  rest, hover)` (an age that gives way to a menu button under the pointer or focus), `indent(levels)`.
- **Small**: icons `PLUG`, `CHEVRONS_RIGHT`, `PANEL_LEFT`, `PANEL_RIGHT` and the modifier keys; `Textarea::mono(true)`;
  `Dialog::scroll_body(true)` and `max_height(points)` (header and footer stay, the body scrolls); `Kbd` draws ⌘ ⌥ ⇧ ⌃ ↩ as icons
  (also run together, "⇧⌘P") and speaks them ("Shift Command P").

### Changed

- Command matches fuzzily and ranks results inside sections that keep their first-seen order. `.matching(Match::Words)` keeps
  the old rule (every word appears in the label).
- Tabs: only the chosen tab is a Tab stop (arrows move between tabs); a tab reports `selected`, not `checked`; Tabs needs `popup`.
- A toast with one line of words sits level with its buttons.
- Kbd's accessible name speaks the glyphs ("Command K", not "⌘ K").

### Breaking

| Break | Migration |
| --- | --- |
| `CommandItem` has new public fields (`subtitle`, `lead`, `hint`, `keywords`) | build items with `CommandItem::new(..)`; in a struct literal add `subtitle: None, lead: None, hint: None, keywords: None` |
| `CommandItem` is no longer `Send` or `Sync` (a lead is an element) | build the items in the view (or a `thread_local!`) instead of a `static OnceLock` |
| Command's default matching is fuzzy and ranked | `.matching(Match::Words)` for the old rule |
| A tab strip has one Tab stop; tabs are `selected`, not `checked` | none for users; update tests that looked for `checked` |
| Items need `rune-ui-style` 0.11 (all ranges are `<0.12.0`) | move `rune-ui-style` and `rune-ui-themes` to `v0.11.0`, then `rune-ui update` |

## 0.10.0

An agent app that was built on rune-ui found parts it had to write itself. They are now here, general and tested light and dark.

### New

- **Portrait** (new item): a rounded tile (picture, initials or your own drawn face) from 20 to 112 points, with an optional progress
  ring that takes the same room in every state, a status dot in a theme meaning on its corner, and its state in words.
  `Portrait::new(key, name, size).state(Activity::Working).ringed(true).under(surface).build(t)`
- **Receipt** (new item): a card for what a run leaves behind: header with totals, files with their own figures (a zero side hidden),
  branch and actions. `Receipt::new(key, sentence).totals(a, r).files(&files).branch(b).action(button).build(t)`
- **Reorder** (new item): rows put in order by dragging or with Alt and the arrows; you keep the list.
  `Reorder::new(key, name, pitch).rows(rows).on_move(|s, from, to| ..).build(t)`
- **Theme file**: an optional identity colour (`color.identity`, `role.identity`), a scrim strength (`number.scrim`), a keys font
  (`base.keys-font`) and exact type steps (`type.<step>`). Themes that leave them out behave as before.
- **Message**: `.header(El)` for a turn's leader; `.bubble(Bubble::Well)` for the person's words in a well.
- **PromptInput**: `.lead(El)` beside the text and `.chip(El)` under it, in Tab order.
- **SidebarItem**: two-line rows (`.image`, `.subtitle`), `.selected_style(Selected::Raised)`, `.trail(El)`, `.button(true)`,
  and a spoken count (`.count_label`).
- **Tool, Confirmation, Reasoning**: neutral step marks (`Marks::Neutral`) and a hollow thinking dot (`Reasoning::icon(None)`).
- **Diff**: `.stats`, `.hide_zero`, `.selectable`, `.foot`.
- **Menus**: `Row::image` leads a row with a picture; shortcuts use the theme's keycap style.
- **Also**: `Context::placement` and `limit_assumed`; `Status::Stopped`; `Task::secondary`; `Confirmation::in_list` and
  `escape_denies`; `Textarea::limit` and `hard_limit`.

### Changed

- Dialog, AlertDialog, Sheet, Drawer and Command dim with black at the theme's strength; a dark page dims instead of fogging.
- Select options and menu rows say their place ("2 of 5"), a listbox's current option is selected, and the open surface is named.
- A sidebar row's count is read to assistive technology ("Count 3").
- Rune is taken with `default-features = false` in this repository, so its lock file no longer holds the MPL-2.0 Ogg audio crates
  (`symphonia`, `opuscule`); an app drops them the same way (README, "Licences"). Apps that keep Rune's defaults are unchanged.

### Breaking

| Break | Migration |
| --- | --- |
| `agent::Status::Stopped` added; `Status::ALL` is `[Status; 8]` | add a `Status::Stopped` arm where you match without a wildcard |
| `popup::Row` has a public `image` field | use `Row::new(..)`; in a struct literal add `image: None` |
| `Palette`, `Theme`, `Base` and `ThemeFile` gained fields | only struct literals break; build from a theme file instead |
| `popup`, `kbd`, `dialog`, `sheet`, `command` and `portrait` need `rune-ui-style` 0.10 | move `rune-ui-style` and `rune-ui-themes` to `v0.10.0`, then `rune-ui update` |

## 0.9.0 and earlier

See the README: "0.9.0: what the first app needed" and its upgrade notes, and the wave sections above them.
