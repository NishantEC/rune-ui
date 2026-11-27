use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::kbd::Kbd;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("kbd", "Kbd", "display").story("Keys", all::<S>).story("Modifier glyphs", glyphs::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![spec(t, "One key", Kbd::new("kbd-one", "Esc").build(t)), spec(t, "A chord", Kbd::new("kbd-chord", "⌘ K").build(t)), spec(t, "Three keys", Kbd::new("kbd-three", "Ctrl Shift P").build(t))])
}

/// ⌘ ⌥ ⇧ ⌃ ↩ are drawn as icons, so no font has to carry them.
fn glyphs<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![
        spec(t, "Command and a letter", Kbd::new("kbd-glyph-cmd", "⌘ K").build(t)),
        spec(t, "Run together", Kbd::new("kbd-glyph-run", "⇧⌘P").build(t)),
        spec(t, "Option and return", Kbd::new("kbd-glyph-return", "⌥ ↩").build(t)),
        spec(t, "All four modifiers", Kbd::new("kbd-glyph-all", "⌃ ⌥ ⇧ ⌘ T").build(t)),
        spec(t, "With words (kept as words)", Kbd::new("kbd-glyph-words", "⌘ Enter").build(t)),
    ])
}
