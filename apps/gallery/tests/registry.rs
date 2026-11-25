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

/// Every `.rs` file of every item (sources and stories).
fn sources() -> Vec<(String, String)> {
    let mut v = Vec::new();
    for (name, _) in items() {
        for e in std::fs::read_dir(root().join(&name)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "rs") {
                v.push((format!("{name}/{}", p.file_name().unwrap().to_string_lossy()), std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    v
}

#[test]
fn no_item_touches_the_filesystem_processes_or_the_network() {
    for (file, src) in sources() {
        for (i, line) in src.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for banned in ["std::fs", "std::process", "std::net", "std::env"] {
                assert!(!line.contains(banned), "{file}:{}: `{banned}`: a copied component hands such work to the app through a callback", i + 1);
            }
        }
    }
}

/// The parameters of each `fn` in `src` (top-level commas between its parentheses, `self` included), with its name.
fn arities(src: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let b = src.as_bytes();
    let mut at = 0;
    while let Some(off) = src[at..].find("fn ") {
        let start = at + off;
        at = start + 3;
        if start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'_') {
            continue;
        }
        let name: String = src[at..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if name.is_empty() {
            continue;
        }
        // Skip generics (`<...>`, nesting) to the opening parenthesis.
        let mut i = at + name.len();
        let mut angle = 0i32;
        while i < b.len() {
            match b[i] {
                b'<' => angle += 1,
                b'>' if i > 0 && b[i - 1] != b'-' && b[i - 1] != b'=' => angle -= 1,
                b'(' if angle == 0 => break,
                b'{' | b';' => break,
                _ => {}
            }
            i += 1;
        }
        if i >= b.len() || b[i] != b'(' {
            continue;
        }
        let (mut depth, mut commas, mut any) = (0i32, 0usize, false);
        let mut j = i + 1;
        while j < b.len() {
            match b[j] {
                b'(' | b'[' | b'{' | b'<' => depth += 1,
                b'>' if b[j - 1] == b'-' || b[j - 1] == b'=' => {}
                b')' | b']' | b'}' | b'>' if depth > 0 => depth -= 1,
                b')' => break,
                b',' if depth == 0 => commas += 1,
                c if !c.is_ascii_whitespace() => any = true,
                _ => {}
            }
            j += 1;
        }
        let trailing = src[i + 1..j].trim_end().ends_with(',');
        out.push((name, if any { commas + 1 - usize::from(trailing) } else { 0 }));
    }
    out
}

#[test]
fn no_item_needs_a_workspace_lint_allowance() {
    // A copied item lands in an app with clippy's defaults: it may not lean on rune-ui's own lint table, nor allow lints itself.
    let manifest = std::fs::read_to_string(root().join("../Cargo.toml")).unwrap();
    let table = manifest.split("[workspace.lints.clippy]").nth(1).unwrap_or_default().split("\n[").next().unwrap_or_default();
    assert!(!table.contains("allow"), "the workspace allows a lint the copies would trip in an app: {table}");
    for (file, src) in sources() {
        assert!(!src.contains("#[allow(") && !src.contains("#![allow("), "{file}: allows a lint");
        for (name, n) in arities(&src) {
            assert!(n <= 7, "{file}: `fn {name}` takes {n} arguments; clippy's too_many_arguments stops at 7 (take a struct)");
        }
    }
}
