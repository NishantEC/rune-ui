//! rune-ui registry: form 0.5.1. Copied into your app by `rune-ui add form`; yours to change. Needs: field, button.
//!
//! A form's frame around Fields: a title and description, the fields stacked (or with labels beside them), a status line above the
//! fields once something happened (what failed, in the danger wash, announced; or that it worked, in the success wash), and a footer
//! with Cancel and the submit button, which shows a spinner and stays put while submitting. Fields keep their own label, required or
//! optional mark, description and error (`Field`); this item adds the small checks most forms need ([`required`], [`email`],
//! [`min_len`]) and counts what is wrong for the status line ([`problems`]).
//!
//! ```ignore
//! Form::new("signup", "Create an account").field(name).field(email).status(s.status.clone())
//!     .submit("Create account", |s: &mut App| s.submit()).cancel("Cancel", |s: &mut App| s.reset()).build(t)
//! ```

use rune::{El, LiveRegion, col, row, text};
use rune_ui_style::{ButtonVariant, Tone, weight};

use super::base::{Press, Shape, Theme, glyph, paint, press};
use super::button::Button;
use super::icons;

/// Where the form is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    /// The submit button spins; nothing can be sent twice.
    Submitting,
    /// What went wrong, above the fields.
    Failed(String),
    /// That it worked, above the fields.
    Done(String),
}

/// A message when `value` is empty (after trimming).
pub fn required(value: &str, what: &str) -> Option<String> {
    value.trim().is_empty().then(|| format!("{what} is required"))
}

/// A message when `value` is not shaped like an email address (something, an at sign, a dotted domain).
pub fn email(value: &str) -> Option<String> {
    let v = value.trim();
    let ok = v.split_once('@').is_some_and(|(a, d)| !a.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.') && !v.contains(' '));
    (!v.is_empty() && !ok).then(|| "Enter an email address like name@example.com".to_owned())
}

/// A message when `value` is shorter than `n` characters.
pub fn min_len(value: &str, n: usize, what: &str) -> Option<String> {
    let len = value.trim().chars().count();
    (len > 0 && len < n).then(|| format!("{what} needs at least {n} characters"))
}

/// The status line for a set of field errors: `None` when all are fine, else "2 fields need attention".
pub fn problems(errors: &[Option<String>]) -> Option<String> {
    match errors.iter().filter(|e| e.is_some()).count() {
        0 => None,
        1 => Some("1 field needs attention".to_owned()),
        n => Some(format!("{n} fields need attention")),
    }
}

pub struct Form<S: 'static> {
    key: String,
    title: String,
    description: Option<String>,
    fields: Vec<Vec<El>>,
    status: Status,
    width: f32,
    submit: Option<(String, Press<S>)>,
    cancel: Option<(String, Press<S>)>,
    framed: bool,
}

impl<S: 'static> Form<S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Form<S> {
        Form { key: key.into(), title: title.into(), description: None, fields: Vec::new(), status: Status::Idle, width: 400.0, submit: None, cancel: None, framed: true }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// A field (usually a `Field`), in order.
    pub fn field(mut self, f: El) -> Self {
        self.fields.push(vec![f]);
        self
    }

    /// Fields side by side in a row (first and last name).
    pub fn row(mut self, fields: Vec<El>) -> Self {
        self.fields.push(fields);
        self
    }

    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// The submit button's words and what it does.
    pub fn submit(mut self, words: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.submit = Some((words.into(), press(f)));
        self
    }

    pub fn cancel(mut self, words: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.cancel = Some((words.into(), press(f)));
        self
    }

    /// Drawn on the slab with padding (on by default); off inside a dialog or a sheet.
    pub fn framed(mut self, on: bool) -> Self {
        self.framed = on;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let mut head = col().w_full().gap(t.space.at(1.0)).child(text(self.title.clone()).font(weight(t.ty.lg, t.ty.semibold)).color(t.ink()).role("heading").level(2));
        if let Some(d) = &self.description {
            head = head.child(text(d.clone()).font(t.ty.base).color(t.ink_muted()).wrap().min_w(0.0));
        }
        let mut c = col().key(k.clone()).role("form").label(self.title.clone()).w(self.width).gap(t.metrics.stack).child(head);
        let note = match &self.status {
            Status::Failed(m) => Some((Tone::Danger, &icons::CIRCLE_X, m.clone())),
            Status::Done(m) => Some((Tone::Success, &icons::CIRCLE_CHECK, m.clone())),
            _ => None,
        };
        if let Some((tone, icon, words)) = note {
            let (wash, ink, _, _) = tone.colors(t);
            let line = row().key(format!("{k}-status")).role(if tone == Tone::Danger { "alert" } else { "status" }).label(words.clone()).live_region(if tone == Tone::Danger { LiveRegion::Assertive } else { LiveRegion::Polite }).w_full().px(t.metrics.pad_sm).py(t.space.at(2.5)).gap(t.metrics.gap).center_y().radius(t.radius.lg).round_as(t);
            c = c.child(paint(line, &rune_ui_style::Recipe::new().fill(wash)).child(glyph(icon, t.control.icon, ink)).child(text(words).font(weight(t.ty.base, t.ty.medium)).color(ink).wrap().min_w(0.0).a11y_hidden()));
        }
        for mut line in self.fields {
            c = c.child(if line.len() == 1 { line.remove(0) } else { row().w_full().gap(t.metrics.stack).items_start().children(line.into_iter().map(|f| col().grow(1.0).shrink().w(0.0).child(f))) });
        }
        let busy = self.status == Status::Submitting;
        let mut foot = row().key(format!("{k}-footer")).w_full().gap(t.space.at(2.0)).justify_end().pt(t.space.at(1.0));
        if let Some((words, f)) = self.cancel {
            foot = foot.child(Button::<S>::new(format!("{k}-cancel"), words).variant(ButtonVariant::Ghost).disabled(busy).on_press(move |s: &mut S| f(s)).build(t));
        }
        if let Some((words, f)) = self.submit {
            foot = foot.child(Button::<S>::new(format!("{k}-submit"), words).loading(busy).on_press(move |s: &mut S| f(s)).build(t));
        }
        c = c.child(foot);
        if self.framed {
            paint(col().key(format!("{k}-frame")).padding(t.metrics.panel).radius(t.radius.xxl).round_as(t), &t.mat.slab.base).child(c)
        } else {
            c
        }
    }
}
