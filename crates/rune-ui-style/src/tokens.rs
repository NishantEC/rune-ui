//! Theme files: three tiers of tokens in one JSON file with a light and a dark mode, shaped like W3C design tokens (`$type`, `$value`,
//! `{alias}`). The parser is strict: every key must be known and every required key present, and it reports every problem it finds
//! with its path, so a typo is an error and never a silently ignored value. `crates/rune-ui-themes/theme.schema.json` describes the
//! same shape for editors; a test keeps the two in step.
//!
//! * **Tier 1, ramps** (`color`): raw `#rrggbb` colours by ramp and step. Only modes read them.
//! * **Tier 2, roles** (`modes.<mode>.role`): every surface role has a `bg` and the `fg` that reads on it; meanings add a `solid`
//!   colour and its `on-solid` text; `line` and `ring` stand alone. Values alias a ramp step, optionally at an alpha:
//!   `"{neutral.900}"`, `"{neutral.1000} / 0.05"`.
//! * **State alphas** (`modes.<mode>.state`): hover, pressed and selected are the page text colour at these alphas; disabled is an opacity.
//! * **Tier 3, component tokens** (`modes.<mode>.component`): the few values a material needs that no role names.
//! * **Base knobs** (`base`): `spacing`, `radius`, `density`.

use std::collections::BTreeMap;

use rune::Color;

pub use crate::fonts::Font;
use serde_json::{Map, Value};


pub const RAMPS: [&str; 5] = ["neutral", "blue", "green", "amber", "red"];
pub const NEUTRAL_STEPS: [&str; 19] = ["0", "25", "50", "100", "150", "200", "300", "400", "500", "600", "700", "750", "800", "850", "900", "925", "950", "975", "1000"];
pub const HUE_STEPS: [&str; 11] = ["50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950"];
pub const SURFACE_ROLES: [&str; 9] = ["page", "surface", "floating", "muted", "subtle", "primary", "secondary", "accent", "input"];
pub const TONE_ROLES: [&str; 4] = ["info", "success", "warning", "danger"];
pub const LINE_ROLES: [&str; 2] = ["line", "ring"];
pub const STATE_KEYS: [&str; 4] = ["hover", "pressed", "selected", "disabled"];
pub const COMPONENT_KEYS: [&str; 11] = ["raised-top", "raised-bottom", "raised-edge", "ink-top", "ink-bottom", "ink-rim", "ink-edge", "well-shadow", "thumb", "float-ring", "shade"];
pub const NUMBER_KEYS: [&str; 4] = ["elevation", "well-blur", "grain", "texture"];
/// The numeric base knobs.
pub const BASE_KEYS: [&str; 4] = ["spacing", "radius", "density", "type-scale"];
/// The base knobs that are words, and the words each allows.
pub const BASE_WORDS: [(&str, &[&str]); 1] = [("corners", &["continuous", "round"])];
/// The base knobs that name something registered at run time: `font` (a built-in or an installed font item, [`crate::fonts`]).
pub const BASE_NAMES: [&str; 1] = ["font"];

/// A surface and the text that reads on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Role {
    pub bg: Color,
    pub fg: Color,
}

/// A meaning: its wash and the words on it, a solid colour and the words on that.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub bg: Color,
    pub fg: Color,
    pub solid: Color,
    pub on_solid: Color,
}

/// One mode (light or dark), resolved to colours.
#[derive(Clone, Debug, PartialEq)]
pub struct Mode {
    pub roles: BTreeMap<String, Role>,
    pub tones: BTreeMap<String, Tone>,
    pub line: Color,
    pub ring: Color,
    pub state: BTreeMap<String, f32>,
    pub component: BTreeMap<String, Color>,
    pub number: BTreeMap<String, f32>,
}

impl Mode {
    pub fn role(&self, k: &str) -> Role {
        self.roles[k]
    }
    pub fn tone(&self, k: &str) -> Tone {
        self.tones[k]
    }
    pub fn component(&self, k: &str) -> Color {
        self.component[k]
    }
    pub fn state(&self, k: &str) -> f32 {
        self.state[k]
    }
    pub fn number(&self, k: &str) -> f32 {
        self.number[k]
    }
}

/// How rounded corners are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Corners {
    /// Apple's continuous curve (a squircle): it eases into the straight edges.
    #[default]
    Continuous,
    /// A plain circular arc.
    Round,
}

/// How much larger the radius ladder is drawn with continuous corners than with round ones.
///
/// Rune draws Apple's continuous curve (`El::continuous_corners`): at the same radius it leaves each edge 1.53 times further from the
/// corner but meets the diagonal at almost the same point, so it never strays more than 0.4 points from a circular arc on a 240 by 160
/// card with a 28 point radius (about 0.1 point on a button). The two styles could not be told apart. Continuous corners are drawn a
/// quarter larger instead, as designers pair them with iOS: the curve now eases in over 1.9 times the nominal radius and its apex sits
/// further in, 2.7 points from the circular arc on that card and 0.8 on a button. Concentric nesting holds because every step of the
/// ladder grows by the same factor.
pub const CONTINUOUS_GAIN: f32 = 1.25;

impl Corners {
    /// What the radius ladder is multiplied by in this style ([`CONTINUOUS_GAIN`] or 1).
    pub const fn gain(self) -> f32 {
        match self {
            Corners::Continuous => CONTINUOUS_GAIN,
            Corners::Round => 1.0,
        }
    }
}

/// The base knobs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Base {
    /// The grid unit, in points (4: `space.at(3)` is 12).
    pub spacing: f32,
    /// The middle of the radius ladder (a control's corner), in points.
    pub radius: f32,
    /// Control heights, paddings, gaps, rows and the type ramp: 1.0 comfortable, 0.8 compact.
    pub density: f32,
    /// Multiplies every type size (1.0 as designed).
    pub type_scale: f32,
    pub corners: Corners,
    pub font: Font,
}

/// A whole theme file, parsed and checked.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeFile {
    pub name: String,
    pub base: Base,
    pub ramps: BTreeMap<String, BTreeMap<String, Color>>,
    pub light: Mode,
    pub dark: Mode,
}

impl ThemeFile {
    pub fn mode(&self, dark: bool) -> &Mode {
        if dark { &self.dark } else { &self.light }
    }

    /// Parses and checks a theme file. Every problem is reported, each with its path.
    pub fn parse(json: &str) -> Result<ThemeFile, Vec<String>> {
        let root: Value = serde_json::from_str(json).map_err(|e| vec![format!("not JSON: {e}")])?;
        let mut p = Problems::default();
        let Some(root) = p.object(&root, "") else { return Err(p.0) };
        p.keys(root, "", &["name", "base", "color", "modes"], &["$schema", "description"]);
        let name = root.get("name").and_then(Value::as_str).unwrap_or_default().to_owned();
        if name.is_empty() {
            p.push("name", "a non-empty string is required");
        }
        let base = root.get("base").map(|b| p.base(b));
        let mut ramps = BTreeMap::new();
        if let Some(color) = root.get("color").and_then(|c| p.object(c, "color")) {
            p.keys(color, "color", &RAMPS, &["$type"]);
            p.type_tag(color, "color", "color");
            for ramp in RAMPS {
                let path = format!("color.{ramp}");
                let Some(steps) = color.get(ramp).and_then(|r| p.object(r, &path)) else { continue };
                let wanted: &[&str] = if ramp == "neutral" { &NEUTRAL_STEPS } else { &HUE_STEPS };
                p.keys(steps, &path, wanted, &[]);
                let mut out = BTreeMap::new();
                for step in wanted {
                    let at = format!("{path}.{step}");
                    if let Some(s) = steps.get(*step).and_then(|v| p.leaf(v, &at)).and_then(Value::as_str) {
                        match rune::color::hex(s).filter(|_| s.len() == 7) {
                            Some(c) => {
                                out.insert((*step).to_owned(), c);
                            }
                            None => p.push(&at, "must be #rrggbb"),
                        }
                    }
                }
                ramps.insert(ramp.to_owned(), out);
            }
        }
        let mut modes = Vec::new();
        if let Some(m) = root.get("modes").and_then(|m| p.object(m, "modes")) {
            p.keys(m, "modes", &["light", "dark"], &[]);
            for which in ["light", "dark"] {
                modes.push(m.get(which).map(|v| p.mode(v, &format!("modes.{which}"), &ramps)));
            }
        }
        if !p.0.is_empty() {
            return Err(p.0);
        }
        let (Some(base), Some(Some(light)), Some(Some(dark))) = (base, modes.first().cloned(), modes.get(1).cloned()) else { return Err(vec!["incomplete theme".to_owned()]) };
        Ok(ThemeFile { name, base, ramps, light, dark })
    }
}

#[derive(Default)]
struct Problems(Vec<String>);

impl Problems {
    fn push(&mut self, path: &str, what: &str) {
        self.0.push(format!("{}: {what}", if path.is_empty() { "(root)" } else { path }));
    }

    fn object<'a>(&mut self, v: &'a Value, path: &str) -> Option<&'a Map<String, Value>> {
        let o = v.as_object();
        if o.is_none() {
            self.push(path, "must be an object");
        }
        o
    }

    /// Every `required` key present; nothing but `required` and `optional` keys.
    fn keys(&mut self, o: &Map<String, Value>, path: &str, required: &[&str], optional: &[&str]) {
        for k in required {
            if !o.contains_key(*k) {
                self.push(&join(path, k), "missing");
            }
        }
        for k in o.keys() {
            if !required.contains(&k.as_str()) && !optional.contains(&k.as_str()) {
                self.push(&join(path, k), "unknown key");
            }
        }
    }

    fn type_tag(&mut self, o: &Map<String, Value>, path: &str, want: &str) {
        if let Some(t) = o.get("$type")
            && t.as_str() != Some(want)
        {
            self.push(&join(path, "$type"), &format!("must be \"{want}\""));
        }
    }

    /// A token: `{"$value": …}` (and an optional `$description`).
    fn leaf<'a>(&mut self, v: &'a Value, path: &str) -> Option<&'a Value> {
        let o = self.object(v, path)?;
        self.keys(o, path, &["$value"], &["$description", "$type"]);
        o.get("$value")
    }

    fn numbers(&mut self, v: &Value, path: &str, keys: &[&str]) -> BTreeMap<String, f32> {
        let mut out = BTreeMap::new();
        let Some(o) = self.object(v, path) else { return out };
        self.keys(o, path, keys, &["$type"]);
        self.type_tag(o, path, "number");
        for k in keys {
            let at = join(path, k);
            if let Some(val) = o.get(*k).and_then(|x| self.leaf(x, &at)) {
                match val.as_f64() {
                    Some(n) => {
                        out.insert((*k).to_owned(), n as f32);
                    }
                    None => self.push(&at, "must be a number"),
                }
            }
        }
        out
    }

    /// The base knobs: numbers, and words from a fixed list.
    fn base(&mut self, v: &Value) -> Base {
        let mut b = Base { spacing: 4.0, radius: 8.0, density: 1.0, type_scale: 1.0, corners: Corners::Continuous, font: Font::INTER };
        let Some(o) = self.object(v, "base") else { return b };
        let all: Vec<&str> = BASE_KEYS.iter().copied().chain(BASE_WORDS.iter().map(|(k, _)| *k)).chain(BASE_NAMES).collect();
        self.keys(o, "base", &all, &[]);
        for k in BASE_KEYS {
            let at = join("base", k);
            if let Some(val) = o.get(k).and_then(|x| self.leaf(x, &at)) {
                match val.as_f64() {
                    Some(n) => match k {
                        "spacing" => b.spacing = n as f32,
                        "radius" => b.radius = n as f32,
                        "density" => b.density = n as f32,
                        _ => b.type_scale = n as f32,
                    },
                    None => self.push(&at, "must be a number"),
                }
            }
        }
        for (k, allowed) in BASE_WORDS {
            let at = join("base", k);
            if let Some(val) = o.get(k).and_then(|x| self.leaf(x, &at)) {
                match val.as_str().filter(|w| allowed.contains(w)) {
                    Some(w) => match w {
                        "continuous" => b.corners = Corners::Continuous,
                        _ => b.corners = Corners::Round,
                    },
                    None => self.push(&at, &format!("must be one of {}", allowed.join(", "))),
                }
            }
        }
        let at = join("base", "font");
        if let Some(val) = o.get("font").and_then(|x| self.leaf(x, &at)) {
            match val.as_str() {
                Some(name) => match crate::fonts::find(name) {
                    Some(f) => b.font = f,
                    None => self.push(&at, &format!("no font called \"{name}\"; this app has {} (a font item adds one: `rune-ui add font_<name>`, then `rune_ui_style::fonts::install(ui::FONTS)` at start-up)", crate::fonts::names().join(", "))),
                },
                None => self.push(&at, "must be a font's name, like \"inter\""),
            }
        }
        b
    }

    /// `{ramp.step}` or `{ramp.step} / alpha`.
    fn alias(&mut self, v: &Value, path: &str, ramps: &BTreeMap<String, BTreeMap<String, Color>>) -> Color {
        let Some(s) = self.leaf(v, path).and_then(Value::as_str) else {
            self.push(path, "must be an alias string like \"{neutral.900}\" or \"{neutral.1000} / 0.05\"");
            return Color::TRANSPARENT;
        };
        let (name, alpha) = match s.split_once(" / ") {
            Some((n, a)) => (n.trim(), a.trim().parse::<f32>().ok().filter(|a| (0.0..=1.0).contains(a))),
            None => (s.trim(), Some(1.0)),
        };
        let Some(alpha) = alpha else {
            self.push(path, "the alpha after \" / \" must be a number from 0 to 1");
            return Color::TRANSPARENT;
        };
        let target = name.strip_prefix('{').and_then(|n| n.strip_suffix('}')).and_then(|n| n.split_once('.'));
        match target.and_then(|(r, st)| ramps.get(r).and_then(|steps| steps.get(st))) {
            Some(c) => c.alpha(alpha),
            None => {
                self.push(path, &format!("{s:?} names no ramp step"));
                Color::TRANSPARENT
            }
        }
    }

    fn mode(&mut self, v: &Value, path: &str, ramps: &BTreeMap<String, BTreeMap<String, Color>>) -> Mode {
        let mut m = Mode { roles: BTreeMap::new(), tones: BTreeMap::new(), line: Color::TRANSPARENT, ring: Color::TRANSPARENT, state: BTreeMap::new(), component: BTreeMap::new(), number: BTreeMap::new() };
        let Some(o) = self.object(v, path) else { return m };
        self.keys(o, path, &["role", "state", "component", "number"], &[]);
        if let Some(roles) = o.get("role").and_then(|r| self.object(r, &join(path, "role"))) {
            let rp = join(path, "role");
            let all: Vec<&str> = SURFACE_ROLES.iter().chain(TONE_ROLES.iter()).chain(LINE_ROLES.iter()).copied().collect();
            self.keys(roles, &rp, &all, &["$type"]);
            self.type_tag(roles, &rp, "color");
            for k in SURFACE_ROLES {
                let at = join(&rp, k);
                if let Some(r) = roles.get(k).and_then(|r| self.object(r, &at)) {
                    self.keys(r, &at, &["bg", "fg"], &[]);
                    let bg = r.get("bg").map_or(Color::TRANSPARENT, |x| self.alias(x, &join(&at, "bg"), ramps));
                    let fg = r.get("fg").map_or(Color::TRANSPARENT, |x| self.alias(x, &join(&at, "fg"), ramps));
                    m.roles.insert(k.to_owned(), Role { bg, fg });
                }
            }
            for k in TONE_ROLES {
                let at = join(&rp, k);
                if let Some(r) = roles.get(k).and_then(|r| self.object(r, &at)) {
                    self.keys(r, &at, &["bg", "fg", "solid", "on-solid"], &[]);
                    let mut get = |key: &str| r.get(key).map_or(Color::TRANSPARENT, |x| self.alias(x, &join(&at, key), ramps));
                    let tone = Tone { bg: get("bg"), fg: get("fg"), solid: get("solid"), on_solid: get("on-solid") };
                    m.tones.insert(k.to_owned(), tone);
                }
            }
            m.line = roles.get("line").map_or(Color::TRANSPARENT, |x| self.alias(x, &join(&rp, "line"), ramps));
            m.ring = roles.get("ring").map_or(Color::TRANSPARENT, |x| self.alias(x, &join(&rp, "ring"), ramps));
        }
        if let Some(s) = o.get("state") {
            m.state = self.numbers(s, &join(path, "state"), &STATE_KEYS);
        }
        if let Some(c) = o.get("component").and_then(|c| self.object(c, &join(path, "component"))) {
            let cp = join(path, "component");
            self.keys(c, &cp, &COMPONENT_KEYS, &["$type"]);
            self.type_tag(c, &cp, "color");
            for k in COMPONENT_KEYS {
                if let Some(x) = c.get(k) {
                    let col = self.alias(x, &join(&cp, k), ramps);
                    m.component.insert(k.to_owned(), col);
                }
            }
        }
        if let Some(n) = o.get("number") {
            m.number = self.numbers(n, &join(path, "number"), &NUMBER_KEYS);
        }
        m
    }
}

fn join(path: &str, k: &str) -> String {
    if path.is_empty() { k.to_owned() } else { format!("{path}.{k}") }
}
