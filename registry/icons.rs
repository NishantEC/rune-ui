//! rune-ui registry: icons 0.1.0. A small set of 16 point glyphs drawn on one 1.5 stroke, in `currentColor` so a control recolours
//! them per state. Replace or extend with your app's own set. Copied into your app by `rune-ui init`.

use rune::Icon;

macro_rules! glyph {
    ($name:ident, $id:literal, $body:literal) => {
        pub static $name: Icon = Icon { name: concat!("rune-ui-", $id), view: 16.0, body: $body };
    };
}

glyph!(CHECK, "check", r#"<path d="M3.5 8.4 6.6 11.3 12.5 4.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CHEVRON_DOWN, "chevron-down", r#"<path d="M4.5 6.4 8 9.9l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CLOSE, "close", r#"<path d="M4.8 4.8l6.4 6.4M11.2 4.8l-6.4 6.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(SEARCH, "search", r#"<circle cx="7.2" cy="7.2" r="4.2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M10.4 10.4 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(PLUS, "plus", r#"<path d="M8 3.5v9M3.5 8h9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(ARROW_RIGHT, "arrow-right", r#"<path d="M3.5 8h8.5M8.8 4.6 12.2 8l-3.4 3.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(TRASH, "trash", r#"<path d="M3.5 4.8h9M6.4 4.6V3.4h3.2v1.2M4.8 4.8l.6 7.8h5.2l.6-7.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(TAG, "tag", r#"<path d="M3 3h4.6L13 8.4 8.4 13 3 7.6Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><circle cx="5.8" cy="5.8" r="0.9" fill="currentColor"/>"#);
glyph!(RING, "ring", r#"<circle cx="8" cy="8" r="5.5" fill="none" stroke="currentColor" stroke-width="1.75"/>"#);
glyph!(ARC, "arc", r#"<path d="M8 2.5A5.5 5.5 0 0 1 13.5 8" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"/>"#);
