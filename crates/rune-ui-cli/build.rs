//! Embeds the registry (`../../registry/*`) in the binary, so `rune-ui` works from anywhere with the components it was built with.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../registry").canonicalize().unwrap();
    println!("cargo::rerun-if-changed={}", root.display());
    let mut out = String::from("/// (item folder, its files: (name, bytes)).\npub static EMBEDDED: &[(&str, &[(&str, &[u8])])] = &[\n");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root).unwrap().map(|e| e.unwrap().path()).filter(|p| p.join("item.toml").exists()).collect();
    dirs.sort();
    for d in dirs {
        println!("cargo::rerun-if-changed={}", d.display());
        let mut files: Vec<PathBuf> = std::fs::read_dir(&d).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_file()).collect();
        files.sort();
        write!(out, "    ({:?}, &[", d.file_name().unwrap().to_string_lossy()).unwrap();
        for f in files {
            println!("cargo::rerun-if-changed={}", f.display());
            write!(out, "({:?}, include_bytes!({:?})), ", f.file_name().unwrap().to_string_lossy(), f.display().to_string()).unwrap();
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    writeln!(out, "/// The repository the binary was built from (the default `--source` for `init`).\npub const REPO: &str = {:?};", root.parent().unwrap().display().to_string()).unwrap();
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded.rs"), out).unwrap();
}
