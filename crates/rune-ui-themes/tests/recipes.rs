//! Every shipped theme's recipes paint the layers the principles ask for (docs/principles.md), in both appearances.

use rune::{Color, Shadow};
use rune_ui_style::{LayerKind, Look, Op, State, Theme, contrast, lightness, over, plan, resolve};
use rune_ui_themes::{ALL, theme};

fn themes() -> Vec<Theme> {
    ALL.iter().map(Theme::from_data).collect()
}

fn shadows(ops: &[Op]) -> Vec<Shadow> {
    ops.iter().filter_map(|o| if let Op::BoxShadow(s) = o { Some(*s) } else { None }).collect()
}

fn gradient(ops: &[Op]) -> (Color, Color) {
    ops.iter().find_map(|o| if let Op::Gradient { top, bottom } = o { Some((*top, *bottom)) } else { None }).expect("a gradient")
}

#[test]
fn raised_is_a_lit_gradient_with_a_contact_shadow_and_no_outline() {
    for t in themes() {
        let ops = plan(&t.mat.raised.rest);
        let (top, bottom) = gradient(&ops);
        assert!(lightness(top) > lightness(bottom), "{}: lighter on top", t.name);
        let s = shadows(&ops);
        assert!(s[0].inset && s[0].y == 1.0 && s[0].blur == 0.0, "{}: a 1 point lit top edge first: {s:?}", t.name);
        assert!(s[1..].iter().all(|s| !s.inset && s.spread == 0.0), "{}: then only soft contact shadows", t.name);
        assert!(!ops.iter().any(|o| matches!(o, Op::Border { .. } | Op::Outline { .. })), "{}: no outline at rest", t.name);
    }
}

#[test]
fn raised_answers_a_press_by_sinking_not_by_colour_and_stays_down_when_selected() {
    for t in themes() {
        let m = &t.mat.raised;
        assert_eq!(m.hover, m.rest, "{}: hover changes nothing", t.name);
        assert_eq!(m.pressed.scale_of(), Some(t.motion.press_scale));
        assert!(!m.pressed.has(LayerKind::Shadow), "{}: pressed sits on the page", t.name);
        assert!(m.pressed.wash_of().is_none(), "{}: no colour change on press", t.name);
        assert_eq!(m.selected, t.mat.well.rest, "{}: selected is pressed in: a well", t.name);
        assert_eq!(m.live().press_scale, Some(0.96));
    }
}

#[test]
fn ink_is_the_raised_recipe_glossier_with_its_rim_under_its_edge() {
    for t in themes() {
        let ops = plan(&t.mat.ink.rest);
        let (top, bottom) = gradient(&ops);
        let s = shadows(&ops);
        assert!(s[0].inset && s[0].spread == 1.0, "{}: the rim first", t.name);
        assert!(s[1].inset && s[1].y == 1.0, "{}: the lit edge on it", t.name);
        let page = t.c.page;
        if t.dark {
            assert!(lightness(bottom) > 0.85, "{}: dark's ink is near white", t.name);
        } else {
            assert!(lightness(top) < 0.35, "{}: light's ink is near black", t.name);
        }
        assert!(contrast(over(page, bottom), page) > 10.0, "{}: the ink is the loudest object on the page", t.name);
        assert!(contrast(t.c.on_ink, bottom) >= 4.5, "{}: its words read", t.name);
    }
}

#[test]
fn a_well_is_a_tint_pressed_in_with_a_ring_that_hugs_it_on_focus() {
    for t in themes() {
        let ops = plan(&t.mat.well.rest);
        assert!(matches!(ops[0], Op::Fill(_)), "{}: a flat tint", t.name);
        let s = shadows(&ops);
        assert_eq!(s.len(), 1);
        assert!(s[0].inset && s[0].y > 0.0 && s[0].blur > 0.0, "{}: a soft shadow inside its top edge", t.name);
        let focused = plan(&t.mat.well.focused);
        assert!(focused.contains(&Op::Outline { width: t.control.ring, offset: 0.0, color: t.c.focus }), "{}: focus is a ring, never a border", t.name);
        assert!(shadows(&focused).iter().any(|s| s.inset), "{}: and keeps its inner shadow", t.name);
        let Op::Fill(fill) = ops[0] else { unreachable!() };
        let painted = over(t.c.page, fill);
        if t.dark {
            assert!(lightness(painted) > lightness(t.c.page), "{}: on dark a well is a lighter tint", t.name);
        } else {
            assert!(lightness(painted) < lightness(t.c.page), "{}: on light a well is below the page", t.name);
        }
    }
}

#[test]
fn floating_is_a_smooth_stack_with_its_hairline_baked_in_last() {
    for t in themes() {
        let s = shadows(&plan(&t.mat.floating.rest));
        assert!(s.len() >= 4, "{}: many faint layers", t.name);
        assert!(s.iter().all(|s| !s.inset));
        let ring = s.last().unwrap();
        assert!(ring.spread == 1.0 && ring.blur == 0.0 && ring.y == 0.0, "{}: the last layer is the ring", t.name);
        let drops = &s[..s.len() - 1];
        assert!(drops.windows(2).all(|w| w[0].blur < w[1].blur), "{}: closest first", t.name);
        if t.dark {
            assert!(ring.color.r > 0.5, "{}: a light ring on dark", t.name);
        } else {
            assert!(ring.color.r < 0.5, "{}: a dark ring on light", t.name);
        }
    }
}

#[test]
fn every_control_material_has_a_focus_ring_a_disabled_look_and_no_border() {
    for t in themes() {
        for m in [&t.mat.raised, &t.mat.ink, &t.mat.danger, &t.mat.well, &t.mat.ghost, &t.mat.chip] {
            assert!(m.live().focus_ring.is_some(), "{}: focus ring", t.name);
            assert_eq!(plan(&m.disabled).iter().find_map(|o| if let Op::Opacity(o) = o { Some(*o) } else { None }), Some(t.control.disabled));
            for s in State::ALL {
                assert!(!m.state(s).has(LayerKind::Border), "{}: borders are for structure, not controls", t.name);
            }
        }
    }
}

#[test]
fn the_ghost_is_nothing_until_hovered() {
    for t in themes() {
        assert!(plan(&t.mat.ghost.rest).is_empty());
        assert_eq!(plan(&t.mat.ghost.hover), vec![Op::Fill(t.c.hover)]);
        let pressed = plan(&t.mat.ghost.pressed);
        assert!(pressed.contains(&Op::Scale(0.96)));
        assert_eq!(t.mat.ghost.live().hover, Some(t.c.hover));
    }
}

#[test]
fn a_slab_is_one_fill_with_no_edge_and_no_shadow() {
    for t in themes() {
        assert_eq!(plan(&t.mat.slab.rest), vec![Op::Fill(t.c.surface)]);
    }
}

#[test]
fn grain_is_carried_but_skipped_because_rune_cannot_draw_it() {
    for t in themes() {
        assert_eq!(plan(&t.mat.atmosphere).last(), Some(&Op::Skipped(LayerKind::Grain)));
    }
}

#[test]
fn a_pinned_pressed_look_is_the_pressed_recipe() {
    let t = theme(false);
    assert_eq!(resolve(&t.mat.raised, Look::Pressed, false, false), t.mat.raised.pressed);
    assert_eq!(resolve(&t.mat.raised, Look::Live, true, false), t.mat.raised.disabled);
}

#[test]
fn text_inks_keep_their_contrast_on_the_page_and_the_slab() {
    for t in themes() {
        for bg in [t.c.page, t.c.surface] {
            assert!(contrast(t.c.ink, bg) >= 12.0, "{}: first ink", t.name);
            assert!(contrast(t.c.ink_muted, bg) >= 4.5, "{}: second ink", t.name);
            assert!(contrast(t.c.ink_faint, bg) >= 3.0, "{}: third ink {:.2}", t.name, contrast(t.c.ink_faint, bg));
        }
        for (word, tone) in [("info", t.c.info), ("success", t.c.success), ("warning", t.c.warning), ("danger", t.c.danger)] {
            let got = contrast(tone.ink, over(t.c.page, tone.wash));
            assert!(got >= 4.5, "{}: {word}'s words on its wash: {got:.2}", t.name);
        }
        assert!(contrast(t.c.on_danger, t.c.danger_solid) >= 4.5, "{}: a destructive button's words", t.name);
        assert!(contrast(t.c.focus, t.c.page) >= 3.0, "{}: the focus ring shows", t.name);
    }
}

#[test]
fn themes_are_found_by_name() {
    assert_eq!(rune_ui_themes::by_name("dark").map(|t| t.dark), Some(true));
    assert!(rune_ui_themes::by_name("nope").is_none());
}
