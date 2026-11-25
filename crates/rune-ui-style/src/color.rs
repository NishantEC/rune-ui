//! Colour helpers. Rune keeps colours in linear light; these mix in sRGB, as a browser and the window's
//! own blending do, so a 4% wash looks like a 4% wash.

use rune::Color;
use rune::color::{srgb_decode, srgb_encode};

/// An sRGB hex colour (`#rgb`, `#rrggbb` or `#rrggbbaa`). Theme data is written in hex; a typo is a bug, so it panics with the value.
pub fn hex(s: &str) -> Color {
    rune::color::hex(s).unwrap_or_else(|| panic!("rune-ui: not a hex colour: {s:?}"))
}

/// `a` and `b` mixed in sRGB, `t` of the way to `b` (alpha mixes linearly).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let m = |x: f32, y: f32| srgb_decode(srgb_encode(x) * (1.0 - t) + srgb_encode(y) * t);
    Color { r: m(a.r, b.r), g: m(a.g, b.g), b: m(a.b, b.b), a: a.a * (1.0 - t) + b.a * t }
}

/// `top` (straight alpha) laid over `under`, mixed in sRGB: what a wash looks like once painted.
pub fn over(under: Color, top: Color) -> Color {
    if under.a <= 0.0 {
        return top;
    }
    let solid = mix(Color { a: 1.0, ..under }, Color { a: 1.0, ..top }, top.a);
    Color { a: under.a + (1.0 - under.a) * top.a, ..solid }
}

/// WCAG contrast ratio of two opaque colours (1 to 21).
pub fn contrast(a: Color, b: Color) -> f32 {
    let (x, y) = (a.luminance(), b.luminance());
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// Perceived lightness in sRGB (0 black to 1 white): enough to say which of two colours is lighter.
pub fn lightness(c: Color) -> f32 {
    0.2126 * srgb_encode(c.r) + 0.7152 * srgb_encode(c.g) + 0.0722 * srgb_encode(c.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_reads_srgb_and_alpha() {
        let c = hex("#80808040");
        assert!((srgb_encode(c.r) - 128.0 / 255.0).abs() < 1e-3);
        assert!((c.a - 64.0 / 255.0).abs() < 1e-3);
    }

    #[test]
    fn a_wash_over_white_darkens_by_its_share_in_srgb() {
        let got = over(Color::WHITE, Color::BLACK.alpha(0.1));
        assert!((srgb_encode(got.r) - 0.9).abs() < 1e-4 && got.a == 1.0);
        assert_eq!(over(Color::TRANSPARENT, Color::BLACK.alpha(0.1)).a, 0.1);
    }

    #[test]
    fn contrast_is_symmetric_and_bounded() {
        assert!((contrast(Color::WHITE, Color::BLACK) - 21.0).abs() < 0.01);
        assert_eq!(contrast(Color::WHITE, Color::BLACK), contrast(Color::BLACK, Color::WHITE));
    }
}
