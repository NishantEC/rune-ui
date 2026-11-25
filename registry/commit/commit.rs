//! rune-ui registry: commit 0.8.0. Copied into your app by `rune-ui add commit`; yours to change.
//! Needs: diff.
//!
//! One commit as one line that folds: a commit mark, the short hash in a monospaced face, the subject, then who and when and the
//! `+n −n` of the whole change in quiet figures. Folded open it lists the files with their own counts (pressing one hands its path
//! to the app) and, for the files it has a patch for, the diff. Copy puts the full hash on the clipboard.
//!
//! ```ignore
//! Commit::new("c1", &info, s.open).on_toggle(|s: &mut App| s.open = !s.open).patch(diff::parse(&patch))
//!     .on_file(|s: &mut App, path: &str| s.show(path)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Tx, col, row, text};
use rune_ui_style::{tabular, weight};

use super::agent::{PressTx, icon_button, meta, mono, return_presses};
use super::base::{Pick, Press, Shape, State, States, Theme, glyph, hit_area, paint_state};
use super::diff::{Diff, FileDiff};
use super::icons;

/// One file a commit touched.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub added: usize,
    pub removed: usize,
}

impl FileChange {
    pub fn new(path: impl Into<String>, added: usize, removed: usize) -> FileChange {
        FileChange { path: path.into(), added, removed }
    }
}

/// What a commit says about itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitInfo {
    pub hash: String,
    pub subject: String,
    pub author: String,
    /// When, in words the app chose ("2 hours ago", "Nov 25").
    pub time: String,
    pub files: Vec<FileChange>,
}

impl CommitInfo {
    /// The first seven characters of the hash.
    pub fn short(&self) -> &str {
        &self.hash[..self.hash.len().min(7)]
    }

    /// (added, removed) over every file.
    pub fn stats(&self) -> (usize, usize) {
        self.files.iter().fold((0, 0), |(a, r), f| (a + f.added, r + f.removed))
    }
}

pub struct Commit<'a, S: 'static> {
    key: String,
    info: &'a CommitInfo,
    open: bool,
    patch: Vec<FileDiff>,
    toggle: Option<Press<S>>,
    file: Option<Pick<S>>,
    copied: bool,
    copy: Option<Press<S>>,
    pinned: Option<State>,
}

impl<'a, S: 'static> Commit<'a, S> {
    pub fn new(key: impl Into<String>, info: &'a CommitInfo, open: bool) -> Commit<'a, S> {
        Commit { key: key.into(), info, open, patch: Vec::new(), toggle: None, file: None, copied: false, copy: None, pinned: None }
    }

    /// The diff shown under the files while open (`diff::parse` of the commit's patch).
    pub fn patch(mut self, files: Vec<FileDiff>) -> Self {
        self.patch = files;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    /// A file's line was pressed: its path.
    pub fn on_file(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.file = Some(Rc::new(f));
        self
    }

    pub fn copied(mut self, on: bool) -> Self {
        self.copied = on;
        self
    }

    /// After the hash is copied.
    pub fn on_copy(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.copy = Some(Rc::new(f));
        self
    }

    /// Pins a live state on the line (galleries).
    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    fn counts(t: &Theme, added: usize, removed: usize) -> El {
        let style = tabular(t.ty.sm);
        row().gap(t.space.at(1.5)).child(text(format!("+{added}")).font(style).color(t.c.success.fg).a11y_hidden()).child(text(format!("−{removed}")).font(style).color(t.c.danger.fg).a11y_hidden())
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let info = self.info;
        let (added, removed) = info.stats();
        let h = t.control.md;
        let trigger = format!("{k}-trigger");
        let st = super::base::states(self.pinned, false, false);
        let label = format!("Commit {}: {}", info.short(), info.subject);
        let mut face = row().key(trigger.clone()).role("button").label(label.clone()).expanded(self.open).focusable(self.toggle.is_some()).grow(1.0).shrink().min_w(0.0).h(h).pl(t.space.at(1.5)).pr(t.space.at(2.0)).gap(t.metrics.gap).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t);
        face = paint_state(face, &t.mat.row, st);
        face = face.child(glyph(&icons::COMMIT, t.control.icon, t.ink_faint()))
            .child(text(info.short().to_owned()).font(mono(t, t.ty.sm)).color(t.ink_muted()).a11y_hidden())
            .child(text(info.subject.clone()).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).truncate().grow(1.0).shrink().min_w(0.0).a11y_hidden())
            .child(meta(t, format!("{} · {}", info.author, info.time)).truncate().shrink().min_w(0.0).a11y_hidden())
            .child(Self::counts(t, added, removed))
            .child(glyph(if self.open { &icons::CHEVRON_DOWN } else { &icons::CHEVRON_RIGHT }, t.control.icon_sm, t.ink_faint()));
        let press = self.toggle.clone();
        if let Some(f) = press.clone() {
            let g = f.clone();
            face = face.on_press(move |s: &mut S| f(s)).on_key(move |s: &mut S, ev: &rune::KeyEvent, _: &mut Tx| return_presses(ev, || g(s)));
        }
        let hash = info.hash.clone();
        let then = self.copy.clone();
        let copy: PressTx<S> = Rc::new(move |s: &mut S, tx: &mut Tx| {
            tx.copy(hash.clone());
            if let Some(f) = &then {
                f(s);
            }
        });
        let line = row().w_full().gap(t.space.at(1.0)).center_y().child(hit_area(&trigger, face, 0.0, t.control.slop(h), press).grow(1.0).shrink().min_w(0.0)).child(icon_button(t, &format!("{k}-copy"), if self.copied { "Copied" } else { "Copy hash" }, if self.copied { &icons::CHECK } else { &icons::COPY }, None, false, Some(copy)));
        let mut c = col().key(k.clone()).role("group").label(label).w_full().gap(t.space.at(1.0)).child(line);
        if self.open {
            let indent = t.space.at(1.5) + t.control.icon + t.metrics.gap;
            let mut files = col().key(format!("{k}-files")).role("list").label("Files changed").w_full().gap(2.0 * t.control.slop(t.control.sm));
            for (i, f) in info.files.iter().enumerate() {
                let key = format!("{k}-file-{i}");
                let hs = t.control.sm;
                let mut r = row().key(key.clone()).role(if self.file.is_some() { "button" } else { "listitem" }).label(format!("{}, {} added, {} removed", f.path, f.added, f.removed)).focusable(self.file.is_some()).w_full().h(hs).px(t.space.at(2.0)).gap(t.metrics.gap).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, hs)).round_as(t);
                r = paint_state(r, &t.mat.row, States::default()).child(text(f.path.clone()).font(mono(t, t.ty.sm)).color(t.ink_muted()).truncate().grow(1.0).shrink().min_w(0.0).a11y_hidden()).child(Self::counts(t, f.added, f.removed));
                let on_file: Option<Press<S>> = self.file.clone().map(|cb| {
                    let path = f.path.clone();
                    let p: Press<S> = Rc::new(move |s: &mut S| cb(s, &path));
                    p
                });
                if let Some(p) = on_file.clone() {
                    let q = p.clone();
                    r = r.on_press(move |s: &mut S| p(s)).on_key(move |s: &mut S, ev: &rune::KeyEvent, _: &mut Tx| return_presses(ev, || q(s)));
                }
                files = files.child(hit_area(&key, r, 0.0, t.control.slop(hs), on_file));
            }
            let mut body = col().key(format!("{k}-content")).w_full().pl(indent).gap(t.space.at(2.0)).child(files);
            for (i, d) in self.patch.iter().enumerate() {
                body = body.child(Diff::<S>::new(format!("{k}-diff-{i}"), d, true).build(t));
            }
            c = c.child(body);
        }
        c
    }
}
