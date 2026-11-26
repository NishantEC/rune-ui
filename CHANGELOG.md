# Changelog

The README's release sections (`0.9.0: what the first app needed`, `0.10.0: closing the gaps`) carry the full tables of options and
keys; this file is the short version for people upgrading. rune-ui is pinned to one Rune rev per release (`RUNE_REV`); 0.10.0 keeps
`53f6d36a`.

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
