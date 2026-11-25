//! rune-ui registry: hover_card 0.8.0 Copied into your app by `rune-ui add hover_card`; yours to change.
//!
//! A preview of what a link or a name points to, shown when the pointer rests on it: the **floating** material beside the trigger,
//! placed by Rune's anchoring through the overlay layer (flipping when there is no room, kept inside the window), with no scrim and no
//! focus trap, since it is a glance and not a dialog. It opens after the pointer rests 500 ms and stays while the pointer is on the
//! trigger or the card, closing 200 ms after it leaves both.
//!
//! From the keyboard the trigger is one Tab stop: Return or Space shows the card (pressing pins it too), Escape hides it.
//!
//! ```ignore
//! HoverCard::new("author", "Asha's profile", s.author).trigger(name).child(profile)
//!     .on_change(|s: &mut App, h| s.author = h).build(t, &mut layer)
//! ```

use std::rc::Rc;
use std::time::Duration;

use rune::{Align, El, Key, KeyEvent, Side, Spring, SpringPair, Transition, Tx, Vec2, col, row};

use super::base::{Shape, Theme, paint};

/// Where the pointer and the keyboard have left the card. The app keeps it; the card is open while any of them holds it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoverCardState {
    pub trigger: bool,
    pub card: bool,
    pub pinned: bool,
}

impl HoverCardState {
    pub fn open(self) -> bool {
        self.trigger || self.card || self.pinned
    }
}

pub type OnChange<S> = Rc<dyn Fn(&mut S, HoverCardState)>;

pub struct HoverCard<S: 'static> {
    key: String,
    name: String,
    state: HoverCardState,
    trigger: Option<El>,
    body: Vec<El>,
    width: f32,
    side: Side,
    change: Option<OnChange<S>>,
}

impl<S: 'static> HoverCard<S> {
    /// `name` names the trigger for assistive technology.
    pub fn new(key: impl Into<String>, name: impl Into<String>, state: HoverCardState) -> HoverCard<S> {
        HoverCard { key: key.into(), name: name.into(), state, trigger: None, body: Vec::new(), width: 300.0, side: Side::Bottom, change: None }
    }

    /// What the pointer rests on (words, an avatar); not itself focusable.
    pub fn trigger(mut self, e: El) -> Self {
        self.trigger = Some(e);
        self
    }

    pub fn child(mut self, e: El) -> Self {
        self.body.push(e);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Which side of the trigger (below by default; it flips when there is no room).
    pub fn side(mut self, s: Side) -> Self {
        self.side = s;
        self
    }

    pub fn on_change(mut self, f: impl Fn(&mut S, HoverCardState) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    /// The trigger in place; the open card goes into `layer` (mounted at the view's root: `base::overlay_root`).
    pub fn build(mut self, t: &Theme, layer: &mut Vec<El>) -> El {
        let anchor = format!("{}-trigger", self.key);
        let st = self.state;
        let open = st.open();
        let mut trigger = row().key(anchor.clone()).role("button").label(self.name.clone()).expanded(open).focusable(true).radius(t.radius.sm).round_as(t).focus_ring(t.control.ring, t.control.ring_offset, t.c.ring).children(self.trigger.take()).hover_delays(Duration::from_millis(500), Duration::from_millis(200));
        if let Some(f) = self.change.clone() {
            let g = f.clone();
            let h = f.clone();
            trigger = trigger
                .on_hover_change(move |s: &mut S, over: bool, _: &mut Tx| f(s, HoverCardState { trigger: over, ..st }))
                .on_press(move |s: &mut S| g(s, HoverCardState { pinned: !open, trigger: false, card: false }))
                .on_key(move |s: &mut S, k: &KeyEvent, _: &mut Tx| {
                    match &k.key {
                        Key::Escape if open => h(s, HoverCardState::default()),
                        Key::Enter => h(s, HoverCardState { pinned: !open, trigger: false, card: false }),
                        Key::Text(c) if c == " " => h(s, HoverCardState { pinned: !open, trigger: false, card: false }),
                        _ => return false,
                    }
                    true
                });
        }
        if open {
            let mut card = col().key(format!("{}-card", self.key)).role("tooltip").label(self.name.clone()).w(self.width).padding(t.metrics.pad_md).gap(t.metrics.gap).radius(t.radius.xl).round_as(t).hover_delays(Duration::ZERO, Duration::from_millis(200));
            card = paint(card, &t.mat.floating.base).children(std::mem::take(&mut self.body));
            if let Some(f) = self.change.clone() {
                card = card.on_hover_change(move |s: &mut S, over: bool, _: &mut Tx| f(s, HoverCardState { card: over, ..st }));
            }
            let from = Vec2 { x: 0.5, y: if self.side == Side::Top { 1.0 } else { 0.0 } };
            let motion = Transition::scale_anchor(0.97, from).springs(SpringPair::new(Spring::new(0.14, 0.0), Spring::new(0.09, 0.0)));
            let placed = card.anchored(anchor.clone(), self.side, Align::Center, t.space.at(2.0));
            layer.push(if t.reduced_motion { placed } else { placed.transition(motion) });
        }
        trigger
    }
}
