# Recipes reference

Everything here is data in `crates/rune-ui-style`. The gallery's **Materials** tab (`apps/gallery/scripts/snap.sh materials`) builds each
recipe one layer at a time and then shows each of its states.

## Layers (`Layer`)

Bottom of the stack first. Outer shadows always sit under the shape; the rest draw in Rune's order: fill, gradient, inner shadows and
edges, border, children, outline.

| Layer | Rune call | Use |
| --- | --- | --- |
| `Fill(c)` | `fill` | a flat colour (several composite into one) |
| `Gradient { top, bottom }` | `gradient` | the raised and ink materials |
| `Linear { angle, stops }` | `linear_gradient` | scrims, eased fades |
| `InnerTopEdge { color, width }` | inset `box_shadow` pushed down by its width | the lit top edge |
| `InnerShadow(s)` | inset `box_shadow` | a well's top |
| `InnerRim { color, width }` | inset `box_shadow` with spread | the ink's rim, the outline variant |
| `Shadow(s)` | `box_shadow` | drop shadows; several make a smooth stack |
| `Ring { width, offset, color }` | `outline` | focus, invalid, open |
| `Border { width, color }` | `border` | structure only (never on a control) |
| `Wash(c)` | folded into the fill and gradient | hover, press, selected tints |
| `Scale(k)` | `scale` | a press |
| `Opacity(o)` | `opacity` | disabled |
| `BackdropBlur(sigma)` | `backdrop_blur` | glass |
| `Grain { amount }` | none: `Op::Skipped` | atmosphere; Rune has no primitive yet |

`plan(&recipe)` returns the `Op`s; `paint(el, &recipe)` applies them; tests read the plan.

## States and the resolver

The closed set (`State`): `Rest, Hover, Pressed, FocusVisible, Disabled, Selected, Invalid, Loading, Open`. A `Material` is a base
recipe plus a `Delta` per state (`replace`, then `remove` layer kinds, then `add` layers). `resolve(&material, states)`:

1. start from the base;
2. apply `selected` if selected;
3. apply the one interaction state that wins: **disabled > loading > pressed > hover > rest**;
4. unless disabled, apply `invalid`, then `open`, then `focus` on top.

`paint_state` paints the resolved recipe and, when nothing is pinned and the control is live, hands Rune the material's live hover and
press washes, press scale and focus ring (`Material::live`) so they animate at Rune's rate.

## Materials (`Theme::mat`)

| Material | Base | Hover | Pressed | Focus | Disabled | Selected | Invalid / Open |
| --- | --- | --- | --- | --- | --- | --- | --- |
| raised | gradient, lit edge, contact shadow | none | no shadow, scale 0.96 | ring outside | no shadow, opacity | becomes a well | |
| ink | dark (light mode) or light (dark mode) gradient, inner rim, lit edge, contact | none | sink | ring | dim | | |
| danger | the ink recipe in `danger.solid` | none | sink | ring | dim | | |
| well | `input.bg` tint, inner shadow | none | | ring hugging it | dim | | danger ring / focus ring |
| floating | `floating.bg`, smooth stack, baked hairline ring | | | | | | |
| slab | `surface.bg` | | | | | | |
| ghost | nothing | wash | wash + press wash + sink | ring | dim | `selected` fill | |
| row | nothing | wash | wash + press wash (no sink) | ring inside | dim | `selected` fill | |
| outline | inner rim | wash | wash + sink | ring | dim | becomes a well | |
| thumb | a light gradient disc, lit edge, small shadows | | | | dim | | |
| chip | raised plus a soft lift shadow | none | sink | ring | dim | becomes a well | |
| atmosphere (recipe) | `surface.bg`, grain | | | | | | |

## Component recipes (`recipes`)

| Function | Variants | Parts |
| --- | --- | --- |
| `button(t, ButtonVariant)` | Default (ink), Secondary (raised), Outline, Ghost, Destructive (danger), Link | root, label, icon |
| `toggle(t, outline)` | ghost or outline | label and icon: second ink, first when selected |
| `field(t)` | | root: well; label: input text; icon: third ink, danger when invalid |
| `row(t)` | | row material; second ink, first when selected |
| `check(t, on)` | | a well off, the ink on (no sink) |
| `series(t, i)` | | a chart series colour: accent, success, warning, danger, second ink, then fainter laps |
| `Tone::colors(t)` | Neutral, Info, Success, Warning, Danger | (wash, words, solid, words on solid) |

Every component takes `.recipe(|parts| …)` (Button now; the others as they need it) to change its parts before painting.

## Tokens

Three base knobs: `spacing` (the grid unit, 4), `radius` (the ladder's middle, 8: xs 3, sm 5, md 8, lg 11, xl 14, xxl 18) and `density`.
Density has two named settings, **Comfortable** (1.0, the default) and **Compact** (0.8), and everything between is interpolated and
rounded to whole points. The 40 point hit area never shrinks.

| Follows density (`Theme::control`, `Theme::metrics`, `Theme::ty`) | Compact (0.8) | Comfortable (1.0) |
| --- | --- | --- |
| Control heights sm / md / lg | 26 / 30 / 36 | 32 / 36 / 44 |
| Control side padding sm / md / lg | 10 / 12 / 16 | 12 / 16 / 20 |
| Icon sizes sm / md | 12 / 14 | 14 / 16 |
| Gap inside a control, label to control | 6, 6 | 8, 8 |
| Between stacked fields | 12 | 20 |
| List, menu, sidebar, command rows | 30 | 36 |
| Card row: least height, padding above and below | 56, 10 | 64, 14 |
| Slab, card and accordion side padding | 16 | 20 |
| Table rows, header | 40, 36 | 48, 40 |
| Dialog and sheet padding | 20 | 24 |
| Between groups | 40 | 56 |
| Type: control words, hints, small, reading | 13/18, 12/16, 11/15, 14/21 | 14/20, 13/18, 12/16, 15/22 |

rune-kit's select and combobox option rows are a fixed 28 points (rune-kit has no knob for them); its standard menu rows follow the
density's row height through the line height `to_kit()` gives them.

| Tier | Where in the file | Keys |
| --- | --- | --- |
| 1, ramps | `color` | `neutral` (0, 25, 50, 100, 150, 200, 300 … 900, 925, 950, 975, 1000); `blue`, `green`, `amber`, `red` (50 … 950) |
| 2, roles | `modes.<mode>.role` | `page`, `surface`, `floating`, `muted`, `subtle`, `primary`, `secondary`, `accent`, `input` (each `bg` + `fg`); `info`, `success`, `warning`, `danger` (`bg`, `fg`, `solid`, `on-solid`); `line`, `ring` |
| states | `modes.<mode>.state` | `hover`, `pressed`, `selected` (alphas of `page.fg`), `disabled` (an opacity) |
| 3, component | `modes.<mode>.component` | `raised-top`, `raised-bottom`, `raised-edge`, `ink-top`, `ink-bottom`, `ink-rim`, `ink-edge`, `well-shadow`, `thumb`, `float-ring`, `shade` |
| numbers | `modes.<mode>.number` | `elevation` (shadow strength), `well-blur`, `grain` |

Values in modes alias ramp steps: `"{neutral.900}"`, or at an alpha: `"{neutral.1000} / 0.05"`.

Scales that are not knobs: the type ramp (xs 11/15, sm 12/16, base 13/18, md 14/21, lg 16/22, xl 20/26, xxl 26/32; weights 400, 500,
600), elevation steps (xs contact, sm card, md floating, lg dialog; smooth stacks of 2 to 5 layers), motion (fast 120 ms, base 200, slow
300, one ease-out curve, press 0.96, popups from 0.98 with a 4 point drift).
