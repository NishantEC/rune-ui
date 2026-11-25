//! rune-ui registry: action_bar 0.8.0. Copied into your app by `rune-ui add action_bar`; yours to change.
//! Needs: button.
//!
//! What can be done to a selection, in one floating bar at the bottom of the window: how many are chosen, the actions, and a button
//! that clears the choice. It is there only while something is chosen (a count of zero draws nothing), rises from the bottom over the
//! page in the overlay layer (it never moves the page; a cut under Reduce Motion), and keeps its place as the page scrolls.
//!
//! Keys: the bar is one Tab stop (a toolbar); Left and Right move between its buttons, Home and End to the ends; Escape clears the
//! choice, as Clear does, and focus goes back to `return_focus` (the table, say). The app owns the selection: the bar only reports
//! what was pressed. It works with DataTable's selection: `DataTableState::selected.len()` is the count, clearing empties it.
//!
//! ```ignore
//! ActionBar::new("bulk", s.table.selected.len()).noun("run", "runs").action("Archive", Some(&icons::INBOX), |s: &mut App| s.archive())
//!     .danger("Delete", Some(&icons::TRASH), |s: &mut App| s.delete()).on_clear(|s: &mut App| s.table.selected.clear())
//!     .return_focus("runs").build(t, &mut layer)
//! ```

use rune::{Edge, El, Icon, Key, KeyEvent, Spring, SpringPair, Transition, Tx, col, el, row, text};
use rune_ui_style::{ButtonVariant, Radius, Size, tabular, weight};

use super::base::{Press, Shape, States, Theme, glyph, hit_area, paint, paint_state, press};
use super::button::Button;
use super::icons;

/// A press that also needs the frame's transaction (to move focus).
type PressTx<S> = std::rc::Rc<dyn Fn(&mut S, &mut Tx)>;

struct Action<S: 'static> {
    label: String,
    icon: Option<&'static Icon>,
    danger: bool,
    press: Press<S>,
}

pub struct ActionBar<S: 'static> {
    key: String,
    count: usize,
    noun: (String, String),
    actions: Vec<Action<S>>,
    clear: Option<Press<S>>,
    return_to: Option<String>,
}

impl<S: 'static> ActionBar<S> {
    /// `count` things are chosen; at zero the bar is not drawn.
    pub fn new(key: impl Into<String>, count: usize) -> ActionBar<S> {
        ActionBar { key: key.into(), count, noun: ("item".to_owned(), "items".to_owned()), actions: Vec::new(), clear: None, return_to: None }
    }

    /// What is counted ("row", "rows"): "3 rows selected".
    pub fn noun(mut self, one: impl Into<String>, many: impl Into<String>) -> Self {
        self.noun = (one.into(), many.into());
        self
    }

    /// An action, with an optional icon before its words.
    pub fn action(mut self, label: impl Into<String>, icon: Option<&'static Icon>, f: impl Fn(&mut S) + 'static) -> Self {
        self.actions.push(Action { label: label.into(), icon, danger: false, press: press(f) });
        self
    }

    /// An action that cannot be taken back, in the danger material.
    pub fn danger(mut self, label: impl Into<String>, icon: Option<&'static Icon>, f: impl Fn(&mut S) + 'static) -> Self {
        self.actions.push(Action { label: label.into(), icon, danger: true, press: press(f) });
        self
    }

    /// Clear (the close button and Escape): the app empties its selection.
    pub fn on_clear(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.clear = Some(press(f));
        self
    }

    /// What takes focus after the selection is cleared from the bar (the list or table it belongs to).
    pub fn return_focus(mut self, key: impl Into<String>) -> Self {
        self.return_to = Some(key.into());
        self
    }

    /// The words the bar says: "1 row selected", "3 rows selected".
    pub fn words(&self) -> String {
        format!("{} {} selected", self.count, if self.count == 1 { &self.noun.0 } else { &self.noun.1 })
    }

    /// Mounts the bar into `layer` (nothing while the count is zero).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) {
        if self.count == 0 {
            return;
        }
        let k = self.key.clone();
        let words = self.words();
        let line = || el().w(1.0).h(t.control.sm - 4.0).fill(t.c.line).a11y_hidden();
        let mut bar = row().key(k.clone()).role("toolbar").label(words.clone()).focus_group().h(t.control.md + 2.0 * t.space.at(1.5)).pl(t.metrics.pad_md).pr(t.space.at(1.5)).gap(t.space.at(1.5)).center_y().radius(t.radius.xl).round_as(t);
        bar = paint(bar, &t.mat.floating.base);
        bar = bar.child(text(words.clone()).key(format!("{k}-count")).font(weight(tabular(t.ty.base), t.ty.medium)).color(t.ink()).live_region(rune::LiveRegion::Polite)).child(row().w(t.space.at(1.5))).child(line());
        for (i, a) in self.actions.into_iter().enumerate() {
            let f = a.press.clone();
            let mut b = Button::<S>::new(format!("{k}-action-{i}"), a.label).variant(if a.danger { ButtonVariant::Destructive } else { ButtonVariant::Ghost }).size(Size::Sm).on_press(move |s: &mut S| f(s));
            if let Some(icon) = a.icon {
                b = b.icon(icon);
            }
            bar = bar.child(b.build(t));
        }
        let return_to = self.return_to.clone();
        let clear = self.clear.clone();
        if let Some(f) = clear.clone() {
            let back = return_to.clone();
            let go: PressTx<S> = std::rc::Rc::new(move |s: &mut S, tx: &mut Tx| {
                f(s);
                if let Some(r) = &back {
                    tx.focus(r.clone(), true);
                }
            });
            let h = t.control.sm;
            let face = row().key(format!("{k}-clear")).role("button").label("Clear selection").help("Clear selection (Escape)").focusable(true).size(h, h).center().radius(Radius::fit(t.radius.md, h)).round_as(t);
            let g = go.clone();
            let face = paint_state(face, &t.mat.ghost, States::default()).child(glyph(&icons::CLOSE, t.control.icon, t.ink_muted())).on_press_tx(move |s: &mut S, tx: &mut Tx| g(s, tx));
            let slop = t.control.slop(h);
            bar = bar.child(line()).child(hit_area::<S>(&format!("{k}-clear"), face, slop, slop, None).on_press_tx(move |s: &mut S, tx: &mut Tx| go(s, tx)));
        }
        bar = bar.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            if ev.key != Key::Escape {
                return false;
            }
            if let Some(f) = &clear {
                f(s);
                if let Some(r) = &return_to {
                    tx.focus(r.clone(), true);
                }
            }
            true
        });
        // Rises a little from the bottom and fades in; leaves faster than it came.
        let motion = Transition::move_edge(Edge::Bottom).springs(SpringPair::new(Spring::new(0.24, 0.0), Spring::new(0.16, 0.0)));
        let bar = if t.reduced_motion { bar } else { bar.transition(motion) };
        layer.push(col().key(format!("{k}-layer")).absolute(0.0, 0.0).w_full().h_full().pb(t.space.at(6.0)).justify_end().items_center().child(bar));
    }
}
