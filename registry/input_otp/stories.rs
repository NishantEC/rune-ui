use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::input_otp::InputOtp;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("input_otp", "InputOTP", "form").story("Six digits, four groups of letters, states", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let code = g.choice("otp", "12");
    let live = InputOtp::<S>::new("otp-demo", "Verification code", &code).on_change(move |s: &mut S, v: String| acc(s).choose("otp", &v)).on_complete(move |s: &mut S, _| acc(s).presses += 1).build(t);
    let words = g.choice("otp-words", "AB");
    let letters = InputOtp::<S>::new("otp-letters", "Recovery code", &words).groups(&[2, 2, 2, 2]).pattern(|c| c.is_ascii_alphanumeric()).on_change(move |s: &mut S, v: String| acc(s).choose("otp-words", &v.to_uppercase())).build(t);
    specs(t, vec![
        spec(t, "Type, Backspace, arrows", live),
        spec(t, "Letters and digits, four groups", letters),
        spec(t, "Focus", InputOtp::<S>::new("otp-focus", "Code", "4").state(Look::FocusVisible).build(t)),
        spec(t, "Invalid", InputOtp::<S>::new("otp-invalid", "Code", "123456").invalid(true).build(t)),
        spec(t, "Disabled", InputOtp::<S>::new("otp-off", "Code", "").disabled(true).build(t)),
    ])
}
