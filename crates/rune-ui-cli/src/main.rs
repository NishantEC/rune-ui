//! `rune-ui`: shadcn's workflow for Rune. Copies registry components into an app's `src/ui/` (yours to change), records each file's
//! version and hash in `rune-ui.lock`, tells you when your copies drift, and merges upstream changes into your edits.
//!
//! ```text
//! rune-ui init [--source <repo> | --git <url> --tag <tag>] [--dry-run]
//! rune-ui add <item>... [--dry-run] [--force]
//! rune-ui list
//! rune-ui diff [<item>...] [--check]
//! rune-ui update [<item>...] [--dry-run]
//! rune-ui registry hash [--registry <dir>]
//! ```
//!
//! Every command works on the app in the current directory (or `--app <dir>`).

mod merge;
mod registry;
mod lock;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lock::{Lock, LockedFile, LockedItem};
use registry::{Item, Registry, sha256};

const LOCK: &str = "rune-ui.lock";
/// The registry's copy of each file as it was written, so `update` can merge three ways.
const BASE: &str = ".rune-ui/base";

struct Args {
    cmd: Vec<String>,
    app: PathBuf,
    dry_run: bool,
    force: bool,
    check: bool,
    source: Option<String>,
    git: Option<(String, String)>,
    registry: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args { cmd: Vec::new(), app: PathBuf::from("."), dry_run: false, force: false, check: false, source: None, git: None, registry: None };
    let mut it = std::env::args().skip(1);
    let mut git_url = None;
    let mut tag = None;
    while let Some(x) = it.next() {
        let mut val = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match x.as_str() {
            "--dry-run" => a.dry_run = true,
            "--force" => a.force = true,
            "--check" => a.check = true,
            "--app" => a.app = PathBuf::from(val("--app")?),
            "--source" => a.source = Some(val("--source")?),
            "--git" => git_url = Some(val("--git")?),
            "--tag" => tag = Some(val("--tag")?),
            "--registry" => a.registry = Some(PathBuf::from(val("--registry")?)),
            "-h" | "--help" => a.cmd = vec!["help".to_owned()],
            f if f.starts_with("--") => return Err(format!("unknown flag {f}")),
            _ => a.cmd.push(x),
        }
    }
    if let Some(u) = git_url {
        a.git = Some((u, tag.ok_or("--git needs --tag (rune-ui pins by tag)")?));
    }
    Ok(a)
}

const HELP: &str = "rune-ui: Tailwind + shadcn for Rune.

  rune-ui init [--source <repo> | --git <url> --tag <tag>] [--dry-run]
      Adds rune, rune-kit, rune-ui-style and rune-ui-themes to Cargo.toml with Rune's [patch.crates-io] table,
      copies base and icons into src/ui/, and writes rune-ui.lock.
  rune-ui add <item>... [--dry-run] [--force]
      Copies items (and what they need) into src/ui/. Never overwrites a file you changed or one it did not write,
      unless --force. Stories are copied once the gallery block is installed.
  rune-ui list
      Every registry item, and what this app has (ok, changed, update).
  rune-ui diff [<item>...] [--check]
      Shows how your copies differ from the registry. --check exits 1 if any copy drifted from rune-ui.lock.
  rune-ui update [<item>...] [--dry-run]
      Brings items to the registry's version, merging the registry's changes into your edits (three-way).
  rune-ui registry hash [--registry <dir>]
      For registry maintainers: rewrites the sha256 of every file in item.toml.

  --app <dir>   the app (default: the current directory)";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("rune-ui: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let a = parse_args()?;
    // Rehashing must not need a registry whose hashes already match.
    if a.cmd.first().map(String::as_str) == Some("registry") && a.cmd.get(1).map(String::as_str) == Some("hash") {
        return hash(&a).map(|_| ExitCode::SUCCESS);
    }
    let reg = match &a.registry {
        Some(d) => Registry::from_dir(d)?,
        None => Registry::embedded()?,
    };
    match a.cmd.first().map(String::as_str) {
        Some("init") => init(&a, &reg).map(|_| ExitCode::SUCCESS),
        Some("add") => add(&a, &reg, &a.cmd[1..]).map(|_| ExitCode::SUCCESS),
        Some("list") => list(&a, &reg).map(|_| ExitCode::SUCCESS),
        Some("diff") => diff(&a, &reg, &a.cmd[1..]),
        Some("update") => update(&a, &reg, &a.cmd[1..]),
        _ => {
            println!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn read_lock(app: &Path) -> Result<Lock, String> {
    match std::fs::read_to_string(app.join(LOCK)) {
        Ok(s) => Lock::parse(&s),
        Err(_) => Err(format!("no {LOCK} in {}: run `rune-ui init` first", app.display())),
    }
}

/// Writes `bytes` to `path` (making folders), or says what it would do.
fn write(path: &Path, bytes: &[u8], dry: bool, what: &str) -> Result<(), String> {
    println!("  {}{what} {}", if dry { "would " } else { "" }, path.display());
    if dry {
        return Ok(());
    }
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

// ---- init ----

const PATCHES: [&str; 5] = ["wgpu-hal", "winit", "parley_engine", "accesskit_windows", "accesskit_macos"];

fn source_dep(a: &Args, krate: &str) -> String {
    match (&a.git, &a.source) {
        (Some((url, tag)), _) => format!("{krate} = {{ git = {url:?}, tag = {tag:?} }}"),
        (None, Some(src)) => format!("{krate} = {{ path = {:?} }}", Path::new(src).join("crates").join(krate).display().to_string()),
        (None, None) => format!("{krate} = {{ path = {:?} }}", Path::new(registry::REPO).join("crates").join(krate).display().to_string()),
    }
}

/// Inserts `lines` after the `[section]` header (adding the section at the end if missing), skipping any whose key is already there.
fn ensure_lines(toml: &str, section: &str, lines: &[String]) -> String {
    let header = format!("[{section}]");
    let mut out: Vec<String> = toml.lines().map(str::to_owned).collect();
    let start = out.iter().position(|l| l.trim() == header);
    let (start, end) = match start {
        Some(s) => {
            let e = out.iter().enumerate().skip(s + 1).find(|(_, l)| l.trim_start().starts_with('[')).map_or(out.len(), |(i, _)| i);
            (s, e)
        }
        None => {
            if out.last().is_some_and(|l| !l.trim().is_empty()) {
                out.push(String::new());
            }
            out.push(header.clone());
            (out.len() - 1, out.len())
        }
    };
    let present: BTreeSet<String> = out[start + 1..end].iter().filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_owned())).collect();
    let missing: Vec<String> = lines.iter().filter(|l| !present.contains(l.split_once('=').map_or("", |(k, _)| k.trim()))).cloned().collect();
    let at = start + 1;
    for (k, l) in missing.into_iter().enumerate() {
        out.insert(at + k, l);
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

fn init(a: &Args, reg: &Registry) -> Result<(), String> {
    let manifest = a.app.join("Cargo.toml");
    let toml_src = std::fs::read_to_string(&manifest).map_err(|_| format!("no Cargo.toml in {}: run init in your app's folder", a.app.display()))?;
    let rev = &reg.get("base")?.rune;
    let git = "https://github.com/anomalyco/rune";
    let deps = vec![
        format!("rune = {{ git = {git:?}, rev = {rev:?} }}"),
        format!("rune-kit = {{ git = {git:?}, rev = {rev:?} }}"),
        source_dep(a, "rune-ui-style"),
        source_dep(a, "rune-ui-themes"),
    ];
    // Cargo does not inherit a dependency's patches: Rune's table goes into the app.
    let patches: Vec<String> = PATCHES.iter().map(|p| format!("{p} = {{ git = {git:?}, rev = {rev:?} }}")).collect();
    let next = ensure_lines(&ensure_lines(&toml_src, "dependencies", &deps), "patch.crates-io", &patches);
    println!("rune-ui init in {}", a.app.display());
    if next != toml_src {
        write(&manifest, next.as_bytes(), a.dry_run, "update")?;
    }
    if !a.dry_run && !a.app.join(LOCK).exists() {
        std::fs::write(a.app.join(LOCK), Lock { rune: rev.clone(), ..Lock::default() }.render()).map_err(|e| e.to_string())?;
    }
    let lock = if a.dry_run { Lock { rune: rev.clone(), ..Lock::default() } } else { read_lock(&a.app)? };
    copy(a, reg, lock, &["base".to_owned(), "icons".to_owned()])?;
    let main_rs = ["src/main.rs", "src/lib.rs"].iter().map(|p| a.app.join(p)).find(|p| p.exists());
    if let Some(m) = main_rs
        && !std::fs::read_to_string(&m).unwrap_or_default().contains("mod ui;")
    {
        println!("next: add `mod ui;` to {}", m.display());
    }
    Ok(())
}

// ---- add ----

fn add(a: &Args, reg: &Registry, names: &[String]) -> Result<(), String> {
    if names.is_empty() {
        return Err("add what? (rune-ui list)".to_owned());
    }
    let lock = read_lock(&a.app)?;
    println!("rune-ui add {}", names.join(" "));
    copy(a, reg, lock, names)
}

/// Copies items (and their needs) into the app, refusing to overwrite what it did not write or what you changed (unless `--force`),
/// then regenerates `src/ui/mod.rs` and `src/ui/stories/mod.rs` and writes the lock.
fn copy(a: &Args, reg: &Registry, mut lock: Lock, names: &[String]) -> Result<(), String> {
    let all = reg.closure(names)?;
    let gallery = lock.items.contains_key("gallery") || all.iter().any(|n| n == "gallery");
    // A story is copied with the gallery, and only once what it shows besides its own item is installed too.
    let installed: BTreeSet<String> = lock.items.keys().cloned().chain(all.iter().cloned()).collect();
    let story_ok = |item: &Item| gallery && item.story_needs.iter().all(|n| installed.contains(n));
    // With the gallery, revisit what was already installed: its stories may now be copyable.
    let mut todo: Vec<String> = all.clone();
    if gallery {
        todo.extend(lock.items.keys().filter(|k| !all.contains(k)).cloned());
    }
    let mut writes: Vec<(String, &Item, PathBuf, Vec<u8>)> = Vec::new();
    let mut seen_targets: Vec<(String, String)> = Vec::new();
    for name in &todo {
        let item = reg.get(name)?;
        for f in &item.files {
            if f.story && !story_ok(item) {
                continue;
            }
            if let Some((_, other)) = seen_targets.iter().find(|(t, _)| *t == f.target) {
                return Err(format!("{name} and {other} both write {}", f.target));
            }
            seen_targets.push((f.target.clone(), name.clone()));
            let bytes = item.contents.get(&f.path).ok_or_else(|| format!("{name}: {} is not in the registry", f.path))?.clone();
            let path = a.app.join(&f.target);
            if let Ok(existing) = std::fs::read(&path) {
                let locked = lock.items.get(name.as_str()).and_then(|it| it.files.iter().find(|lf| lf.target == f.target));
                let owner = lock.owner(&f.target);
                if owner.is_some_and(|o| o != name) && !a.force {
                    return Err(format!("{} belongs to {} (use --force to take it)", f.target, owner.unwrap_or_default()));
                }
                match locked {
                    None if !a.force => return Err(format!("{} exists and rune-ui did not write it (use --force to replace it)", f.target)),
                    Some(lf) if lf.sha256 != sha256(&existing) && !a.force => return Err(format!("{} has your changes (use `rune-ui update {name}` to merge, or --force)", f.target)),
                    _ => {}
                }
                if existing == bytes && locked.is_some() {
                    continue;
                }
            }
            writes.push((name.clone(), item, path, bytes));
        }
    }
    for (_, _, path, bytes) in &writes {
        write(path, bytes, a.dry_run, "write")?;
    }
    for name in &todo {
        let item = reg.get(name)?;
        let files: Vec<LockedFile> = item.files.iter().filter(|f| !f.story || story_ok(item)).map(|f| LockedFile { target: f.target.clone(), sha256: sha256(&item.contents[&f.path]) }).collect();
        if !a.dry_run {
            for f in item.files.iter().filter(|f| !f.story || story_ok(item)) {
                let base = a.app.join(BASE).join(&f.target);
                std::fs::create_dir_all(base.parent().unwrap()).map_err(|e| e.to_string())?;
                std::fs::write(base, &item.contents[&f.path]).map_err(|e| e.to_string())?;
            }
        }
        lock.items.insert(name.clone(), LockedItem { version: item.version.clone(), files });
    }
    write_modules(a, &lock)?;
    if !a.dry_run {
        std::fs::write(a.app.join(LOCK), lock.render()).map_err(|e| e.to_string())?;
    }
    let cargo: BTreeSet<&String> = todo.iter().filter_map(|n| reg.items.get(n)).flat_map(|i| i.cargo.iter()).collect();
    println!("{} {} file(s); needs crates: {}", if a.dry_run { "would write" } else { "wrote" }, writes.len(), cargo.into_iter().cloned().collect::<Vec<_>>().join(", "));
    Ok(())
}

/// `src/ui/mod.rs` and `src/ui/stories/mod.rs`, from the lock: every installed item, and (with the gallery) every story.
fn write_modules(a: &Args, lock: &Lock) -> Result<(), String> {
    let mut m = String::from("//! Components copied by `rune-ui` (yours to change). This file is regenerated by `rune-ui add`: add your own modules elsewhere.\n\n// A kit is a library inside the app: the parts this app does not use yet are not mistakes.\n#![allow(dead_code, unused_imports)]\n\n");
    for name in lock.items.keys() {
        m.push_str(&format!("pub mod {name};\n"));
    }
    let gallery = lock.items.contains_key("gallery");
    if gallery {
        m.push_str("pub mod stories;\n");
    }
    write_if_changed(&a.app.join("src/ui/mod.rs"), &m, a.dry_run)?;
    if gallery {
        let with: Vec<&String> = lock.items.iter().filter(|(_, it)| it.files.iter().any(|f| f.target.starts_with("src/ui/stories/"))).map(|(n, _)| n).collect();
        let mut s = String::from("//! Every installed component's stories, for the gallery page. Regenerated by `rune-ui add`.\n\n");
        for n in &with {
            s.push_str(&format!("pub mod {n};\n"));
        }
        s.push_str("\n/// Every installed component's stories (the gallery page lists these).\npub fn all<S: 'static>() -> Vec<super::gallery::StoryItem<S>> {\n    vec![");
        s.push_str(&with.iter().map(|n| format!("{n}::item()")).collect::<Vec<_>>().join(", "));
        s.push_str("]\n}\n");
        write_if_changed(&a.app.join("src/ui/stories/mod.rs"), &s, a.dry_run)?;
    }
    Ok(())
}

fn write_if_changed(path: &Path, s: &str, dry: bool) -> Result<(), String> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(s) {
        return Ok(());
    }
    write(path, s.as_bytes(), dry, "write")
}

// ---- list ----

fn status(a: &Args, item: &Item, lock: &Lock) -> String {
    let Some(li) = lock.items.get(&item.name) else { return String::new() };
    let changed = li.files.iter().any(|f| std::fs::read(a.app.join(&f.target)).ok().map(|b| sha256(&b)) != Some(f.sha256.clone()));
    let behind = li.version != item.version;
    match (changed, behind) {
        (false, false) => format!("{} ok", li.version),
        (true, false) => format!("{} changed", li.version),
        (false, true) => format!("{} update to {}", li.version, item.version),
        (true, true) => format!("{} changed, update to {}", li.version, item.version),
    }
}

fn list(a: &Args, reg: &Registry) -> Result<(), String> {
    let lock = read_lock(&a.app).unwrap_or_default();
    println!("{:<16} {:<16} {:<8} {:<10} {:<10} this app", "item", "title", "version", "type", "group");
    for item in reg.items.values() {
        println!("{:<16} {:<16} {:<8} {:<10} {:<10} {}", item.name, item.title, item.version, item.kind, item.group, status(a, item, &lock));
    }
    Ok(())
}

// ---- diff ----

fn chosen<'a>(lock: &'a Lock, names: &[String]) -> Result<Vec<&'a String>, String> {
    if names.is_empty() {
        return Ok(lock.items.keys().collect());
    }
    names.iter().map(|n| lock.items.get_key_value(n).map(|(k, _)| k).ok_or_else(|| format!("`{n}` is not installed"))).collect()
}

fn diff(a: &Args, reg: &Registry, names: &[String]) -> Result<ExitCode, String> {
    let lock = read_lock(&a.app)?;
    let mut drifted = 0;
    for name in chosen(&lock, names)? {
        let li = &lock.items[name];
        let item = reg.items.get(name);
        for f in &li.files {
            let path = a.app.join(&f.target);
            let local = std::fs::read(&path).unwrap_or_default();
            if sha256(&local) != f.sha256 {
                drifted += 1;
                println!("{name}: {} differs from what rune-ui wrote", f.target);
                if !a.check
                    && let Some(src) = item.and_then(|i| i.files.iter().find(|x| x.target == f.target)).and_then(|x| item?.contents.get(&x.path))
                {
                    print!("{}", merge::unified(&String::from_utf8_lossy(src), &String::from_utf8_lossy(&local), &format!("registry {}", f.target), &format!("yours {}", f.target), 2));
                }
            }
        }
        if let Some(i) = item
            && i.version != li.version
        {
            println!("{name}: {} installed, {} in the registry (rune-ui update {name})", li.version, i.version);
        }
    }
    if a.check && drifted > 0 {
        println!("{drifted} file(s) drifted from {LOCK}");
        return Ok(ExitCode::from(1));
    }
    if drifted == 0 {
        println!("no drift");
    }
    Ok(ExitCode::SUCCESS)
}

// ---- update ----

fn update(a: &Args, reg: &Registry, names: &[String]) -> Result<ExitCode, String> {
    let mut lock = read_lock(&a.app)?;
    let mut conflicts = 0;
    for name in chosen(&lock, names)?.into_iter().cloned().collect::<Vec<_>>() {
        let Ok(item) = reg.get(&name) else { continue };
        let mut files = Vec::new();
        for f in item.files.iter().filter(|f| lock.items[&name].files.iter().any(|lf| lf.target == f.target)) {
            let theirs = String::from_utf8_lossy(&item.contents[&f.path]).into_owned();
            let path = a.app.join(&f.target);
            let ours = std::fs::read_to_string(&path).unwrap_or_default();
            let base_path = a.app.join(BASE).join(&f.target);
            let base = std::fs::read_to_string(&base_path).unwrap_or_else(|_| ours.clone());
            let (merged, conflict) = merge::three_way(&base, &ours, &theirs);
            if conflict {
                conflicts += 1;
                println!("{name}: {} has conflicts (marked <<<<<<< yours / >>>>>>> rune-ui)", f.target);
            }
            if merged != ours {
                write(&path, merged.as_bytes(), a.dry_run, "merge into")?;
            }
            if !a.dry_run {
                std::fs::write(&base_path, &theirs).map_err(|e| e.to_string())?;
            }
            files.push(LockedFile { target: f.target.clone(), sha256: sha256(merged.as_bytes()) });
        }
        lock.items.insert(name.clone(), LockedItem { version: item.version.clone(), files });
    }
    if !a.dry_run {
        std::fs::write(a.app.join(LOCK), lock.render()).map_err(|e| e.to_string())?;
    }
    Ok(if conflicts > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS })
}

// ---- registry hash ----

fn hash(a: &Args) -> Result<(), String> {
    let root = a.registry.clone().unwrap_or_else(|| Path::new(registry::REPO).join("registry"));
    let mut changed = 0;
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root).map_err(|e| format!("{}: {e}", root.display()))?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.join("item.toml").exists()).collect();
    dirs.sort();
    for d in dirs {
        let manifest = d.join("item.toml");
        let src = std::fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
        let mut out = String::new();
        let mut path: Option<String> = None;
        for line in src.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == "path" {
                    path = Some(v.trim().trim_matches('"').to_owned());
                }
                if k.trim() == "sha256"
                    && let Some(p) = &path
                {
                    let bytes = std::fs::read(d.join(p)).map_err(|e| format!("{}: {e}", d.join(p).display()))?;
                    out.push_str(&format!("sha256 = \"{}\"\n", sha256(&bytes)));
                    continue;
                }
            }
            out.push_str(line);
            out.push('\n');
        }
        if out != src {
            changed += 1;
            std::fs::write(&manifest, out).map_err(|e| e.to_string())?;
            println!("  rehashed {}", manifest.display());
        }
    }
    println!("{changed} item(s) rehashed");
    Ok(())
}
