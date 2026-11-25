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
    let (a, b) = (with(|e| e.corners = Some(Corners::Round)), with(|e| { e.radius = Some(14.0); e.corners = Some(Corners::Round) }));
    assert_eq!(a.radius.md, 8.0);
    assert_eq!(b.radius.md, 14.0);
    assert_eq!((a.base_radius, b.base_radius), (8.0, 14.0));
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
    let flat = with(|e| e.texture = Some(0.0));
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
    assert_eq!(with(|e| e.corners = Some(Corners::Round)).corners, Corners::Round);
    assert_eq!(with(|e| e.type_scale = Some(1.2)).ty.base.size, 17.0);
    assert_ne!(with(|e| e.neutral = 4).c.page.bg, with(|_| {}).c.page.bg);
    let mut g = GalleryState::default();
    g.compact = Some(true);
    assert_eq!(theme_for(JSON, false, &g).control.md, 30.0);
}

#[test]
fn what_the_editor_exports_is_a_valid_theme_file() {
    let mut g = GalleryState::default();
    g.editor = Editor { neutral: 3, accent: Some(ACCENTS[2].to_owned()), radius: Some(12.0), corners: Some(Corners::Round), type_scale: Some(1.1), shadow: 1.5, texture: Some(0.5), ..Editor::default() };
    g.compact = Some(true);
    let json = edit_theme(JSON, &g).to_string();
    let f = ThemeFile::parse(&json).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!((f.base.radius, f.base.corners, f.base.density), (12.0, Corners::Round, rune_ui_style::COMPACT));
}

#[test]
fn continuous_corners_draw_the_whole_ladder_larger_and_keep_it_concentric() {
    let (round, cont) = (with(|e| e.corners = Some(Corners::Round)), with(|e| e.corners = Some(Corners::Continuous)));
    assert_eq!(cont.base_radius, round.base_radius, "the knob is the same number");
    assert_eq!(cont.radius.md, round.radius.md * rune_ui_style::CONTINUOUS_GAIN);
    for (c, r) in [(cont.radius.sm, round.radius.sm), (cont.radius.lg, round.radius.lg), (cont.radius.xl, round.radius.xl), (cont.radius.xxl, round.radius.xxl)] {
        assert!(c > r, "every step grows ({c} vs {r})");
    }
    // Nesting still steps down by the inset, and a control's corner still stays under its share of the height.
    assert!(rune_ui_style::Radius::inner(cont.radius.lg, 4.0) < cont.radius.lg);
    assert!(rune_ui_style::Radius::fit(cont.radius.lg, cont.control.sm) <= cont.control.sm * rune_ui_style::MAX_CORNER_SHARE);
}

#[test]
fn the_font_control_lists_installed_fonts_and_writes_their_name() {
    rune_ui_style::fonts::install(gallery::ui::FONTS);
    let names = rune_ui_style::fonts::names();
    for n in ["inter", "system", "serif", "mono", "geist", "manrope", "dm-sans", "figtree", "plus-jakarta-sans", "ibm-plex-sans", "geist-mono", "jetbrains-mono"] {
        assert!(names.contains(&n), "{n} in {names:?}");
    }
    let geist = with(|e| e.font = Some("geist"));
    assert_eq!(geist.font.name, "geist");
    assert_eq!(geist.ty.base.family, rune::Family::Named("Geist"));
    let figtree = with(|e| e.font = Some("figtree"));
    assert!(figtree.ty.md.size > with(|_| {}).ty.md.size, "a small x-height is sized up");
    let mut g = GalleryState::default();
    g.editor.font = Some("ibm-plex-sans");
    assert_eq!(ThemeFile::parse(&edit_theme(JSON, &g).to_string()).unwrap().base.font.name, "ibm-plex-sans");
    // Every font a font item ships registers with Rune at start-up, Inter among them (Rune drops its own once an app brings fonts).
    let files = rune_ui_style::fonts::install(gallery::ui::FONTS);
    assert!(files.len() >= 11 && rune_ui_style::fonts::find("inter").is_some_and(|f| f.files.len() == 3));
}

/// The default theme file with its own font, radius, corners, density, type scale and material changed.
fn own_theme() -> String {
    let mut v: serde_json::Value = serde_json::from_str(JSON).unwrap();
    v["base"]["font"]["$value"] = "serif".into();
    v["base"]["radius"]["$value"] = 12.0.into();
    v["base"]["corners"]["$value"] = "round".into();
    v["base"]["density"]["$value"] = rune_ui_style::COMPACT.into();
    v["base"]["type-scale"]["$value"] = 1.1.into();
    for m in ["light", "dark"] {
        v["modes"][m]["number"]["texture"]["$value"] = 0.5.into();
    }
    v.to_string()
}

#[test]
fn the_editor_starts_from_the_theme_files_own_values_and_reset_returns_to_them() {
    let json = own_theme();
    let file = ThemeFile::parse(&json).unwrap();
    let mut g = GalleryState::default();
    let untouched = ThemeFile::parse(&edit_theme(&json, &g).to_string()).unwrap();
    assert_eq!((untouched.base.font.name, untouched.base.radius, untouched.base.corners, untouched.base.density), (file.base.font.name, file.base.radius, file.base.corners, file.base.density), "nothing set: the file's own values");
    let t = theme_for(&json, false, &g);
    assert_eq!((t.font.name, t.base_radius, t.corners, t.control.md), ("serif", 12.0, Corners::Round, 30.0));
    assert_eq!(t.ty.base.size, theme_for(&json, false, &g).ty.base.size);
    // Change everything, then do what Reset does: the file's values come back, not Inter, continuous and comfortable.
    g.editor = Editor { font: Some("inter"), radius: Some(4.0), corners: Some(Corners::Continuous), type_scale: Some(0.9), texture: Some(1.0), shadow: 2.0, ..Editor::default() };
    g.compact = Some(false);
    assert_eq!(theme_for(&json, false, &g).font.name, "inter");
    g.editor = Editor::default();
    g.compact = None;
    let back = theme_for(&json, false, &g);
    assert_eq!((back.font.name, back.base_radius, back.corners, back.control.md), ("serif", 12.0, Corners::Round, 30.0));
    assert_eq!(edit_theme(&json, &g), serde_json::from_str::<serde_json::Value>(&json).unwrap(), "an untouched editor writes the file back unchanged");
}
