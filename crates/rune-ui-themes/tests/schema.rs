//! Theme files are checked: the shipped one parses, unknown and missing keys fail with their paths, bad aliases fail, the light and
//! dark modes have the same keys, and the JSON Schema names exactly the keys the parser accepts.

use rune_ui_style::ThemeFile;
use rune_ui_style::tokens::{BASE_KEYS, BASE_WORDS, COMPONENT_KEYS, HUE_STEPS, LINE_ROLES, NEUTRAL_STEPS, NUMBER_KEYS, RAMPS, STATE_KEYS, SURFACE_ROLES, TONE_ROLES};
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
    let base: Vec<&str> = BASE_KEYS.iter().copied().chain(BASE_WORDS.iter().map(|(k, _)| *k)).collect();
    assert_eq!(props(&s, &["properties", "base"]), sorted(&base));
    assert_eq!(props(&s, &["properties", "color"]), sorted(&RAMPS));
    assert_eq!(props(&s, &["properties", "color", "properties", "neutral"]), sorted(&NEUTRAL_STEPS));
    assert_eq!(props(&s, &["properties", "color", "properties", "blue"]), sorted(&HUE_STEPS));
    let roles: Vec<&str> = SURFACE_ROLES.iter().chain(&TONE_ROLES).chain(&LINE_ROLES).copied().collect();
    assert_eq!(props(&s, &["properties", "modes", "properties", "light", "properties", "role"]), sorted(&roles));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "state"]), sorted(&STATE_KEYS));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "component"]), sorted(&COMPONENT_KEYS));
    assert_eq!(props(&s, &["properties", "modes", "properties", "dark", "properties", "number"]), sorted(&NUMBER_KEYS));
}
