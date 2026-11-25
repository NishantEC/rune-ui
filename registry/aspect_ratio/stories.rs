use rune::{El, col, text};
use rune_ui_style::{Recipe, paint};

use super::super::aspect_ratio::AspectRatio;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("aspect_ratio", "AspectRatio", "structure").story("16:9, 1:1, 4:3", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let fill = |words: &str| paint(col().center(), &Recipe::new().linear(150.0, &[(0.0, t.c.muted.bg), (1.0, t.c.accent.bg)])).child(text(words.to_owned()).font(t.ty.base).color(t.ink_muted()));
    specs(t, vec![
        spec(t, "16:9 at 320 wide", col().w(320.0).child(AspectRatio::new("ratio-wide", 16.0 / 9.0).child(fill("16:9")).build(t))),
        spec(t, "1:1 at 160", col().w(160.0).child(AspectRatio::new("ratio-square", 1.0).child(fill("1:1")).build(t))),
        spec(t, "4:3, square corners", col().w(200.0).child(AspectRatio::new("ratio-photo", 4.0 / 3.0).square().child(fill("4:3")).build(t))),
    ])
}
