# Principles

The theory behind rune-ui's look, in our own words and with our own values. The numbers live in the theme file and in
`crates/rune-ui-style`; this page explains why they are shaped the way they are.

## 1. Two materials, opposite directions

Anything you **press** is *raised*: lighter at the top than the bottom, with a one-point line of light just inside its top edge and a
faint contact shadow so it sits on the page. Anything you **put something into**, or that **holds a value**, is a *well*: a flat tint a
step off the page with a soft shadow inside its top edge.

Up means press me; down means put something in me. Tracks (switch, slider, segmented toggle group, contained tabs) are wells holding a
raised thumb, so the two materials explain the control without words. A raised toggle that stays on becomes a well: it is down.

## 2. No outlines

Surfaces are told apart by a small step in fill, and things that float by a soft shadow, never by a border round a box. Hairlines
between rows and the keyboard focus ring are the only lines; the ink's inner rim is part of the object, not a separator. Because the
lines are gone, the few cues left (a lit edge, a well's inner shadow, a hairline) carry a lot of meaning.

## 3. Depth from light

Fills step by small amounts of the page's text colour. Only floating things (menus, popovers, dialogs, sheets, toasts) cast a real drop
shadow, and that shadow is a stack of faint layers growing outward, closest first, with a hairline ring baked in as its last layer so
the edge melts into the shadow instead of being drawn.

## 4. One surface divided into rows

A group is one slab with hairlines inset from its sides, not a stack of cards. A raised card is for the one thing that should stand out.

## 5. One shape language

Radii come from a ladder built on one knob. Nested shapes are concentric: an inner radius is the outer minus the inset. A corner stays
under about four tenths of a control's height, or the continuous corner stops reading as a rounded rectangle; true pills and circles
stay round.

Corners come in two styles (`base.corners`). Round is a circular arc. Continuous is Apple's curve, which eases into the straight edges;
Rune draws it so close to a circular arc at the same radius (within 0.4 points on a 28 point corner) that the two cannot be told apart, so
continuous corners draw the whole ladder 1.25 times larger (`CONTINUOUS_GAIN`), as designers pair them on iOS. Every step grows by the
same factor, so nesting stays concentric. Circles, dots, thumbs and pills are drawn round in both styles. The Materials tab's "corner
shape" row shows the two side by side.

## 6. Colour is state, at low alpha

A meaning is a wash at low alpha behind words in its own readable ink, with a small dot; never colour alone. The primary action is
**ink** (near black in light, near white in dark), not a hue, and there is one per view. The loud solid forms (a destructive button, a
failure count) are rare.

## 7. Motion is short, caused and quiet

A press sinks the object (0.96) rather than recolouring it; raised and ink controls do not change colour on hover, ghosts and rows take a
wash. Popups grow from just under full size with a small drift. A frequent interaction lands at once. Nothing loops except work in
progress, and Rune turns motion off under Reduce Motion.

## Light and dark are not inverses

Dark is graphite, not navy. Its raised gradient is narrower, its lit edge much dimmer, its inner and drop shadows deeper, its ink a near
white with a light rim, and its floating ring light (a dark ring vanishes on a dark page). Each mode is tuned on its own and tested on its
own for contrast.

## Atmosphere is for backdrops

Texture beyond the materials (grain, a dithered glow) belongs on large backdrops like a home page, never on controls or inside a
scroller. rune-ui carries grain in its recipes; Rune cannot draw it yet.

## The rulebook these follow

Concentric radii, optical alignment (an icon's side two points tighter), shadows for depth and borders for structure, 40 point hit
areas, tabular figures for anything that changes in place, a press scale of 0.96, a visible focus ring for keyboard focus, and every
state of every component shown side by side in the gallery.
