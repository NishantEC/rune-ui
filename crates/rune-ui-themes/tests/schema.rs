//! Theme files are checked: the shipped one parses, unknown and missing keys fail with their paths, bad aliases fail, the light and
//! dark modes have the same keys, and the JSON Schema names exactly the keys the parser accepts.

use rune_ui_style::ThemeFile;
use rune_ui_style::tokens::{BASE_KEYS, BASE_NAMES, BASE_WORDS, COMPONENT_KEYS, HUE_STEPS, LINE_ROLES, NEUTRAL_STEPS, NUMBER_KEYS, OPTIONAL_BASE_NAMES, OPTIONAL_NUMBER_KEYS, OPTIONAL_RAMPS, OPTIONAL_ROLES, RAMPS, STATE_KEYS, SURFACE_ROLES, TONE_ROLES};
use rune_ui_themes::{ALL, DEFAULT, SCHEMA};
use serde_json::Value;

fn edit(f: impl FnOnce(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(DEFAULT).unwrap();
    f(&mut v);
    v.to_string()
}

#[test]
fn every_shipped_theme_parses() {
    for (name, json) in ALL {
        let f = ThemeFile::parse(json).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(f.name, name);
    }
}

#[test]
fn an_unknown_key_is_an_error_with_its_path() {
    let json = edit(|v| {
        v["modes"]["light"]["role"]["pgae"] = serde_json::json!({ "bg": { "$value": "{neutral.0}" }, "fg": { "$value": "{neutral.0}" } });
        v["base"]["gutter"] = serde_json::json!({ "$value": 3 });
    });
    let errs = ThemeFile::parse(&json).unwrap_err();
    assert!(errs.iter().any(|e| e == "modes.light.role.pgae: unknown key"), "{errs:?}");
    assert!(errs.iter().any(|e| e == "base.gutter: unknown key"), "{errs:?}");
}

#[test]
fn a_font_is_a_name_resolved_through_the_installed_fonts() {
    for name in ["inter", "system", "serif", "mono"] {
        let f = ThemeFile::parse(&edit(|v| v["base"]["font"]["$value"] = Value::from(name))).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(f.base.font.name, name);
    }
    let errs = ThemeFile::parse(&edit(|v| v["base"]["font"]["$value"] = Value::from("comic-sans"))).unwrap_err();
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].starts_with("base.font: no font called \"comic-sans\"; this app has inter, system, serif, mono"), "{errs:?}");
    assert!(errs[0].contains("rune-ui add font_"), "says how to add one: {errs:?}");
    // Installing a font item makes its name valid.
    const FACE: rune_ui_style::Font = rune_ui_style::Font::embedded("comic-sans", "Comic", "Comic", &[]).size_adjust(1.1);
    rune_ui_style::fonts::install(&[FACE]);
    let f = ThemeFile::parse(&edit(|v| v["base"]["font"]["$value"] = Value::from("comic-sans"))).unwrap();
    assert_eq!(f.base.font.size_adjust, 1.1);
    let t = rune_ui_style::Theme::from_file(&f, false);
    assert_eq!(t.ty.base.family, rune::Family::Named("Comic"));
    assert_eq!((rune_ui_style::Theme::from_json(DEFAULT, false).unwrap().ty.md.size, t.ty.md.size), (15.0, 16.5), "size-adjust scales the ramp, to half points");
}

#[test]
fn a_missing_key_is_an_error() {
    let json = edit(|v| {
        v["modes"]["dark"]["component"].as_object_mut().unwrap().remove("ink-rim");
    });
    let errs = ThemeFile::parse(&json).unwrap_err();
    assert_eq!(errs, vec!["modes.dark.component.ink-rim: missing".to_owned()]);
}

#[test]
fn a_bad_alias_or_alpha_or_hex_is_an_error() {
    let json = edit(|v| {
        v["modes"]["light"]["role"]["line"]["$value"] = Value::from("{neutral.33}");
        v["modes"]["light"]["role"]["ring"]["$value"] = Value::from("{neutral.500} / 1.5");
        v["color"]["blue"]["500"]["$value"] = Value::from("blue");
    });
    let errs = ThemeFile::parse(&json).unwrap_err();
    assert!(errs.iter().any(|e| e.starts_with("modes.light.role.line:")), "{errs:?}");
    assert!(errs.iter().any(|e| e.starts_with("modes.light.role.ring:") && e.contains("alpha")), "{errs:?}");
    assert!(errs.iter().any(|e| e == "color.blue.500: must be #rrggbb"), "{errs:?}");
}

#[test]
fn light_and_dark_have_the_same_keys() {
    let f = ThemeFile::parse(DEFAULT).unwrap();
    let keys = |m: &rune_ui_style::Mode| (m.roles.keys().cloned().collect::<Vec<_>>(), m.tones.keys().cloned().collect::<Vec<_>>(), m.component.keys().cloned().collect::<Vec<_>>(), m.state.keys().cloned().collect::<Vec<_>>(), m.number.keys().cloned().collect::<Vec<_>>());
    assert_eq!(keys(&f.light), keys(&f.dark));
    assert_ne!(f.light.role("page"), f.dark.role("page"));
}

fn props(schema: &Value, path: &[&str]) -> Vec<String> {
    let mut node = schema;
    for p in path {
        node = &node[p];
        if let Some(r) = node["$ref"].as_str() {
            node = &schema["$defs"][r.trim_start_matches("#/$defs/")];
        }
    }
    let mut keys: Vec<String> = node["properties"].as_object().unwrap().keys().filter(|k| !k.starts_with('$')).cloned().collect();
    keys.sort();
    keys
}

fn sorted(keys: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
    v.sort();
    v
}

#[test]
fn the_schema_names_exactly_what_the_parser_accepts() {
    let s: Value = serde_json::from_str(SCHEMA).unwrap();
    assert_eq!(s["additionalProperties"], Value::Bool(false));
    let base: Vec<&str> = BASE_KEYS.iter().copied().chain(BASE_WORDS.iter().map(|(k, _)| *k)).chain(BASE_NAMES).chain(OPTIONAL_BASE_NAMES).collect();
    assert_eq!(props(&s, &["properties", "base"]), sorted(&base));
    let ramps: Vec<&str> = RAMPS.iter().chain(&OPTIONAL_RAMPS).copied().collect();
    assert_eq!(props(&s, &["properties", "color"]), sorted(&ramps));
    assert_eq!(props(&s, &["properties", "color", "properties", "identity"]), sorted(&HUE_STEPS));
    assert_eq!(props(&s, &["properties", "color", "properties", "neutral"]), sorted(&NEUTRAL_STEPS));
    assert_eq!(props(&s, &["properties", "color", "properties", "blue"]), sorted(&HUE_STEPS));
    let roles: Vec<&str> = SURFACE_ROLES.iter().chain(&TONE_ROLES).chain(&LINE_ROLES).chain(&OPTIONAL_ROLES).copied().collect();
    assert_eq!(props(&s, &["properties", "modes", "properties", "light", "properties", "role"]), sorted(&roles));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "state"]), sorted(&STATE_KEYS));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "component"]), sorted(&COMPONENT_KEYS));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "number"]), sorted(&NUMBER_KEYS.iter().chain(&OPTIONAL_NUMBER_KEYS).copied().collect::<Vec<_>>()));
}

/// The default theme without its optional identity ramp and role.
fn without_identity() -> String {
    edit(|v| {
        v["color"].as_object_mut().unwrap().remove("identity");
        for m in ["light", "dark"] {
            v["modes"][m]["role"].as_object_mut().unwrap().remove("identity");
        }
    })
}

#[test]
fn the_identity_colour_is_optional_and_a_theme_without_it_falls_back_to_info() {
    let with = rune_ui_style::Theme::from_json(DEFAULT, false).unwrap();
    assert!(with.has_identity(), "the default theme sets an identity colour as the example");
    assert_ne!(with.identity(), with.c.info, "a teal of its own, not the blue");
    for dark in [false, true] {
        let t = rune_ui_style::Theme::from_json(&without_identity(), dark).expect("a theme without identity is as valid as before");
        assert!(!t.has_identity() && t.c.identity.is_none());
        assert_eq!(t.identity(), t.c.info, "components that ask for it draw in info");
        // Nothing else changes: the ink, the accent and every meaning are the same as in the theme that sets it.
        let w = rune_ui_style::Theme::from_json(DEFAULT, dark).unwrap();
        assert_eq!((t.c.primary, t.c.accent, t.c.info, t.c.ring), (w.c.primary, w.c.accent, w.c.info, w.c.ring));
    }
}

#[test]
fn an_identity_role_without_its_ramp_is_an_error_with_its_path() {
    let json = edit(|v| {
        v["color"].as_object_mut().unwrap().remove("identity");
    });
    let errs = ThemeFile::parse(&json).unwrap_err();
    assert!(errs.iter().any(|e| e.starts_with("modes.light.role.identity.bg:") && e.contains("names no ramp step")), "{errs:?}");
    let bad = edit(|v| v["color"]["identity"]["500"]["$value"] = Value::from("teal"));
    assert!(ThemeFile::parse(&bad).unwrap_err().iter().any(|e| e == "color.identity.500: must be #rrggbb"));
}

#[test]
fn the_scrim_is_black_at_the_themes_strength() {
    for dark in [false, true] {
        let t = rune_ui_style::Theme::from_json(DEFAULT, dark).unwrap();
        let c = t.scrim();
        assert_eq!((c.r, c.g, c.b), (0.0, 0.0, 0.0), "black, never the page ink: pale ink would fog a dark page");
        assert!(c.a > 0.1 && c.a < 0.8, "a veil, not a wall: {}", c.a);
    }
    let light = rune_ui_style::Theme::from_json(DEFAULT, false).unwrap().scrim().a;
    let dark = rune_ui_style::Theme::from_json(DEFAULT, true).unwrap().scrim().a;
    assert!(dark > light, "dark pages need more black to dim");
    // Set in the file, followed; left out, the usual strengths.
    let set = rune_ui_style::Theme::from_json(&edit(|v| v["modes"]["dark"]["number"]["scrim"]["$value"] = Value::from(0.7)), true).unwrap();
    assert_eq!(set.scrim().a, 0.7);
    let left_out = rune_ui_style::Theme::from_json(&edit(|v| { v["modes"]["dark"]["number"].as_object_mut().unwrap().remove("scrim"); }), true).unwrap();
    assert_eq!(left_out.scrim().a, 0.5);
    assert!(set.scrim_soft().a < set.scrim().a);
}

#[test]
fn keycaps_use_the_keys_font_or_the_themes_own() {
    let t = rune_ui_style::Theme::from_json(DEFAULT, false).unwrap();
    assert_eq!(t.keycap.family, t.ty.sm.family, "the theme's own font when keys-font is left out");
    assert_eq!(t.keycap.features, "'tnum' 1");
    let mono = rune_ui_style::Theme::from_json(&edit(|v| v["base"]["keys-font"] = serde_json::json!({ "$value": "mono" })), false).unwrap();
    assert_eq!(mono.keycap.family, rune_ui_style::Font::MONO.family);
    assert_eq!(mono.ty.sm.family, t.ty.sm.family, "only the keys change face");
    let errs = ThemeFile::parse(&edit(|v| v["base"]["keys-font"] = serde_json::json!({ "$value": "comic-sans" }))).unwrap_err();
    assert!(errs.iter().any(|e| e.starts_with("base.keys-font: no font called")), "{errs:?}");
}
