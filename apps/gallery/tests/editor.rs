//! The theme editor writes the theme file the page is read from: each control changes the resolved theme and its recipes, and what it
//! exports is a valid theme file.

use gallery::ui::gallery::{ACCENTS, Editor, GalleryState, edit_theme, theme_for};
use rune_ui_style::{ButtonVariant, Corners, LayerKind, Op, Theme, ThemeFile, plan, recipes};

const JSON: &str = rune_ui_themes::DEFAULT;

fn with(f: impl FnOnce(&mut Editor)) -> Theme {
    let mut g = GalleryState::default();
    f(&mut g.editor);
    theme_for(JSON, false, &g)
}

#[test]
fn radius_changes_the_ladder_and_every_rounded_control() {
    let (a, b) = (with(|_| {}), with(|e| e.radius = Some(14.0)));
    assert_eq!(a.radius.md, 8.0);
    assert_eq!(b.radius.md, 14.0);
    assert!(b.radius.lg > a.radius.lg && b.radius.xl > a.radius.xl);
}

#[test]
fn the_accent_changes_the_accent_role_and_the_recipes_that_use_it() {
    let (a, b) = (with(|_| {}), with(|e| e.accent = Some(ACCENTS[4].to_owned())));
    assert_ne!(a.c.accent.fg, b.c.accent.fg);
    assert_ne!(recipes::button(&a, ButtonVariant::Link).label.rest, recipes::button(&b, ButtonVariant::Link).label.rest);
    assert_ne!(a.c.info.solid, b.c.info.solid);
}

#[test]
fn texture_flattens_the_raised_material_and_shadow_strength_scales_the_stacks() {
    let flat = with(|e| e.texture = 0.0);
    let grad = plan(&flat.mat.raised.base).into_iter().find_map(|o| if let Op::Gradient { top, bottom } = o { Some((top, bottom)) } else { None }).unwrap();
    assert_eq!(grad.0, grad.1, "flat: the gradient's ends meet");
    assert!(flat.mat.raised.base.layers.iter().any(|l| matches!(l, rune_ui_style::Layer::InnerTopEdge { color, .. } if color.a == 0.0)), "and the lit edge is gone");
    let (a, b) = (with(|_| {}), with(|e| e.shadow = 2.0));
    let alpha = |t: &Theme| plan(&t.mat.floating.base).iter().filter_map(|o| if let Op::BoxShadow(s) = o { Some(s.color.a) } else { None }).next().unwrap();
    assert!(alpha(&b) > alpha(&a) * 1.9);
    assert!(a.mat.floating.base.has(LayerKind::Shadow));
}

#[test]
fn corners_font_type_scale_neutral_and_density_reach_the_theme() {
    assert_eq!(with(|e| e.corners = Corners::Round).corners, Corners::Round);
    assert_eq!(with(|e| e.type_scale = 1.2).ty.base.size, 17.0);
    assert_ne!(with(|e| e.neutral = 4).c.page.bg, with(|_| {}).c.page.bg);
    let mut g = GalleryState::default();
    g.compact = true;
    assert_eq!(theme_for(JSON, false, &g).control.md, 30.0);
}

#[test]
fn what_the_editor_exports_is_a_valid_theme_file() {
    let mut g = GalleryState::default();
    g.editor = Editor { neutral: 3, accent: Some(ACCENTS[2].to_owned()), radius: Some(12.0), corners: Corners::Round, type_scale: 1.1, shadow: 1.5, texture: 0.5, ..Editor::default() };
    g.compact = true;
    let json = edit_theme(JSON, &g).to_string();
    let f = ThemeFile::parse(&json).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!((f.base.radius, f.base.corners, f.base.density), (12.0, Corners::Round, rune_ui_style::COMPACT));
}
