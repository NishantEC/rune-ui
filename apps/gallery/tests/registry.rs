//! The registry's own rules: hashes in `item.toml` match the files, no two items write the same path, every component has its stories,
//! accessibility check and snapshot, and components read tokens only (no hex colours, no raw `Color` values).

use std::collections::HashMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry")
}

fn items() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = std::fs::read_dir(root()).unwrap().filter_map(|e| {
        let d = e.unwrap().path();
        let toml = std::fs::read_to_string(d.join("item.toml")).ok()?;
        Some((d.file_name().unwrap().to_string_lossy().into_owned(), toml))
    }).collect();
    v.sort();
    v
}

fn values<'a>(toml: &'a str, key: &str) -> Vec<&'a str> {
    toml.lines().filter_map(|l| l.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim().trim_matches('"'))).collect()
}

#[test]
fn hashes_match_and_no_two_items_write_the_same_path() {
    use sha2::Digest;
    let mut owners: HashMap<String, String> = HashMap::new();
    for (name, toml) in items() {
        assert_eq!(values(&toml, "name"), vec![name.as_str()], "{name}: folder and name agree");
        for ((path, target), sha) in values(&toml, "path").into_iter().zip(values(&toml, "target")).zip(values(&toml, "sha256")) {
            let bytes = std::fs::read(root().join(&name).join(path)).unwrap();
            let got = format!("{:x}", sha2::Sha256::digest(&bytes));
            assert_eq!(got, sha, "{name}/{path} changed: run `rune-ui registry hash`");
            if let Some(other) = owners.insert(target.to_owned(), name.clone()) {
                panic!("{name} and {other} both write {target}");
            }
        }
    }
}

#[test]
fn every_component_has_stories_an_accessibility_check_and_a_snapshot() {
    for (name, toml) in items() {
        if values(&toml, "type") == ["component"] {
            for f in ["stories.rs", "a11y.rs", "snapshot.txt"] {
                assert!(root().join(&name).join(f).exists(), "{name} lacks {f}");
            }
        }
    }
}

#[test]
fn components_read_tokens_only() {
    let banned = ["Color::rgb", "Color {", "Color::WHITE", "Color::BLACK", "rune::hex", "hex(\"", "rgb8("];
    for (name, _) in items() {
        let path = root().join(&name).join(format!("{name}.rs"));
        let src = std::fs::read_to_string(&path).unwrap();
        for (i, line) in src.lines().enumerate() {
            if line.contains("guard: allow") || line.trim_start().starts_with("//") {
                continue;
            }
            for b in banned {
                assert!(!line.contains(b), "{name}.rs:{}: `{b}`: read a token instead", i + 1);
            }
            let hexish = line.match_indices('#').any(|(at, _)| line[at + 1..].chars().take(6).filter(char::is_ascii_hexdigit).count() == 6 && !line[..at].ends_with('"'));
            assert!(!hexish, "{name}.rs:{}: a hex colour: read a token instead", i + 1);
        }
    }
}
