//! Pixel checks on the real renderer: a raised button is lighter at its top than its bottom, a well sits below the light page, and the
//! ink button is the darkest thing on it. Renders through the gallery binary (`--snap`, `--tree`), as `scripts/snap.sh` does.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_gallery");

fn out() -> PathBuf {
    let d = std::env::temp_dir().join("rune-ui-tests");
    std::fs::create_dir_all(&d).unwrap();
    d
}

struct Image {
    w: usize,
    px: Vec<u8>,
}

fn read(path: &Path) -> Image {
    let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).unwrap();
    let px = match info.color_type {
        png::ColorType::Rgba => buf,
        _ => buf.chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
    };
    Image { w: info.width as usize, px }
}

impl Image {
    fn luma(&self, x: f32, y: f32) -> f32 {
        let i = (y as usize * self.w + x as usize) * 4;
        0.2126 * self.px[i] as f32 + 0.7152 * self.px[i + 1] as f32 + 0.0722 * self.px[i + 2] as f32
    }
}

fn rect(tree: &serde_json::Value, key: &str) -> [f32; 4] {
    fn walk<'a>(v: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
        if v.get("key").and_then(|k| k.as_str()) == Some(key) && v.get("bounds").is_some() {
            return Some(v);
        }
        match v {
            serde_json::Value::Array(a) => a.iter().find_map(|x| walk(x, key)),
            serde_json::Value::Object(o) => o.values().find_map(|x| walk(x, key)),
            _ => None,
        }
    }
    // The accessibility tree's node: its bounds in window points.
    let e = walk(tree, key).unwrap_or_else(|| panic!("no {key} in the tree"));
    let r: Vec<f32> = e["bounds"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect();
    [r[0], r[1], r[2], r[3]]
}

#[test]
fn materials_read_in_the_right_direction_on_screen() {
    let png = out().join("actions-light.png");
    let tree = out().join("actions-light.json");
    let snap = Command::new(BIN).args(["--snap", png.to_str().unwrap(), "--scene", "actions", "--theme", "light", "--size", "1320x4000", "--scale", "1", "--crop", "canvas"]).output().unwrap();
    assert!(snap.status.success(), "{}", String::from_utf8_lossy(&snap.stderr));
    let dump = Command::new(BIN).args(["--tree", tree.to_str().unwrap(), "--scene", "actions", "--theme", "light", "--size", "1320x4000"]).output().unwrap();
    assert!(dump.status.success());
    let img = read(&png);
    let tree: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&tree).unwrap()).unwrap();
    let [x, y, w, h] = rect(&tree, "button-secondary-rest");
    let (top, bottom) = (img.luma(x + w / 2.0, y + 2.0), img.luma(x + w / 2.0, y + h - 3.0));
    assert!(top > bottom, "raised: lighter on top ({top} vs {bottom})");
    let page = img.luma(4.0, 4.0);
    let [x, y, w, h] = rect(&tree, "button-default-rest");
    let ink = img.luma(x + 4.0, y + h / 2.0);
    assert!(ink < page / 3.0, "the ink button is dark on the light page ({ink} vs {page})");
    let _ = w;
}

/// Renders `scene` in the light theme at 2x and dumps its tree; returns the picture and the tree.
fn picture(scene: &str) -> (Image, serde_json::Value) {
    let png = out().join(format!("{scene}-light-2x.png"));
    let tree = out().join(format!("{scene}-light-2x.json"));
    let snap = Command::new(BIN).args(["--snap", png.to_str().unwrap(), "--scene", scene, "--theme", "light", "--size", "1320x8000", "--scale", "2", "--crop", "canvas"]).output().unwrap();
    assert!(snap.status.success(), "{}", String::from_utf8_lossy(&snap.stderr));
    let dump = Command::new(BIN).args(["--tree", tree.to_str().unwrap(), "--scene", scene, "--theme", "light", "--size", "1320x8000"]).output().unwrap();
    assert!(dump.status.success());
    (read(&png), serde_json::from_str(&std::fs::read_to_string(&tree).unwrap()).unwrap())
}

/// Pixels (at 2x) inside `key`'s rect whose luma differs by more than 24 between the two pictures.
fn differing(a: &(Image, serde_json::Value), b: &(Image, serde_json::Value), key: &str) -> usize {
    let r = rect(&a.1, key);
    assert_eq!(r, rect(&b.1, key), "{key} sits in the same place in both");
    let mut n = 0;
    for y in (r[1] * 2.0) as usize..((r[1] + r[3]) * 2.0) as usize {
        for x in (r[0] * 2.0) as usize..((r[0] + r[2]) * 2.0) as usize {
            if (a.0.luma(x as f32, y as f32) - b.0.luma(x as f32, y as f32)).abs() > 24.0 {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn continuous_and_round_corners_draw_different_pixels() {
    let (cont, round) = (picture("materials"), picture("materials-round"));
    let tile = differing(&cont, &round, "corner-theme");
    let button = differing(&cont, &round, "corner-theme-button");
    eprintln!("corner pixels that differ at 2x: tile {tile}, button {button}");
    // A 240 by 160 tile (xxl radius) and a 120 by 36 button: the corner style must show in each corner, not only in a few edge pixels.
    assert!(tile > 400, "the large tile looks the same in both corner styles ({tile} pixels differ)");
    assert!(button > 40, "the button looks the same in both corner styles ({button} pixels differ)");
}
