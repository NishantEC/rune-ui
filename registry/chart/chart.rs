//! rune-ui registry: chart 0.5.1. Copied into your app by `rune-ui add chart`; yours to change.
//!
//! Small, solid charts drawn with Rune's shapes: **line**, **bar** (grouped), **area** and **donut**. Series take the theme's series
//! colours (`recipes::series`: the accent first), so a theme or the editor's accent recolours them. Line, bar and area charts have a
//! y axis on rounded ("nice") steps with hairline grid lines, category labels under the plot in tabular figures, and a legend; the
//! donut has its total in the middle and a legend of its slices.
//!
//! Hovering a category (or a slice) highlights it and shows a tooltip on the floating material with each series' colour, name and
//! value; the tooltip goes into the overlay layer, anchored beside the point, so it is never clipped and flips at the window's edge.
//! The plot is one Tab stop: Left and Right move the highlight, Escape clears it. The figure's name for assistive technology lists
//! every value. The app keeps which category is highlighted.
//!
//! ```ignore
//! Chart::bar("visits", "Visits").categories(&months).series(Series::new("Desktop", desktop)).series(Series::new("Mobile", mobile))
//!     .hover(s.hover).on_hover(|s: &mut App, i| s.hover = i).build(t, &mut layer)
//! ```

use std::f32::consts::{PI, TAU};
use std::rc::Rc;

use rune::{Align, Color, El, Key, KeyEvent, PointerEvent, Shape, Side, Tx, canvas, col, el, row, text};
use rune_ui_style::{recipes, tabular, weight};

use super::base::{Theme, paint};

/// What is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Line,
    Bar,
    Area,
    Donut,
}

/// A named run of values, one per category.
#[derive(Clone, Debug)]
pub struct Series {
    pub name: String,
    pub values: Vec<f32>,
}

impl Series {
    pub fn new(name: impl Into<String>, values: Vec<f32>) -> Series {
        Series { name: name.into(), values }
    }
}

/// Round steps for an axis from 0 to at least `max`: about four of 1, 2, 2.5 or 5 times a power of ten. Returns (step, top).
pub fn nice(max: f32) -> (f32, f32) {
    if max <= 0.0 || !max.is_finite() {
        return (1.0, 4.0);
    }
    let raw = max / 4.0;
    let p = 10f32.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * p).find(|s| *s >= raw).unwrap_or(10.0 * p);
    (step, (max / step).ceil() * step)
}

fn words(v: f32) -> String {
    let whole = v.round() as i64;
    let digits = whole.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if (v - whole as f32).abs() > 0.05 { format!("{v:.1}") } else if whole < 0 { format!("-{out}") } else { out }
}

pub type OnHover<S> = Rc<dyn Fn(&mut S, Option<usize>)>;

pub struct Chart<S: 'static> {
    key: String,
    name: String,
    kind: Kind,
    categories: Vec<String>,
    series: Vec<Series>,
    w: f32,
    h: f32,
    legend: bool,
    hover: Option<usize>,
    on_hover: Option<OnHover<S>>,
    format: fn(f32) -> String,
}

impl<S: 'static> Chart<S> {
    fn new(kind: Kind, key: impl Into<String>, name: impl Into<String>) -> Chart<S> {
        Chart { key: key.into(), name: name.into(), kind, categories: Vec::new(), series: Vec::new(), w: 420.0, h: 200.0, legend: true, hover: None, on_hover: None, format: words }
    }

    pub fn line(key: impl Into<String>, name: impl Into<String>) -> Chart<S> {
        Chart::new(Kind::Line, key, name)
    }

    pub fn bar(key: impl Into<String>, name: impl Into<String>) -> Chart<S> {
        Chart::new(Kind::Bar, key, name)
    }

    pub fn area(key: impl Into<String>, name: impl Into<String>) -> Chart<S> {
        Chart::new(Kind::Area, key, name)
    }

    /// One series; its values are the slices, the categories their names.
    pub fn donut(key: impl Into<String>, name: impl Into<String>) -> Chart<S> {
        Chart { w: 200.0, h: 200.0, ..Chart::new(Kind::Donut, key, name) }
    }

    pub fn categories(mut self, c: &[&str]) -> Self {
        self.categories = c.iter().map(|s| (*s).to_owned()).collect();
        self
    }

    pub fn series(mut self, s: Series) -> Self {
        self.series.push(s);
        self
    }

    /// The plot's size (the donut's diameter is the smaller side).
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }

    /// The highlighted category or slice (the app keeps this).
    pub fn hover(mut self, i: Option<usize>) -> Self {
        self.hover = i;
        self
    }

    pub fn on_hover(mut self, f: impl Fn(&mut S, Option<usize>) + 'static) -> Self {
        self.on_hover = Some(Rc::new(f));
        self
    }

    /// How values are written (whole numbers with thousands separators by default).
    pub fn format(mut self, f: fn(f32) -> String) -> Self {
        self.format = f;
        self
    }

    /// The figure's name: every value, for assistive technology.
    fn summary(&self) -> String {
        let kind = match self.kind {
            Kind::Line => "line chart",
            Kind::Bar => "bar chart",
            Kind::Area => "area chart",
            Kind::Donut => "donut chart",
        };
        let f = self.format;
        let parts: Vec<String> = self.categories.iter().enumerate().map(|(i, c)| {
            let vals: Vec<String> = self.series.iter().map(|s| format!("{} {}", s.name, s.values.get(i).map_or("none".to_owned(), |v| f(*v)))).collect();
            format!("{c}: {}", vals.join(", "))
        }).collect();
        format!("{}, {kind}. {}", self.name, parts.join("; "))
    }

    fn color(&self, t: &Theme, i: usize) -> Color { // guard: allow (a return type, not a colour value)
        recipes::series(t, i)
    }

    /// The tooltip for category `i`, mounted beside `anchor`.
    fn tooltip(&self, t: &Theme, i: usize, anchor: &str, side: Side, layer: &mut Vec<El>) {
        let f = self.format;
        let mut tip = col().key(format!("{}-tooltip", self.key)).role("tooltip").gap(t.space.at(1.0)).px(t.space.at(2.5)).py(t.space.at(2.0)).radius(t.radius.lg).min_w(120.0);
        tip = paint(tip, &t.mat.floating.base);
        let title = self.categories.get(i).cloned().unwrap_or_default();
        tip = tip.child(text(title.clone()).font(weight(t.ty.sm, t.ty.medium)).color(t.ink()));
        let lines: Vec<(Color, String, f32)> = if self.kind == Kind::Donut {
            self.series.first().and_then(|s| s.values.get(i)).map(|v| vec![(self.color(t, i), self.series[0].name.clone(), *v)]).unwrap_or_default()
        } else {
            self.series.iter().enumerate().filter_map(|(si, s)| s.values.get(i).map(|v| (self.color(t, si), s.name.clone(), *v))).collect()
        };
        let mut label = title;
        for (c, name, v) in lines {
            label.push_str(&format!(", {name} {}", f(v)));
            tip = tip.child(row().w_full().gap(t.space.at(2.0)).center_y().child(el().size(8.0, 8.0).radius(2.0).fill(c)).child(text(name).font(t.ty.sm).color(t.ink_muted()).grow(1.0)).child(text(f(v)).font(weight(tabular(t.ty.sm), t.ty.medium)).color(t.ink())));
        }
        layer.push(tip.label(label).anchored(anchor.to_owned(), side, Align::Center, t.space.at(3.0)));
    }

    fn legend_row(&self, t: &Theme, names: Vec<String>) -> El {
        let mut r = row().key(format!("{}-legend", self.key)).gap(t.space.at(4.0)).flex_wrap().center().a11y_hidden();
        for (i, n) in names.into_iter().enumerate() {
            r = r.child(row().gap(t.space.at(1.5)).center_y().child(el().size(8.0, 8.0).radius(2.0).fill(self.color(t, i))).child(text(n).font(t.ty.sm).color(t.ink_muted())));
        }
        r
    }

    /// Hover and keys on the plot: `pick` turns a local point into a category.
    fn interactive(&self, plot: El, n: usize, pick: impl Fn(f32, f32) -> Option<usize> + 'static) -> El {
        let Some(f) = self.on_hover.clone() else { return plot };
        let now = self.hover;
        let (g, h) = (f.clone(), f.clone());
        plot.focusable(true)
            .on_pointer_move_local(move |s: &mut S, p: PointerEvent, tx: &mut Tx| {
                let at = pick(p.x, p.y);
                if at == now {
                    tx.live_only();
                } else {
                    f(s, at);
                }
            })
            .on_hover_change(move |s: &mut S, over: bool, _: &mut Tx| {
                if !over {
                    g(s, None);
                }
            })
            .on_key(move |s: &mut S, k: &KeyEvent, _: &mut Tx| {
                let to = match (&k.key, now) {
                    (Key::Right, None) | (Key::Home, _) => Some(0),
                    (Key::Left, None) | (Key::End, _) => Some(n.saturating_sub(1)),
                    (Key::Right, Some(i)) => Some((i + 1).min(n.saturating_sub(1))),
                    (Key::Left, Some(i)) => Some(i.saturating_sub(1)),
                    (Key::Escape, Some(_)) => None,
                    _ => return false,
                };
                h(s, to);
                true
            })
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        if self.kind == Kind::Donut {
            return self.donut_el(t, layer);
        }
        let k = self.key.clone();
        let n = self.categories.len().max(1);
        let max = self.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0f32, f32::max);
        let (step, top) = nice(max);
        let f = self.format;
        let yw = (0..=((top / step).round() as usize)).map(|i| f(step * i as f32).chars().count()).max().unwrap_or(1) as f32 * t.ty.xs.size * 0.62 + t.space.at(3.0);
        let (pw, ph) = (self.w - yw, self.h);
        let band = pw / n as f32;
        let x = |i: usize| (i as f32 + 0.5) * band;
        let y = |v: f32| ph - (v / top).clamp(0.0, 1.0) * ph;
        let lh = t.ty.xs.line_height;
        // The y axis and the grid.
        let mut axis = el().w(yw).h(ph).a11y_hidden();
        let mut plot = el().key(format!("{k}-plot")).role("img").label(self.summary()).size(pw, ph);
        let ticks = (top / step).round() as usize;
        for i in 0..=ticks {
            let v = step * i as f32;
            let yy = y(v);
            axis = axis.child(row().absolute(0.0, yy - lh / 2.0).w(yw - t.space.at(2.0)).justify_end().child(text(f(v)).font(tabular(t.ty.xs)).color(t.ink_faint())));
            plot = plot.child(el().absolute(0.0, (yy - 0.5).max(0.0).min(ph - 1.0)).w(pw).h(1.0).fill(if i == 0 { t.c.line } else { t.c.line.alpha(t.c.line.a * 0.5) }).a11y_hidden());
        }
        // The highlighted band.
        if let Some(i) = self.hover.filter(|i| *i < n) {
            plot = plot.child(el().absolute(i as f32 * band, 0.0).size(band, ph).radius(t.radius.sm).fill(t.c.hover).a11y_hidden());
        }
        match self.kind {
            Kind::Bar => {
                let ns = self.series.len().max(1) as f32;
                let inner = band * 0.72;
                let gap = 2.0;
                let bw = ((inner - gap * (ns - 1.0)) / ns).max(1.0);
                let r = t.radius.sm.min(bw / 2.0);
                for (si, s) in self.series.iter().enumerate() {
                    for (i, v) in s.values.iter().enumerate().take(n) {
                        let x0 = i as f32 * band + (band - inner) / 2.0 + si as f32 * (bw + gap);
                        let hgt = ph - y(*v);
                        plot = plot.child(el().key(format!("{k}-bar-{si}-{i}")).absolute(x0, ph - hgt).size(bw, hgt.max(0.5)).corners([r, r, 0.0, 0.0]).fill(self.color(t, si)).a11y_hidden());
                    }
                }
            }
            Kind::Line | Kind::Area => {
                let mut shape = Shape::new();
                for (si, s) in self.series.iter().enumerate() {
                    let c = self.color(t, si);
                    let pts: Vec<[f32; 2]> = s.values.iter().enumerate().take(n).map(|(i, v)| [x(i), y(*v)]).collect();
                    if self.kind == Kind::Area && pts.len() >= 2 {
                        let mut poly = vec![[pts[0][0], ph]];
                        poly.extend(pts.iter().copied());
                        poly.push([pts[pts.len() - 1][0], ph]);
                        shape = shape.fill(poly, c.alpha(0.16));
                    }
                    shape = shape.stroke(pts, 2.0, c);
                }
                plot = plot.child(canvas(shape).absolute(0.0, 0.0).size(pw, ph).a11y_hidden());
                if let Some(i) = self.hover.filter(|i| *i < n) {
                    for (si, s) in self.series.iter().enumerate() {
                        if let Some(v) = s.values.get(i) {
                            plot = plot.child(el().absolute(x(i) - 4.0, y(*v) - 4.0).size(8.0, 8.0).radius(4.0).fill(self.color(t, si)).box_shadow(rune::Shadow { spread: 2.0, color: t.c.page.bg, ..Default::default() }).a11y_hidden());
                        }
                    }
                }
            }
            Kind::Donut => {}
        }
        // A one-point anchor per category at its top centre: the tooltip's place.
        for i in 0..n {
            let top_v = self.series.iter().filter_map(|s| s.values.get(i)).copied().fold(0.0f32, f32::max);
            plot = plot.child(el().key(format!("{k}-anchor-{i}")).absolute(x(i), y(top_v).min(ph - 1.0)).size(1.0, 1.0).a11y_hidden());
        }
        let plot = self.interactive(plot, n, move |px, _| (px >= 0.0 && px < pw).then(|| ((px / band) as usize).min(n - 1)));
        if let Some(i) = self.hover.filter(|i| *i < n) {
            self.tooltip(t, i, &format!("{k}-anchor-{i}"), Side::Right, layer);
        }
        let mut xs = el().w(pw).h(lh + t.space.at(2.0)).a11y_hidden();
        for (i, c) in self.categories.iter().enumerate() {
            xs = xs.child(row().absolute(i as f32 * band, t.space.at(2.0)).w(band).justify_center().child(text(c.clone()).font(tabular(t.ty.xs)).color(t.ink_faint()).truncate()));
        }
        let mut figure = col().key(k.clone()).role("figure").label(self.name.clone()).gap(t.space.at(3.0)).child(row().pt(lh / 2.0).child(axis).child(plot)).child(row().child(el().w(yw)).child(xs));
        if self.legend {
            figure = figure.child(row().w(self.w).child(el().w(yw)).child(row().w(pw).justify_center().child(self.legend_row(t, self.series.iter().map(|s| s.name.clone()).collect()))));
        }
        figure
    }

    fn donut_el(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let k = self.key.clone();
        let d = self.w.min(self.h);
        let (cx, cy, ro) = (d / 2.0, d / 2.0, d / 2.0);
        let ri = ro * 0.62;
        let values: Vec<f32> = self.series.first().map(|s| s.values.iter().map(|v| v.max(0.0)).collect()).unwrap_or_default();
        let total: f32 = values.iter().sum::<f32>().max(f32::EPSILON);
        let n = values.len().max(1);
        let pad = if values.len() > 1 { 0.025 } else { 0.0 };
        let mut shape = Shape::new();
        let mut plot = el().key(format!("{k}-plot")).role("img").label(self.summary()).size(d, d);
        let mut start = -PI / 2.0;
        let mut spans = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let sweep = v / total * TAU;
            let (a0, a1) = (start + pad, start + sweep - pad);
            spans.push((start, start + sweep));
            if a1 > a0 {
                let hovered = self.hover == Some(i);
                let grow = if hovered { 3.0 } else { 0.0 };
                let steps = ((a1 - a0) / 0.05).ceil().max(2.0) as usize;
                let arc = |r: f32, rev: bool| -> Vec<[f32; 2]> {
                    (0..=steps).map(|s| {
                        let f = s as f32 / steps as f32;
                        let a = if rev { a1 - (a1 - a0) * f } else { a0 + (a1 - a0) * f };
                        [cx + r * a.cos(), cy + r * a.sin()]
                    }).collect()
                };
                let mut poly = arc(ro - 3.0 + grow, false);
                poly.extend(arc(ri, true));
                let c = self.color(t, i);
                shape = shape.fill(poly, if self.hover.is_some() && !hovered { c.alpha(0.55) } else { c });
            }
            let mid = start + sweep / 2.0;
            plot = plot.child(el().key(format!("{k}-anchor-{i}")).absolute(cx + ro * mid.cos(), cy + ro * mid.sin()).size(1.0, 1.0).a11y_hidden());
            start += sweep;
        }
        plot = plot.child(canvas(shape).absolute(0.0, 0.0).size(d, d).a11y_hidden());
        let f = self.format;
        let label = self.series.first().map(|s| s.name.clone()).unwrap_or_default();
        plot = plot.child(col().absolute(0.0, 0.0).size(d, d).center().a11y_hidden().child(text(f(values.iter().sum())).font(weight(tabular(t.ty.xl), t.ty.semibold)).color(t.ink())).child(text(label).font(t.ty.xs).color(t.ink_muted())));
        let plot = self.interactive(plot, n, move |px, py| {
            let (dx, dy) = (px - cx, py - cy);
            let r = dx.hypot(dy);
            if r < ri || r > ro {
                return None;
            }
            let mut a = dy.atan2(dx);
            if a < -PI / 2.0 {
                a += TAU;
            }
            spans.iter().position(|(s, e)| a >= *s && a < *e)
        });
        if let Some(i) = self.hover.filter(|i| *i < n) {
            self.tooltip(t, i, &format!("{k}-anchor-{i}"), Side::Right, layer);
        }
        let mut figure = col().key(k).role("figure").label(self.name.clone()).items_center().gap(t.space.at(3.0)).child(plot);
        if self.legend {
            figure = figure.child(self.legend_row(t, self.categories.clone()).max_w(d.max(240.0)));
        }
        figure
    }
}
