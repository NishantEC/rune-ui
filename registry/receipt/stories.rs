use rune::El;

use super::super::base::{ButtonVariant, Size};
use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::portrait::{Activity, Portrait};
use super::super::receipt::{Receipt, ReceiptFile};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("receipt", "Receipt", "agent")
        .story("A finished run: totals, files, branch, actions", all::<S>)
        .story("Only the figures that are not zero; more than five files; no foot", edge::<S>)
}

fn files() -> Vec<ReceiptFile> {
    vec![
        ReceiptFile::new("src/importer/retry.rs", 38, 6),
        ReceiptFile::new("src/importer/mod.rs", 4, 0),
        ReceiptFile::new("docs/old-notes.md", 0, 22).note("removed"),
        ReceiptFile::new("assets/logo.png", 0, 0).note("binary"),
        ReceiptFile::new("tests/importer/gives_up_after_three_attempts_with_a_very_long_name.rs", 14, 2),
    ]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let lead = Portrait::new("receipt-portrait", "Asha", 24.0).state(Activity::None).decorative(true).build(t);
    let review = Button::<S>::new("receipt-review", "Review").variant(ButtonVariant::Secondary).size(Size::Sm).on_press(|_: &mut S| {}).build(t);
    let undo = Button::<S>::new("receipt-undo", "Undo").variant(ButtonVariant::Ghost).size(Size::Sm).on_press(|_: &mut S| {}).build(t);
    let card = Receipt::<S>::new("receipt", "Fixed the login test").lead(lead).totals(56, 30).files(&files()).branch("fix/login-test").action(review).action(undo).on_file(|_: &mut S, _: &str| {}).build(t);
    specs(t, vec![spec_wide(t, "Header, files in a well (paths cut in the middle), branch and actions", 560.0, card)])
}

fn edge<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let grew = Receipt::<S>::new("receipt-grew", "Added the retry module").totals(42, 0).files(&[ReceiptFile::new("src/retry.rs", 42, 0)]).build(t);
    let many: Vec<ReceiptFile> = (0..8).map(|i| ReceiptFile::new(format!("src/part{i}.rs"), i + 1, i % 3)).collect();
    let more = Receipt::<S>::new("receipt-more", "Split the importer").totals(36, 8).files(&many).note("Review before merging: the retry timing changed.").branch("split-importer").build(t);
    let bare = Receipt::<S>::new("receipt-bare", "Looked around, changed nothing").build(t);
    specs(t, vec![spec_wide(t, "Only added", 560.0, grew), spec_wide(t, "Eight files: five, then \"and 3 more\"; a note under the header", 560.0, more), spec_wide(t, "Nothing changed: just the sentence", 560.0, bare)])
}
