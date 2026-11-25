//! rune-ui registry: icons 0.2.0. A small set of 16 point glyphs on one 1.5 stroke, drawn in `currentColor` so a control recolours
//! them per state. Replace or extend with your app's own set. `rune-ui init` copies it.

use rune::Icon;

macro_rules! glyph {
    ($name:ident, $id:literal, $body:literal) => {
        pub static $name: Icon = Icon { name: concat!("rune-ui-", $id), view: 16.0, body: $body };
    };
}


glyph!(CHECK, "check", r#"<path d="M3.5 8.4 6.6 11.3 12.5 4.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(MINUS, "minus", r#"<path d="M4 8h8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(PLUS, "plus", r#"<path d="M8 3.5v9M3.5 8h9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(CHEVRON_DOWN, "chevron-down", r#"<path d="M4.5 6.4 8 9.9l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CHEVRON_RIGHT, "chevron-right", r#"<path d="M6.4 4.5 9.9 8l-3.5 3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CHEVRON_UP_DOWN, "chevron-up-down", r#"<path d="M5.2 6.2 8 3.6l2.8 2.6M5.2 9.8 8 12.4l2.8-2.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CLOSE, "close", r#"<path d="M4.8 4.8l6.4 6.4M11.2 4.8l-6.4 6.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(SEARCH, "search", r#"<circle cx="7.2" cy="7.2" r="4.2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M10.4 10.4 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(ARROW_RIGHT, "arrow-right", r#"<path d="M3.5 8h8.5M8.8 4.6 12.2 8l-3.4 3.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(TRASH, "trash", r#"<path d="M3.5 4.8h9M6.4 4.6V3.4h3.2v1.2M4.8 4.8l.6 7.8h5.2l.6-7.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(MORE, "more", r#"<circle cx="4" cy="8" r="1.1" fill="currentColor"/><circle cx="8" cy="8" r="1.1" fill="currentColor"/><circle cx="12" cy="8" r="1.1" fill="currentColor"/>"#);
glyph!(INFO, "info", r#"<circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M8 7.3v3.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="8" cy="5.2" r="0.9" fill="currentColor"/>"#);
glyph!(WARNING, "warning", r#"<path d="M8 2.8 13.6 12.6H2.4Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M8 6.6v2.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="8" cy="11" r="0.85" fill="currentColor"/>"#);
glyph!(CIRCLE_CHECK, "circle-check", r#"<circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M5.6 8.2 7.3 9.8 10.4 6.4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#);
glyph!(CIRCLE_X, "circle-x", r#"<circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M6.2 6.2l3.6 3.6M9.8 6.2l-3.6 3.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(COPY, "copy", r#"<rect x="5.5" y="5.5" width="7" height="7" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3.5 10.2V4.6c0-.6.5-1.1 1.1-1.1h5.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(GEAR, "gear", r#"<circle cx="8" cy="8" r="2.2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M8 2.5v1.6M8 11.9v1.6M2.5 8h1.6M11.9 8h1.6M4.1 4.1l1.1 1.1M10.8 10.8l1.1 1.1M4.1 11.9l1.1-1.1M10.8 5.2l1.1-1.1" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(USER, "user", r#"<circle cx="8" cy="5.8" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3.4 13c.6-2.3 2.4-3.6 4.6-3.6s4 1.3 4.6 3.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(HOME, "home", r#"<path d="M2.8 7.2 8 3l5.2 4.2V13H2.8Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>"#);
glyph!(INBOX, "inbox", r#"<path d="M2.5 9.5 4.3 3.5h7.4l1.8 6V12.5h-11Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M2.5 9.5h3l1 1.5h3l1-1.5h3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>"#);
glyph!(FOLDER, "folder", r#"<path d="M2.5 4.5c0-.8.7-1.5 1.5-1.5h2.4L8 4.8h4c.8 0 1.5.7 1.5 1.5v5.2c0 .8-.7 1.5-1.5 1.5H4c-.8 0-1.5-.7-1.5-1.5Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>"#);
glyph!(FILE, "file", r#"<path d="M4 2.5h5l3 3v8H4Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M9 2.5v3h3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>"#);
glyph!(BELL, "bell", r#"<path d="M4.2 11V7.4a3.8 3.8 0 0 1 7.6 0V11l1 1.2H3.2Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M6.6 13.6h2.8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(CALENDAR, "calendar", r#"<rect x="2.5" y="3.5" width="11" height="10" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M5.5 2v2.5M10.5 2v2.5M2.5 6.8h11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(BOLD, "bold", r#"<path d="M5 3h3.6a2.4 2.4 0 0 1 0 4.8H5Zm0 4.8h4.2a2.6 2.6 0 0 1 0 5.2H5Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>"#);
glyph!(ITALIC, "italic", r#"<path d="M7 3h5M4 13h5M9.6 3 6.4 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(UNDERLINE, "underline", r#"<path d="M4.5 2.8v4.4a3.5 3.5 0 0 0 7 0V2.8M3.5 13.5h9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"#);
glyph!(TAG, "tag", r#"<path d="M3 3h4.6L13 8.4 8.4 13 3 7.6Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><circle cx="5.8" cy="5.8" r="0.9" fill="currentColor"/>"#);
glyph!(RING, "ring", r#"<circle cx="8" cy="8" r="5.5" fill="none" stroke="currentColor" stroke-width="1.75"/>"#);
glyph!(ARC, "arc", r#"<path d="M8 2.5A5.5 5.5 0 0 1 13.5 8" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"/>"#);
