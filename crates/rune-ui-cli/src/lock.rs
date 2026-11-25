//! `rune-ui.lock`: what was copied into the app, at which version, and each file's hash as written. `diff --check` compares the files
//! with it; `add` and `update` keep it current. Written by hand-formatting (stable, diff-friendly); read with `toml`.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct LockedFile {
    pub target: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LockedItem {
    pub version: String,
    pub files: Vec<LockedFile>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lock {
    pub rune: String,
    pub items: BTreeMap<String, LockedItem>,
}

impl Lock {
    pub fn parse(src: &str) -> Result<Lock, String> {
        let t: toml::Table = src.parse().map_err(|e| format!("rune-ui.lock: {e}"))?;
        if t.get("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err("rune-ui.lock: version must be 1".to_owned());
        }
        let mut lock = Lock { rune: t.get("rune").and_then(|v| v.as_str()).unwrap_or_default().to_owned(), items: BTreeMap::new() };
        for it in t.get("item").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let name = it.get("name").and_then(|v| v.as_str()).ok_or("rune-ui.lock: an item lacks its name")?.to_owned();
            let version = it.get("version").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
            let files = it.get("files").and_then(|v| v.as_array()).cloned().unwrap_or_default().iter().filter_map(|f| Some(LockedFile { target: f.get("target")?.as_str()?.to_owned(), sha256: f.get("sha256")?.as_str()?.to_owned() })).collect();
            lock.items.insert(name, LockedItem { version, files });
        }
        Ok(lock)
    }

    pub fn render(&self) -> String {
        let mut s = String::from("# rune-ui.lock: what `rune-ui` copied into this app, and each file's hash as written. Kept by `rune-ui add`; commit it.\n");
        s.push_str("version = 1\n");
        s.push_str(&format!("rune = {:?}\n", self.rune));
        for (name, it) in &self.items {
            s.push_str(&format!("\n[[item]]\nname = {name:?}\nversion = {:?}\nfiles = [\n", it.version));
            for f in &it.files {
                s.push_str(&format!("  {{ target = {:?}, sha256 = {:?} }},\n", f.target, f.sha256));
            }
            s.push_str("]\n");
        }
        s
    }

    /// Which item owns `target`.
    pub fn owner(&self, target: &str) -> Option<&str> {
        self.items.iter().find(|(_, it)| it.files.iter().any(|f| f.target == target)).map(|(n, _)| n.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lock_survives_a_round_trip() {
        let mut l = Lock { rune: "abc".to_owned(), items: BTreeMap::new() };
        l.items.insert("button".to_owned(), LockedItem { version: "0.2.0".to_owned(), files: vec![LockedFile { target: "src/ui/button.rs".to_owned(), sha256: "00".to_owned() }] });
        assert_eq!(Lock::parse(&l.render()).unwrap(), l);
        assert_eq!(l.owner("src/ui/button.rs"), Some("button"));
    }
}
