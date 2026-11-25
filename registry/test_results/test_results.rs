//! rune-ui registry: test_results 0.8.0. Copied into your app by `rune-ui add test_results`; yours to change.
//!
//! A test run: a summary line of counts (failed, passed, skipped, still running) and the time, then each suite as a quiet line that
//! folds, failed suites first. Inside a suite the failed tests come first, each with its mark, its name and its time; a failed test
//! shows its message and, when it has them, the expected and received values on the danger and success washes. Colour is state only:
//! the counts are words with a small dot, never colour alone.
//!
//! ```ignore
//! TestResults::new("run", &suites).open(s.open_suites.iter().cloned()).on_toggle(|s: &mut App, suite: &str| s.flip(suite)).build(t)
//! ```

use std::collections::BTreeSet;
use std::rc::Rc;

use rune::{El, col, el, row, text};
use rune_ui_style::{tabular, weight};

use super::agent::{FoldLine, Status, elapsed, mark, meta, mono};

/// A test's time: milliseconds under a second ("4ms"), the agent set's words above ("1.8s").
pub fn took(secs: f64) -> String {
    if secs < 1.0 { format!("{}ms", (secs * 1000.0).round().max(if secs > 0.0 { 1.0 } else { 0.0 })) } else { elapsed(secs) }
}
use super::base::{Pick, Theme, Tone};

/// Where a test stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TestStatus {
    #[default]
    Passed,
    Failed,
    Skipped,
    Running,
}

impl TestStatus {
    /// The agent set's status for its mark.
    pub fn status(self) -> Status {
        match self {
            TestStatus::Passed => Status::Done,
            TestStatus::Failed => Status::Error,
            TestStatus::Skipped => Status::Skipped,
            TestStatus::Running => Status::Running,
        }
    }

    /// Failed first, then running, passed, skipped.
    fn rank(self) -> u8 {
        match self {
            TestStatus::Failed => 0,
            TestStatus::Running => 1,
            TestStatus::Passed => 2,
            TestStatus::Skipped => 3,
        }
    }
}

/// One test.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TestCase {
    pub name: String,
    pub status: TestStatus,
    pub secs: f64,
    /// Why it failed.
    pub message: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

impl TestCase {
    pub fn new(name: impl Into<String>, status: TestStatus, secs: f64) -> TestCase {
        TestCase { name: name.into(), status, secs, ..TestCase::default() }
    }

    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = Some(m.into());
        self
    }

    /// What the test wanted and what it got.
    pub fn diff(mut self, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self.actual = Some(actual.into());
        self
    }
}

/// A group of tests (a file, a module).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Suite {
    pub name: String,
    pub tests: Vec<TestCase>,
}

impl Suite {
    pub fn new(name: impl Into<String>, tests: Vec<TestCase>) -> Suite {
        Suite { name: name.into(), tests }
    }

    pub fn count(&self, s: TestStatus) -> usize {
        self.tests.iter().filter(|t| t.status == s).count()
    }

    pub fn secs(&self) -> f64 {
        self.tests.iter().map(|t| t.secs).sum()
    }

    /// The suite's own status: failed if any failed, running if any run, skipped if all were, passed otherwise.
    pub fn status(&self) -> TestStatus {
        if self.count(TestStatus::Failed) > 0 {
            TestStatus::Failed
        } else if self.count(TestStatus::Running) > 0 {
            TestStatus::Running
        } else if !self.tests.is_empty() && self.count(TestStatus::Skipped) == self.tests.len() {
            TestStatus::Skipped
        } else {
            TestStatus::Passed
        }
    }
}

/// The counts in words: "2 failed · 41 passed · 1 skipped".
pub fn summary(suites: &[Suite]) -> Vec<(TestStatus, usize)> {
    [TestStatus::Failed, TestStatus::Running, TestStatus::Passed, TestStatus::Skipped].iter().map(|s| (*s, suites.iter().map(|x| x.count(*s)).sum())).filter(|(_, n)| *n > 0).collect()
}

pub struct TestResults<'a, S: 'static> {
    key: String,
    suites: &'a [Suite],
    open: Option<BTreeSet<String>>,
    toggle: Option<Pick<S>>,
}

impl<'a, S: 'static> TestResults<'a, S> {
    pub fn new(key: impl Into<String>, suites: &'a [Suite]) -> TestResults<'a, S> {
        TestResults { key: key.into(), suites, open: None, toggle: None }
    }

    /// The suites folded open, by name (otherwise the failed ones).
    pub fn open(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.open = Some(names.into_iter().collect());
        self
    }

    /// A suite's line was pressed.
    pub fn on_toggle(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    fn word(s: TestStatus) -> &'static str {
        match s {
            TestStatus::Failed => "failed",
            TestStatus::Running => "running",
            TestStatus::Passed => "passed",
            TestStatus::Skipped => "skipped",
        }
    }

    fn case(t: &Theme, key: &str, c: &TestCase) -> El {
        let line = row().key(key.to_owned()).role("listitem").label(format!("{}, {}", c.name, Self::word(c.status))).w_full().min_h(t.control.sm).gap(t.metrics.gap).center_y()
            .child(mark(t, c.status.status(), t.control.icon_sm))
            .child(text(c.name.clone()).font(mono(t, t.ty.sm)).color(if c.status == TestStatus::Skipped { t.ink_faint() } else { t.ink() }).truncate().grow(1.0).shrink().min_w(0.0).a11y_hidden())
            .child(meta(t, took(c.secs)).a11y_hidden());
        let mut out = col().w_full().gap(t.space.at(1.0)).child(line);
        if c.status != TestStatus::Failed {
            return out;
        }
        let indent = t.control.icon_sm + t.metrics.gap;
        if let Some(m) = &c.message {
            out = out.child(text(m.clone()).key(format!("{key}-message")).font(t.ty.sm).color(t.c.danger.fg).wrap().min_w(0.0).pl(indent));
        }
        if let (Some(want), Some(got)) = (&c.expected, &c.actual) {
            let style = mono(t, t.ty.sm);
            let line = |sign: &str, words: &str, wash, ink, k: &str| row().key(format!("{key}-{k}")).w_full().px(t.space.at(2.0)).py(t.space.at(0.5)).gap(t.space.at(2.0)).fill(wash).child(text(sign.to_owned()).font(style).color(ink).a11y_hidden()).child(text(words.to_owned()).font(style).color(t.ink()).wrap().grow(1.0).shrink().min_w(0.0));
            let diff = col().key(format!("{key}-diff")).role("group").label("Expected and received").w_full().radius(t.radius.sm).clip_rounded(t.radius.sm)
                .child(line("−", &format!("expected {want}"), t.c.danger.bg, t.c.danger.fg, "expected"))
                .child(line("+", &format!("received {got}"), t.c.success.bg, t.c.success.fg, "actual"));
            out = out.child(row().w_full().pl(indent).child(diff));
        }
        out
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let total: f64 = self.suites.iter().map(Suite::secs).sum();
        let counts = summary(self.suites);
        let mut head = row().key(format!("{k}-summary")).role("status").label(counts.iter().map(|(s, n)| format!("{n} {}", Self::word(*s))).collect::<Vec<_>>().join(", ")).w_full().gap(t.space.at(4.0)).center_y().flex_wrap();
        for (s, n) in &counts {
            let tone = match s {
                TestStatus::Failed => Tone::Danger,
                TestStatus::Passed => Tone::Success,
                TestStatus::Running => Tone::Info,
                TestStatus::Skipped => Tone::Neutral,
            };
            let (_, ink, solid, _) = tone.colors(t);
            head = head.child(row().gap(t.space.at(1.5)).center_y().child(el().size(6.0, 6.0).radius(3.0).fill(solid).a11y_hidden()).child(text(format!("{n} {}", Self::word(*s))).font(weight(tabular(t.ty.base), t.ty.medium)).color(if *s == TestStatus::Skipped { t.ink_muted() } else { ink }).a11y_hidden()));
        }
        head = head.child(row().grow(1.0)).child(meta(t, format!("{} in {}", plural(self.suites.iter().map(|s| s.tests.len()).sum(), "test"), took(total))).a11y_hidden());
        let mut suites: Vec<&Suite> = self.suites.iter().collect();
        suites.sort_by_key(|s| s.status().rank());
        let mut list = col().key(format!("{k}-suites")).w_full().gap(t.space.at(1.0));
        for (i, s) in suites.iter().enumerate() {
            let open = match &self.open {
                Some(set) => set.contains(&s.name),
                None => s.status() == TestStatus::Failed,
            };
            let failed = s.count(TestStatus::Failed);
            let figures = if failed > 0 { format!("{failed} of {} failed", s.tests.len()) } else { plural(s.tests.len(), "test") };
            let mut line = FoldLine::<S>::new(format!("{k}-suite-{i}"), s.name.clone(), open).lead(mark(t, s.status().status(), t.control.icon)).meta(figures).meta(took(s.secs()));
            if let Some(f) = self.toggle.clone() {
                let name = s.name.clone();
                line = line.on_toggle(move |st: &mut S| f(st, &name));
            }
            let mut cases: Vec<&TestCase> = s.tests.iter().collect();
            cases.sort_by_key(|c| c.status.rank());
            let body = vec![col().key(format!("{k}-suite-{i}-tests")).role("list").label(s.name.clone()).w_full().gap(t.space.at(1.5)).children(cases.iter().enumerate().map(|(j, c)| Self::case(t, &format!("{k}-suite-{i}-test-{j}"), c)).collect::<Vec<_>>())];
            list = list.child(line.build(t, body));
        }
        col().key(k).role("group").label("Test results").w_full().gap(t.space.at(3.0)).child(head).child(list)
    }
}

fn plural(n: usize, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
}
