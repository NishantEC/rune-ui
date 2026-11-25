use rune_ui_style::{ButtonVariant, Size, Tone};

use super::super::alert::Alert;
use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("alert", "Alert", "display").story("Meanings", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let words = [
        (Tone::Neutral, "A new version is ready", "Restart to use it."),
        (Tone::Info, "Sync runs every hour", "Change it in Settings."),
        (Tone::Success, "Placeholder project published", "Everyone with the link can see it."),
        (Tone::Warning, "Almost out of credits", "Work pauses at the limit."),
        (Tone::Danger, "The last run failed", "Two tests did not pass."),
    ];
    let mut col = rune::col().w_full().max_w(520.0).gap(t.space.at(3.0));
    for (tone, title, desc) in words {
        let mut a = Alert::new(format!("alert-{}", tone.word().to_lowercase()), tone, title).description(desc);
        if tone == Tone::Danger {
            a = a.action(Button::<S>::new("alert-retry", "Retry").variant(ButtonVariant::Secondary).size(Size::Sm).build(t));
        }
        col = col.child(a.build(t));
    }
    col
}
