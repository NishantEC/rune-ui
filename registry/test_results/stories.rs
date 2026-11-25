use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::test_results::{Suite, TestCase, TestResults, TestStatus};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("test_results", "TestResults", "agent").story("Counts, suites that fold, failures inline", all::<S>)
}

/// A placeholder run: one suite with a failure, one all passed, one with a skip.
pub fn suites() -> Vec<Suite> {
    vec![
        Suite::new("billing::invoice", vec![
            TestCase::new("totals_include_tax", TestStatus::Passed, 0.012),
            TestCase::new("rounds_half_up", TestStatus::Failed, 0.004).message("assertion failed: left == right").diff("$12.35", "$12.34"),
            TestCase::new("empty_invoice_is_zero", TestStatus::Passed, 0.001),
        ]),
        Suite::new("ui::table", vec![TestCase::new("sorts_by_cost", TestStatus::Passed, 0.31), TestCase::new("keeps_selection_when_paging", TestStatus::Passed, 0.27)]),
        Suite::new("sync::retry", vec![TestCase::new("backs_off", TestStatus::Passed, 1.8), TestCase::new("gives_up_after_three", TestStatus::Skipped, 0.0)]),
    ]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let s = suites();
    let open: Vec<String> = s.iter().filter(|x| g.flags.get(&format!("tests-{}", x.name)).copied().unwrap_or(x.status() == TestStatus::Failed)).map(|x| x.name.clone()).collect();
    let live = TestResults::<S>::new("tests-demo", &s).open(open).on_toggle(move |st: &mut S, name: &str| {
        let g = acc(st);
        let k = format!("tests-{name}");
        let was = g.flags.get(&k).copied().unwrap_or(name == "billing::invoice");
        g.flags.insert(k, !was);
    }).build(t);
    specs(t, vec![spec_wide(t, "Failed suites first and open; press a suite to fold it", 620.0, live)])
}
