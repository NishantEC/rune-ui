//! Every shipped theme's materials paint the layers the principles ask for (docs/principles.md), in both modes.

use rune::{Color, Shadow};
use rune_ui_style::{ButtonVariant, LayerKind, Op, State, States, Theme, contrast, lightness, over, plan, recipes, resolve};
use rune_ui_themes::{ALL, theme};

fn themes() -> Vec<Theme> {
    ALL.iter().flat_map(|(_, json)| [false, true].map(|dark| Theme::from_json(json, dark).unwrap())).collect()
}

fn shadows(ops: &[Op]) -> Vec<Shadow> {
    ops.iter().filter_map(|o| if let Op::BoxShadow(s) = o { Some(*s) } else { None }).collect()
}

fn gradient(ops: &[Op]) -> (Color, Color) {
    ops.iter().find_map(|o| if let Op::Gradient { top, bottom } = o { Some((*top, *bottom)) } else { None }).expect("a gradient")
}

fn mode(t: &Theme) -> &'static str {
    if t.dark { "dark" } else { "light" }
}

#[test]
fn raised_is_a_lit_gradient_with_a_contact_shadow_and_no_outline() {
    for t in themes() {
        let ops = plan(&t.mat.raised.base);
        let (top, bottom) = gradient(&ops);
        assert!(lightness(top) > lightness(bottom), "{}: lighter on top", mode(&t));
        let s = shadows(&ops);
        assert!(s[0].inset && s[0].y == 1.0 && s[0].blur == 0.0, "{}: a 1 point lit top edge first", mode(&t));
        assert!(s[1..].iter().all(|s| !s.inset && s.spread == 0.0), "{}: then soft contact shadows", mode(&t));
        assert!(!ops.iter().any(|o| matches!(o, Op::Border { .. } | Op::Outline { .. })));
    }
}

#[test]
fn raised_sinks_on_press_without_changing_colour_and_stays_down_when_selected() {
    for t in themes() {
        let m = &t.mat.raised;
        assert!(m.hover.is_empty(), "{}: hover changes nothing", mode(&t));
        let pressed = m.state(State::Pressed);
        assert_eq!(pressed.scale_of(), Some(t.motion.press_scale));
        assert!(!pressed.has(LayerKind::Shadow) && pressed.wash_of().is_none());
        assert_eq!(m.state(State::Selected), t.mat.well.base, "{}: selected is a well", mode(&t));
        assert_eq!(m.live().press_scale, Some(0.96));
    }
}

#[test]
fn ink_is_glossier_with_its_rim_under_its_edge_and_its_words_read() {
    for t in themes() {
        let ops = plan(&t.mat.ink.base);
        let (top, bottom) = gradient(&ops);
        let s = shadows(&ops);
        assert!(s[0].inset && s[0].spread == 1.0, "rim first");
        assert!(s[1].inset && s[1].y == 1.0, "lit edge on it");
        if t.dark {
            assert!(lightness(bottom) > 0.85, "dark's ink is near white");
        } else {
            assert!(lightness(top) < 0.35, "light's ink is near black");
        }
        assert!(contrast(over(t.c.page.bg, bottom), t.c.page.bg) > 10.0, "{}: the loudest object", mode(&t));
        assert!(contrast(t.c.primary.fg, bottom) >= 4.5 && contrast(t.c.primary.fg, top) >= 4.5, "{}: its words read", mode(&t));
    }
}

#[test]
fn a_well_is_a_tint_pressed_in_with_rings_for_focus_invalid_and_open() {
    for t in themes() {
        let ops = plan(&t.mat.well.base);
        assert!(matches!(ops[0], Op::Fill(_)));
        let s = shadows(&ops);
        assert!(s.len() == 1 && s[0].inset && s[0].blur > 0.0);
        for st in [State::FocusVisible, State::Invalid, State::Open] {
            let r = t.mat.well.state(st);
            assert!(r.ring_of().is_some_and(|(_, offset, _)| offset == 0.0), "{}: {st:?} rings it, hugging", mode(&t));
            assert!(r.has(LayerKind::InnerShadow), "and keeps the inner shadow");
        }
        let Op::Fill(fill) = ops[0] else { unreachable!() };
        let painted = over(t.c.page.bg, fill);
        assert!(if t.dark { lightness(painted) > lightness(t.c.page.bg) } else { lightness(painted) < lightness(t.c.page.bg) }, "{}: a step off the page", mode(&t));
    }
}

#[test]
fn floating_is_a_smooth_stack_with_its_hairline_baked_in_last() {
    for t in themes() {
        let s = shadows(&plan(&t.mat.floating.base));
        assert!(s.len() >= 4 && s.iter().all(|s| !s.inset));
        let ring = s.last().unwrap();
        assert!(ring.spread == 1.0 && ring.blur == 0.0);
        assert!(s[..s.len() - 1].windows(2).all(|w| w[0].blur < w[1].blur), "closest first");
        assert!(if t.dark { ring.color.r > 0.5 } else { ring.color.r < 0.5 }, "{}: ring contrasts with the page", mode(&t));
    }
}

#[test]
fn every_control_material_has_a_focus_ring_a_disabled_look_and_no_border() {
    for t in themes() {
        for (name, m) in t.mat.all() {
            if ["floating", "slab"].contains(&name) {
                continue;
            }
            let disabled = plan(&m.state(State::Disabled));
            assert!(disabled.contains(&Op::Opacity(t.control.disabled)), "{name}: disabled dims");
            if name != "thumb" {
                assert!(m.live().focus_ring.is_some(), "{name}: focus ring");
            }
            for s in State::ALL {
                assert!(!m.state(s).has(LayerKind::Border), "{name}: borders are for structure");
            }
        }
    }
}

#[test]
fn every_button_variant_resolves_in_every_state_and_its_words_read() {
    for t in themes() {
        for v in ButtonVariant::ALL {
            let p = recipes::button(&t, v);
            for s in State::ALL {
                let _ = resolve(&p.root, States::rest().with(s));
            }
            let bg = plan(&p.root.base).iter().find_map(|o| match o {
                Op::Gradient { bottom, .. } => Some(*bottom),
                Op::Fill(c) => Some(*c),
                _ => None,
            });
            let surface = bg.map_or(t.c.page.bg, |c| over(t.c.page.bg, c));
            assert!(contrast(p.label.rest, surface) >= 4.5, "{} {v:?}: label on its surface {:.2}", mode(&t), contrast(p.label.rest, surface));
        }
    }
}

#[test]
fn the_ghost_is_nothing_until_hovered() {
    for t in themes() {
        assert!(plan(&t.mat.ghost.base).is_empty());
        assert_eq!(plan(&t.mat.ghost.state(State::Hover)), vec![Op::Fill(t.c.hover)]);
        assert!(plan(&t.mat.ghost.state(State::Pressed)).contains(&Op::Scale(0.96)));
        assert!(t.mat.row.live().press_scale.is_none(), "rows never sink");
    }
}

#[test]
fn grain_is_carried_but_skipped_because_rune_cannot_draw_it() {
    for t in themes() {
        assert_eq!(plan(&t.mat.atmosphere).last(), Some(&Op::Skipped(LayerKind::Grain)));
    }
}

#[test]
fn every_role_text_reads_on_its_surface() {
    for t in themes() {
        let c = &t.c;
        let on = |bg: Color| over(c.page.bg, bg);
        let check = |name: &str, fg: Color, bg: Color, min: f32| {
            let got = contrast(fg, on(bg));
            assert!(got >= min, "{} {name}: {got:.2} < {min}", mode(&t));
        };
        check("page", c.page.fg, c.page.bg, 12.0);
        check("surface", c.surface.fg, c.surface.bg, 12.0);
        check("floating", c.floating.fg, c.floating.bg, 12.0);
        check("muted", c.muted.fg, c.muted.bg, 4.5);
        check("muted on surface", c.muted.fg, c.surface.bg, 4.5);
        check("subtle", c.subtle.fg, c.subtle.bg, 3.0);
        check("subtle on surface", c.subtle.fg, c.surface.bg, 3.0);
        check("primary", c.primary.fg, c.primary.bg, 4.5);
        check("secondary", c.secondary.fg, c.secondary.bg, 4.5);
        check("accent", c.accent.fg, c.accent.bg, 4.5);
        check("input", c.input.fg, c.input.bg, 12.0);
        for (name, tone) in [("info", c.info), ("success", c.success), ("warning", c.warning), ("danger", c.danger)] {
            check(name, tone.fg, tone.bg, 4.5);
            check(&format!("{name} solid"), tone.on_solid, tone.solid, 4.5);
        }
        check("ring", c.ring, c.page.bg, 3.0);
    }
}

#[test]
fn the_shipped_theme_loads_by_name() {
    assert!(rune_ui_themes::by_name("default", true).is_some_and(|t| t.dark));
    assert!(rune_ui_themes::by_name("nope", false).is_none());
    assert_eq!(theme(false).name, "default");
}
