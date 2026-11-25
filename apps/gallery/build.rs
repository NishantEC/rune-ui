//! Generates the gallery's `ui` module from `registry/*/item.toml`, laid out as `rune-ui add` lays out an app's `src/ui/`: each item's
//! source as `ui::<name>`, its stories as `ui::stories::<name>`, `ui::stories::all()` listing them, and the accessibility checks.
//! A new item folder appears in the gallery with no list to edit.

use std::fmt::Write as _;
use std::path::PathBuf;

fn field<'a>(toml: &'a str, key: &str) -> Option<&'a str> {
    toml.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"'))
    })
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../registry").canonicalize().unwrap();
    println!("cargo::rerun-if-changed={}", root.display());
    let mut items: Vec<(String, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        let manifest = dir.join("item.toml");
        if manifest.exists() {
            println!("cargo::rerun-if-changed={}", dir.display());
            let toml = std::fs::read_to_string(&manifest).unwrap();
            items.push((field(&toml, "name").unwrap().to_owned(), dir));
        }
    }
    items.sort();
    let mut out = String::from("pub mod ui {\n");
    for (name, dir) in &items {
        writeln!(out, "    #[path = {:?}]\n    pub mod {name};", dir.join(format!("{name}.rs")).display().to_string()).unwrap();
    }
    // Font items, as `rune-ui add` lists them in an app's `ui::FONTS`.
    let fonts: Vec<&String> = items.iter().filter(|(_, d)| field(&std::fs::read_to_string(d.join("item.toml")).unwrap(), "type") == Some("font")).map(|(n, _)| n).collect();
    out.push_str("    /// Every installed font item (`rune_ui_style::fonts::install`).\n    pub const FONTS: &[rune_ui_style::Font] = &[");
    for name in &fonts {
        write!(out, "{name}::FONT, ").unwrap();
    }
    out.push_str("];\n");
    let with_stories: Vec<&(String, PathBuf)> = items.iter().filter(|(_, d)| d.join("stories.rs").exists()).collect();
    out.push_str("    pub mod stories {\n");
    for (name, dir) in &with_stories {
        writeln!(out, "        #[path = {:?}]\n        pub mod {name};", dir.join("stories.rs").display().to_string()).unwrap();
    }
    out.push_str("        /// Every item's stories (an app's copy of this list is kept by `rune-ui add`).\n        pub fn all<S: 'static>() -> Vec<super::gallery::StoryItem<S>> {\n            vec![");
    for (name, _) in &with_stories {
        write!(out, "{name}::item(), ").unwrap();
    }
    out.push_str("]\n        }\n    }\n}\n\npub mod a11y {\n");
    for (name, dir) in &with_stories {
        writeln!(out, "    #[path = {:?}]\n    pub mod {name};", dir.join("a11y.rs").display().to_string()).unwrap();
    }
    out.push_str("}\n\n/// `(item, [(key, role)])`: what each item's accessibility check expects.\npub const A11Y: &[(&str, &[(&str, &str)])] = &[\n");
    for (name, _) in &with_stories {
        writeln!(out, "    ({name:?}, a11y::{name}::CHECKS),").unwrap();
    }
    out.push_str("];\n");
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("registry.rs"), out).unwrap();
}
