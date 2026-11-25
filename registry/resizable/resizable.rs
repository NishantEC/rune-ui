//! rune-ui registry: resizable 0.5.1. Copied into your app by `rune-ui add resizable`; yours to change.
//!
//! Panes side by side (or stacked) split by handles: a hairline with a 9 point grab area, an optional grip, the resize cursor. Drag a
//! handle, or Tab to it and use the arrow keys (5%, Shift for 20%) and Home/End (the pane's least and most). Each pane keeps inside its
//! least and most share. The app keeps the shares (they add up to 1) and hears every change.
//!
//! A handle turns its travel into a share with the group's length, which the group records as it lays out.
//!
//! ```ignore
//! Resizable::new("editor", "Editor panes").pane(Pane::new(files).min(0.15)).pane(Pane::new(code)).sizes(&s.split)
//!     .on_resize(|s: &mut App, v: &[f32]| s.split = v.to_vec()).build(t)
//! ```

use std::rc::Rc;

use rune::{Cursor, Drag, El, Key, KeyEvent, Phase, Tx, col, el, row};

use super::base::{Theme, paint};

thread_local! {
    static LENGTHS: std::cell::RefCell<std::collections::HashMap<String, f32>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// A handle's thickness: its grab area. The hairline is drawn in its middle.
pub const HANDLE: f32 = 9.0;

pub struct Pane {
    content: El,
    name: String,
    min: f32,
    max: f32,
}

impl Pane {
    pub fn new(content: El) -> Pane {
        Pane { content, name: String::new(), min: 0.1, max: 0.9 }
    }

    /// What the handle after it says it resizes.
    pub fn name(mut self, n: impl Into<String>) -> Self {
        self.name = n.into();
        self
    }

    /// The least share (0 to 1).
    pub fn min(mut self, f: f32) -> Self {
        self.min = f;
        self
    }

    /// The most share (0 to 1).
    pub fn max(mut self, f: f32) -> Self {
        self.max = f;
        self
    }
}

pub type OnResize<S> = Rc<dyn Fn(&mut S, &[f32])>;

pub struct Resizable<S: 'static> {
    key: String,
    name: String,
    panes: Vec<Pane>,
    sizes: Vec<f32>,
    vertical: bool,
    grip: bool,
    on_resize: Option<OnResize<S>>,
}

/// `sizes` with the edge between panes `i` and `i + 1` moved to `at` (a share of the whole from the start), each pane kept inside its
/// bounds and the pair's total kept.
pub fn moved(sizes: &[f32], bounds: &[(f32, f32)], i: usize, at: f32) -> Vec<f32> {
    let mut v = sizes.to_vec();
    if i + 1 >= v.len() {
        return v;
    }
    let before: f32 = v[..i].iter().sum();
    let pair = v[i] + v[i + 1];
    let lo = bounds[i].0.max(pair - bounds[i + 1].1);
    let hi = bounds[i].1.min(pair - bounds[i + 1].0);
    let a = (at - before).clamp(lo.min(hi), hi.max(lo));
    v[i] = a;
    v[i + 1] = pair - a;
    v
}

impl<S: 'static> Resizable<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Resizable<S> {
        Resizable { key: key.into(), name: name.into(), panes: Vec::new(), sizes: Vec::new(), vertical: false, grip: false, on_resize: None }
    }

    pub fn pane(mut self, p: Pane) -> Self {
        self.panes.push(p);
        self
    }

    /// Each pane's share; missing or wrong-length shares split evenly.
    pub fn sizes(mut self, v: &[f32]) -> Self {
        self.sizes = v.to_vec();
        self
    }

    /// Stacked, with handles between rows.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// A small grip on each handle.
    pub fn grip(mut self) -> Self {
        self.grip = true;
        self
    }

    pub fn on_resize(mut self, f: impl Fn(&mut S, &[f32]) + 'static) -> Self {
        self.on_resize = Some(Rc::new(f));
        self
    }

    /// The key of the handle after pane `i`.
    pub fn handle_key(key: &str, i: usize) -> String {
        format!("{key}-handle-{i}")
    }

    pub fn build(self, t: &Theme) -> El {
        let n = self.panes.len();
        let sizes: Vec<f32> = if self.sizes.len() == n && n > 0 { self.sizes.clone() } else { vec![1.0 / n.max(1) as f32; n] };
        let bounds: Vec<(f32, f32)> = self.panes.iter().map(|p| (p.min, p.max)).collect();
        let vertical = self.vertical;
        let k = self.key.clone();
        let mut group = if vertical { col() } else { row() }.key(k.clone()).role("group").label(self.name.clone()).w_full().h_full();
        for (i, p) in self.panes.into_iter().enumerate() {
            let holder = if vertical { col().w_full().h(0.0) } else { col().h_full().w(0.0) }.key(format!("{k}-pane-{i}")).grow(sizes[i]).shrink().min_w(0.0).min_h(0.0).child(p.content);
            group = group.child(holder);
            if i + 1 == n {
                break;
            }
            let mid = (HANDLE - 1.0) / 2.0;
            let line = if vertical { el().absolute(0.0, mid).w_full().h(1.0) } else { el().absolute(mid, 0.0).h_full().w(1.0) }.fill(t.c.line).a11y_hidden();
            let mut h = if vertical { row().w_full().h(HANDLE) } else { col().h_full().w(HANDLE) }.key(Self::handle_key(&k, i)).role("separator").label(format!("Resize {}", if p.name.is_empty() { format!("pane {}", i + 1) } else { p.name.clone() })).range_value(f64::from(sizes[i] * 100.0), f64::from(bounds[i].0 * 100.0), f64::from(bounds[i].1 * 100.0)).focusable(true).center().cursor(if vertical { Cursor::ResizeRow } else { Cursor::ResizeColumn }).hover(t.c.hover).radius(HANDLE / 2.0).focus_ring(t.control.ring, 0.0, t.c.ring).child(line);
            if self.grip {
                let (gw, gh) = if vertical { (24.0, 6.0) } else { (6.0, 24.0) };
                h = h.child(paint(el().size(gw, gh).radius(3.0), &t.mat.raised.base).a11y_hidden());
            }
            if let Some(f) = self.on_resize.clone() {
                let (sizes2, bounds2, gk, g) = (sizes.clone(), bounds.clone(), k.clone(), f.clone());
                let handles = n.saturating_sub(1) as f32;
                h = h.on_drag(move |s: &mut S, d: &mut Drag, tx: &mut Tx| {
                    tx.instant();
                    if d.phase == Phase::Down {
                        d.data = sizes2[i];
                    }
                    if d.cancelled() {
                        return;
                    }
                    let len = LENGTHS.with(|m| m.borrow().get(&gk).copied()).unwrap_or(0.0);
                    let avail = (len - handles * HANDLE).max(1.0);
                    let before: f32 = sizes2[..i].iter().sum();
                    let travel = if vertical { d.dy } else { d.dx };
                    g(s, &moved(&sizes2, &bounds2, i, before + d.data + travel / avail));
                });
                let (sizes, bounds) = (sizes.clone(), bounds.clone());
                h = h.on_key(move |s: &mut S, ev: &KeyEvent, _: &mut Tx| {
                    let step = if ev.mods.shift { 0.2 } else { 0.05 };
                    let before: f32 = sizes[..i].iter().sum();
                    let at = match ev.key {
                        Key::Left | Key::Up => before + sizes[i] - step,
                        Key::Right | Key::Down => before + sizes[i] + step,
                        Key::Home => before,
                        Key::End => before + 1.0,
                        _ => return false,
                    };
                    f(s, &moved(&sizes, &bounds, i, at));
                    true
                });
            }
            group = group.child(h);
        }
        // The group's length, for turning a handle's travel into a share: kept per group as it lays out (no view rebuild).
        let lk = k.clone();
        group.on_layout_tx(move |_: &mut S, r: [f32; 4], tx: &mut Tx| {
            LENGTHS.with(|m| m.borrow_mut().insert(lk.clone(), if vertical { r[3] } else { r[2] }));
            tx.live_only();
        })
    }
}
