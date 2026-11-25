use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::input_otp::InputOtp;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("input_otp", "InputOTP", "form").story("Six digits, four groups of letters, states; paste a code", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let live = InputOtp::<S>::new("otp-demo", "Verification code", g.edit("otp"), move |s: &mut S| acc(s).edit_mut("otp")).on_complete(move |s: &mut S, _| acc(s).presses += 1).build(t);
    let letters = InputOtp::<S>::new("otp-letters", "Recovery code", g.edit("otp-words"), move |s: &mut S| acc(s).edit_mut("otp-words")).groups(&[2, 2, 2, 2]).pattern(|c| c.is_ascii_alphanumeric()).on_change(move |s: &mut S, v: String| acc(s).choose("otp-words", &v.to_uppercase())).build(t);
    let focus = rune::TextEdit::new("4");
    let invalid = rune::TextEdit::new("123456");
    let off = rune::TextEdit::new("");
    specs(t, vec![
        spec(t, "Type, paste, Backspace, arrows", live),
        spec(t, "Letters and digits, four groups", letters),
        spec(t, "Focus", InputOtp::<S>::new("otp-focus", "Code", &focus, move |s: &mut S| acc(s).edit_mut("otp-focus")).state(Look::FocusVisible).build(t)),
        spec(t, "Invalid", InputOtp::<S>::new("otp-invalid", "Code", &invalid, move |s: &mut S| acc(s).edit_mut("otp-invalid")).invalid(true).build(t)),
        spec(t, "Disabled", InputOtp::<S>::new("otp-off", "Code", &off, move |s: &mut S| acc(s).edit_mut("otp-off")).disabled(true).build(t)),
    ])
}
