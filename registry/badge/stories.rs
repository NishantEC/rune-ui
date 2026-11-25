use rune_ui_style::Tone;

use super::super::badge::{Badge, BadgeVariant};
use super::super::gallery::{Ctx, StoryItem, labelled, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("badge", "Badge", "display").story("Variants, meanings, dots", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let variants = [BadgeVariant::Default, BadgeVariant::Secondary, BadgeVariant::Outline, BadgeVariant::Destructive];
    let words = ["Default", "Secondary", "Outline", "Destructive"];
    let v: Vec<rune::El> = variants.iter().zip(words).map(|(v, w)| spec(t, w, Badge::new(format!("badge-{}", w.to_lowercase()), "Badge").variant(*v).build(t))).collect();
    let tones: Vec<rune::El> = Tone::ALL.iter().map(|tone| spec(t, tone.word(), Badge::new(format!("badge-tone-{}", tone.word().to_lowercase()), tone.word()).tone(*tone).dot().build(t))).collect();
    let extra = vec![spec(t, "Count", Badge::new("badge-count", "12").build(t)), spec(t, "Draft (hollow)", Badge::new("badge-draft", "Draft").hollow().build(t)), spec(t, "Loud count", Badge::new("badge-loud", "99+").variant(BadgeVariant::Destructive).build(t))];
    rune::col().gap(t.space.at(4.0)).child(labelled(t, "Variants", specs(t, v))).child(labelled(t, "Meanings", specs(t, tones))).child(labelled(t, "Other", specs(t, extra)))
}
