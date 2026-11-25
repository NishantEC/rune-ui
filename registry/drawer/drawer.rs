//! rune-ui registry: drawer 0.8.0 Copied into your app by `rune-ui add drawer`; yours to change. Needs: sheet.
//!
//! A sheet that rises from the bottom edge: the **floating** material rounded on its top corners, a grab bar, a centred title and
//! description, its body, and a footer of full-width actions. It slides up over a scrim and back down. Behaviour is Sheet's (the
//! shared `sheet::mount`): focus trapped, Escape and a press outside close it, focus goes back to what opened it. Dragging the grab
//! bar down more than 64 points closes it too. `panel` draws it in place.
//!
//! ```ignore
//! Drawer::new("goal", "Move goal").description("Set your daily activity goal.").child(body).footer(vec![submit, cancel])
//!     .open(s.drawer).return_focus("goal-open").on_close(|s: &mut App| s.drawer = false).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{Drag, Edge, El, Phase, Shadow, Spring, SpringPair, Transition, Tx, col, el, row, text};
use rune_ui_style::{Recipe, weight};

use super::base::{Press, Shape, Theme, paint};
use super::sheet::{Side, mount};

/// How far down the grab bar must be dragged to close the drawer.
pub const DISMISS: f32 = 64.0;

pub struct Drawer<S: 'static> {
    key: String,
    title: String,
    description: Option<String>,
    body: Vec<El>,
    footer: Vec<El>,
    width: f32,
    open: bool,
    return_to: Option<String>,
    close: Option<Press<S>>,
}

impl<S: 'static> Drawer<S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Drawer<S> {
        Drawer { key: key.into(), title: title.into(), description: None, body: Vec::new(), footer: Vec::new(), width: 440.0, open: false, return_to: None, close: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn child(mut self, c: El) -> Self {
        self.body.push(c);
        self
    }

    /// Actions under the body, stacked and full width, most important first.
    pub fn footer(mut self, buttons: Vec<El>) -> Self {
        self.footer = buttons;
        self
    }

    /// The content's width (the panel spans the window).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn return_focus(mut self, key: impl Into<String>) -> Self {
        self.return_to = Some(key.into());
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(Rc::new(f));
        self
    }

    /// The panel alone, drawn in place.
    pub fn panel(&mut self, t: &Theme) -> El {
        let r = t.radius.xxl;
        let mut grab = row().key(format!("{}-grab", self.key)).w_full().h(t.space.at(6.0)).center().child(el().size(40.0, 5.0).radius(2.5).fill(t.ink_faint().alpha(0.4)).a11y_hidden());
        if let Some(f) = self.close.clone() {
            grab = grab.cursor(rune::Cursor::Grab).on_drag(move |s: &mut S, d: &mut Drag, _: &mut Tx| {
                if d.phase == Phase::Up && !d.cancelled() && d.dy > DISMISS {
                    f(s);
                }
            });
        }
        let mut head = col().w_full().items_center().gap(t.space.at(1.5)).child(text(self.title.clone()).font(weight(t.ty.lg, t.ty.semibold)).color(t.c.floating.fg).role("heading").level(2));
        if let Some(d) = &self.description {
            head = head.child(text(d.clone()).font(t.ty.base).color(t.ink_muted()).wrap().min_w(0.0));
        }
        let mut inner = col().w_full().max_w(self.width).gap(t.metrics.stack - 4.0).child(head).children(std::mem::take(&mut self.body));
        if !self.footer.is_empty() {
            inner = inner.child(col().key(format!("{}-footer", self.key)).w_full().gap(t.space.at(2.0)).pt(t.space.at(1.0)).children(std::mem::take(&mut self.footer)));
        }
        let mut p = col().key(format!("{}-drawer", self.key)).role("dialog").label(self.title.clone()).w_full().items_center().px(t.metrics.panel).pb(t.metrics.panel).corners([r, r, 0.0, 0.0]).round_as(t);
        if let Some(d) = &self.description {
            p = p.description(d.clone());
        }
        p = paint(p, &Recipe::new().fill(t.c.floating.bg).shadows(&t.elevation.lg).shadow(Shadow { spread: 1.0, color: t.c.line, ..Shadow::default() }));
        p.child(grab).child(inner)
    }

    /// Nothing in place; while open, the scrim and the drawer go into `layer`.
    pub fn build(mut self, t: &Theme, layer: &mut Vec<El>) {
        if !self.open {
            return;
        }
        let key = self.key.clone();
        let (close, return_to) = (self.close.clone(), self.return_to.clone());
        let motion = Transition::move_edge(Edge::Bottom).springs(SpringPair::new(Spring::new(0.3, 0.0), Spring::new(0.22, 0.0)));
        let panel = if t.reduced_motion { self.panel(t) } else { self.panel(t).transition(motion) };
        mount(t, layer, &key, panel, Side::Bottom, return_to.as_deref(), close);
    }
}
