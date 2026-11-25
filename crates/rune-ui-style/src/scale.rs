//! The scales: space, type, radius, shadow and motion. Like Tailwind's, each is a short ladder of named
//! steps so a screen picks a step instead of a number. The values are rune-ui's own.

use rune::{Color, Shadow, TextStyle};

/// Space on a 4 point grid with 2 point half steps at the small end. `t.space.at(3.0)` is 12, as
/// Tailwind's `p-3` is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space {
    pub unit: f32,
}

impl Space {
    pub const DEFAULT: Space = Space { unit: 4.0 };

    /// `steps` units (half steps allowed: `at(1.5)` is 6).
    pub fn at(&self, steps: f32) -> f32 {
        self.unit * steps
    }
}

/// The type ramp: one sans, three weights (400 body, 500 controls and labels, 600 headings), line
/// heights on the 4 point grid where they can be, headings tracked a little tighter than body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    /// 11/15: badges, counts.
    pub xs: TextStyle,
    /// 12/16: chips, captions, hints.
    pub sm: TextStyle,
    /// 13/18: controls, rows, menus.
    pub base: TextStyle,
    /// 14/21: reading text.
    pub md: TextStyle,
    /// 16/22: a card's title.
    pub lg: TextStyle,
    /// 20/26: a section or page title.
    pub xl: TextStyle,
    /// 26/32: a page headline.
    pub xxl: TextStyle,
    pub regular: f32,
    pub medium: f32,
    pub semibold: f32,
}

impl TypeScale {
    pub const DEFAULT: TypeScale = TypeScale {
        xs: TextStyle::new(11.0, 400.0).lh(15.0),
        sm: TextStyle::new(12.0, 400.0).lh(16.0),
        base: TextStyle::new(13.0, 400.0).lh(18.0),
        md: TextStyle::new(14.0, 400.0).lh(21.0),
        lg: TextStyle::new(16.0, 600.0).lh(22.0).tracking(-0.1),
        xl: TextStyle::new(20.0, 600.0).lh(26.0).tracking(-0.25),
        xxl: TextStyle::new(26.0, 600.0).lh(32.0).tracking(-0.5),
        regular: 400.0,
        medium: 500.0,
        semibold: 600.0,
    };
}

/// Figures of one width, for counts, money and times that change in place.
pub fn tabular(style: TextStyle) -> TextStyle {
    TextStyle { features: "'tnum' 1", ..style }
}

/// `style` at another weight.
pub fn weight(style: TextStyle, w: f32) -> TextStyle {
    TextStyle { weight: w, ..style }
}

/// A graduated radius ladder. Steps, not one constant, because nesting needs them: an inner radius is
/// the outer one minus the inset ([`Radius::inner`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    /// Fully round (pills, dots, thumbs): larger than any half height.
    pub full: f32,
}

/// Past this share of a control's height a continuous corner stops reading as a rounded rectangle and
/// starts reading as a pill; [`Radius::fit`] keeps a radius under it.
pub const MAX_CORNER_SHARE: f32 = 0.38;

impl Radius {
    pub const DEFAULT: Radius = Radius { xs: 3.0, sm: 5.0, md: 8.0, lg: 11.0, xl: 14.0, xxl: 18.0, full: 999.0 };

    /// Concentric nesting: the radius of something `inset` points inside a shape of radius `outer`.
    pub fn inner(outer: f32, inset: f32) -> f32 {
        (outer - inset).max(0.0)
    }

    /// `r`, or less, so it stays a rounded rectangle on a shape `height` points high.
    pub fn fit(r: f32, height: f32) -> f32 {
        r.min(height * MAX_CORNER_SHARE)
    }
}

/// Elevations as stacks of faint shadows rather than one hard one. Each stack is made by
/// [`smooth_stack`]: offsets and blurs grow geometrically and the alpha is split across the layers,
/// so the shadow fades out the way light does.
#[derive(Clone, Debug, PartialEq)]
pub struct Elevation {
    /// A contact shadow: what sits on the page (a raised control).
    pub xs: Vec<Shadow>,
    /// A card that lifts a little.
    pub sm: Vec<Shadow>,
    /// A menu, a select's list, a popover.
    pub md: Vec<Shadow>,
    /// A dialog or sheet.
    pub lg: Vec<Shadow>,
}

impl Elevation {
    /// The ladder in `shade` (black, usually), with `strength` scaling every alpha (dark themes need more).
    pub fn new(shade: Color, strength: f32) -> Elevation {
        let k = strength;
        Elevation {
            xs: smooth_stack(shade, 1.0, 2.0, 0.07 * k, 2),
            sm: smooth_stack(shade, 4.0, 10.0, 0.08 * k, 3),
            md: smooth_stack(shade, 10.0, 26.0, 0.12 * k, 4),
            lg: smooth_stack(shade, 22.0, 52.0, 0.18 * k, 5),
        }
    }
}

/// `layers` shadows whose largest is `y` down with a `blur` (CSS blur, `2σ`), each one half the size of
/// the next, sharing `alpha` (the small, close layers darker, so the edge stays defined while the far
/// ones only tint). Never more than Rune springs smoothly per element when it changes (`layers <= 6`).
pub fn smooth_stack(shade: Color, y: f32, blur: f32, alpha: f32, layers: usize) -> Vec<Shadow> {
    let n = layers.clamp(1, 6);
    // Weights 1, 1/2, 1/3 ... from the closest layer out, normalised to `alpha`.
    let weights: Vec<f32> = (0..n).map(|i| 1.0 / (i as f32 + 1.0)).collect();
    let total: f32 = weights.iter().sum();
    (0..n)
        .map(|i| {
            // Closest first: the last layer is the full size.
            let f = 0.5f32.powi((n - 1 - i) as i32);
            Shadow { x: 0.0, y: y * f, blur: blur * f, spread: 0.0, color: shade.alpha(alpha * weights[i] / total), inset: false }
        })
        .collect()
}

/// Motion tokens. Short, caused by the person, quiet; one ease-out curve; nothing loops but work in
/// progress. Rune drives hover with its own eased transition and press with a spring; these are the
/// values a component asks for when it animates something itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// A hover wash, a colour change.
    pub fast_ms: f32,
    /// Something opening (a menu, a popover).
    pub base_ms: f32,
    /// Something large moving (a sheet).
    pub slow_ms: f32,
    /// The one ease-out curve, as CSS cubic-bezier control points.
    pub ease_out: [f32; 4],
    /// What a pressed control shrinks to.
    pub press_scale: f32,
    /// What a popup grows from as it opens.
    pub pop_scale: f32,
    /// How far a popup drifts toward its trigger as it opens, in points.
    pub drift: f32,
}

impl Motion {
    pub const DEFAULT: Motion = Motion { fast_ms: 120.0, base_ms: 200.0, slow_ms: 300.0, ease_out: [0.2, 0.8, 0.2, 1.0], press_scale: 0.96, pop_scale: 0.98, drift: 4.0 };
}

/// The sizes controls are built on: three heights, the hit area each tops up to, icon sizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Controls {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    /// Everything pressable is at least this big to the pointer (an invisible margin makes up the rest).
    pub hit: f32,
    pub icon_sm: f32,
    pub icon: f32,
    pub icon_lg: f32,
    /// The opacity of a disabled control.
    pub disabled: f32,
    /// The focus ring's width and its gap outside the control.
    pub ring: f32,
    pub ring_offset: f32,
}

impl Controls {
    pub const DEFAULT: Controls = Controls { sm: 26.0, md: 30.0, lg: 36.0, hit: 40.0, icon_sm: 12.0, icon: 14.0, icon_lg: 16.0, disabled: 0.45, ring: 2.0, ring_offset: 2.0 };

    /// How far a control `size` points tall falls short of the hit area on each side.
    pub fn slop(&self, size: f32) -> f32 {
        ((self.hit - size) / 2.0).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_is_a_four_point_grid() {
        assert_eq!(Space::DEFAULT.at(3.0), 12.0);
        assert_eq!(Space::DEFAULT.at(1.5), 6.0);
    }

    #[test]
    fn radii_nest_and_never_turn_a_control_into_a_pill() {
        assert_eq!(Radius::inner(11.0, 3.0), 8.0);
        assert_eq!(Radius::inner(2.0, 3.0), 0.0);
        let r = Radius::DEFAULT;
        for h in [Controls::DEFAULT.sm, Controls::DEFAULT.md, Controls::DEFAULT.lg] {
            assert!(Radius::fit(r.md, h) <= h * MAX_CORNER_SHARE);
        }
        assert!(r.xs < r.sm && r.sm < r.md && r.md < r.lg && r.lg < r.xl && r.xl < r.xxl);
    }

    #[test]
    fn a_smooth_stack_grows_outward_and_keeps_its_total_alpha() {
        let s = smooth_stack(Color::BLACK, 16.0, 40.0, 0.2, 4);
        assert_eq!(s.len(), 4);
        assert!(s.windows(2).all(|w| w[0].y < w[1].y && w[0].blur < w[1].blur), "closest layer first");
        assert!(s.windows(2).all(|w| w[0].color.a > w[1].color.a), "closest layer darkest");
        let total: f32 = s.iter().map(|l| l.color.a).sum();
        assert!((total - 0.2).abs() < 1e-5);
        assert_eq!(s.last().unwrap().y, 16.0);
    }

    #[test]
    fn controls_top_up_to_the_hit_area() {
        let c = Controls::DEFAULT;
        assert_eq!(c.slop(30.0), 5.0);
        assert_eq!(c.slop(44.0), 0.0);
    }
}
