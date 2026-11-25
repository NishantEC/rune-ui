//! The registry: items and their files, from the binary (embedded at build time) or from a folder (`--registry`).

use std::collections::BTreeMap;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

/// One file an item writes into the app.
#[derive(Clone, Debug)]
pub struct ItemFile {
    pub path: String,
    pub target: String,
    pub sha256: String,
    /// A story file, copied only once the gallery block is installed.
    pub story: bool,
    /// Bytes, not text (`binary = true`, or not valid UTF-8): `update` replaces it whole, never merges it.
    pub binary: bool,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub title: String,
    pub version: String,
    pub kind: String,
    pub group: String,
    pub rune: String,
    pub needs: Vec<String>,
    /// What the item's stories use besides the item (they are copied only when these are installed).
    pub story_needs: Vec<String>,
    pub cargo: Vec<String>,
    /// Cargo features the item needs, as `crate/feature` (`rune-kit/markdown`); `add` turns them on in the app's Cargo.toml.
    pub features: Vec<String>,
    pub files: Vec<ItemFile>,
    /// File name to bytes, for every file in the item's folder.
    pub contents: BTreeMap<String, Vec<u8>>,
}

pub struct Registry {
    pub items: BTreeMap<String, Item>,
}

fn strings(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

pub fn parse_item(toml_src: &str, contents: BTreeMap<String, Vec<u8>>) -> Result<Item, String> {
    let t: toml::Table = toml_src.parse().map_err(|e| format!("item.toml: {e}"))?;
    let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_owned).ok_or_else(|| format!("item.toml: `{k}` is missing"));
    if t.get("schema").and_then(|v| v.as_integer()) != Some(1) {
        return Err("item.toml: schema must be 1".to_owned());
    }
    let req = t.get("requires").and_then(|v| v.as_table()).ok_or("item.toml: [requires] is missing")?;
    let deps = t.get("dependencies").and_then(|v| v.as_table());
    let mut files = Vec::new();
    for f in t.get("files").and_then(|v| v.as_array()).ok_or("item.toml: [[files]] is missing")? {
        let g = |k: &str| f.get(k).and_then(|v| v.as_str()).map(str::to_owned).ok_or_else(|| format!("item.toml: a file lacks `{k}`"));
        files.push(ItemFile { path: g("path")?, target: g("target")?, sha256: g("sha256")?, story: f.get("role").and_then(|v| v.as_str()) == Some("story"), binary: f.get("binary").and_then(|v| v.as_bool()) == Some(true) });
    }
    for f in &files {
        match contents.get(&f.path) {
            Some(bytes) if sha256(bytes) == f.sha256 => {}
            Some(_) => return Err(format!("{} does not match its sha256 (run `rune-ui registry hash`)", f.path)),
            None => return Err(format!("{} is missing", f.path)),
        }
    }
    Ok(Item {
        name: s("name")?,
        title: s("title")?,
        version: s("version")?,
        kind: s("type")?,
        group: s("group")?,
        rune: req.get("rune").and_then(|v| v.as_str()).unwrap_or_default().to_owned(),
        needs: strings(deps.and_then(|d| d.get("registry"))),
        story_needs: strings(deps.and_then(|d| d.get("stories"))),
        cargo: strings(deps.and_then(|d| d.get("cargo"))),
        features: strings(deps.and_then(|d| d.get("features"))),
        files,
        contents,
    })
}

impl Registry {
    pub fn embedded() -> Result<Registry, String> {
        let mut items = BTreeMap::new();
        for (dir, files) in EMBEDDED {
            let contents: BTreeMap<String, Vec<u8>> = files.iter().map(|(n, b)| ((*n).to_owned(), b.to_vec())).collect();
            let toml_src = String::from_utf8_lossy(&contents["item.toml"]).into_owned();
            let item = parse_item(&toml_src, contents).map_err(|e| format!("{dir}: {e}"))?;
            items.insert(item.name.clone(), item);
        }
        Ok(Registry { items })
    }

    pub fn from_dir(root: &Path) -> Result<Registry, String> {
        let mut items = BTreeMap::new();
        let mut dirs: Vec<_> = std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.join("item.toml").exists()).collect();
        dirs.sort();
        for d in dirs {
            let mut contents = BTreeMap::new();
            for f in std::fs::read_dir(&d).map_err(|e| e.to_string())?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()) {
                contents.insert(f.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read(&f).map_err(|e| e.to_string())?);
            }
            let toml_src = String::from_utf8_lossy(&contents["item.toml"]).into_owned();
            let item = parse_item(&toml_src, contents).map_err(|e| format!("{}: {e}", d.display()))?;
            items.insert(item.name.clone(), item);
        }
        Ok(Registry { items })
    }

    pub fn get(&self, name: &str) -> Result<&Item, String> {
        self.items.get(name).ok_or_else(|| format!("no item called `{name}` (run `rune-ui list`)"))
    }

    /// `names` and everything they need, needs first, each once.
    pub fn closure(&self, names: &[String]) -> Result<Vec<String>, String> {
        fn visit(r: &Registry, n: &str, seen: &mut Vec<String>, stack: &mut Vec<String>) -> Result<(), String> {
            if seen.iter().any(|s| s == n) {
                return Ok(());
            }
            if stack.iter().any(|s| s == n) {
                return Err(format!("items need each other in a loop: {} -> {n}", stack.join(" -> ")));
            }
            stack.push(n.to_owned());
            for d in &r.get(n)?.needs {
                visit(r, d, seen, stack)?;
            }
            stack.pop();
            seen.push(n.to_owned());
            Ok(())
        }
        let mut seen = Vec::new();
        for n in names {
            visit(self, n, &mut seen, &mut Vec::new())?;
        }
        Ok(seen)
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(bytes))
}
