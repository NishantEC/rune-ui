use rune::{El, Pixels, col, row, text};
use rune_ui_style::weight;

use super::super::base::glyph;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::portrait::{self, Activity, Portrait};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("portrait", "Portrait", "display")
        .story("Initials from 20 to 112 points", sizes::<S>)
        .story("A picture and a drawn face", faces::<S>)
        .story("The ring and the dot: every state takes the same room", states::<S>)
}

thread_local! {
    /// A picture made of arithmetic, so the story needs no file.
    static PICTURE: Pixels = {
        let p = Pixels::new();
        let n = 96u32;
        let mut rgba = Vec::with_capacity((n * n * 4) as usize);
        for y in 0..n {
            for x in 0..n {
                let (fx, fy) = (x as f32 / n as f32, y as f32 / n as f32);
                let d = ((fx - 0.5).powi(2) + (fy - 0.62).powi(2)).sqrt();
                let sky = [(120.0 + 90.0 * fy) as u8, (170.0 + 50.0 * fy) as u8, (215.0 - 40.0 * fy) as u8];
                let hill = d < 0.34;
                rgba.extend_from_slice(&if hill { [(70.0 + 60.0 * fx) as u8, 140, (90.0 + 40.0 * fy) as u8, 255] } else { [sky[0], sky[1], sky[2], 255] });
            }
        }
        p.set(n, n, rgba);
        p
    };
}

fn slug(word: &str) -> String {
    word.to_lowercase().replace(' ', "-")
}

fn sizes<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let items = [20.0, 24.0, 28.0, 40.0, 56.0, 76.0, 112.0].iter().map(|&s| spec(t, &format!("{s} pt"), Portrait::new(format!("portrait-{s}"), "Asha Rao", s).build(t))).collect();
    specs(t, items)
}

fn faces<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let drawn = |size: f32| row().size(size, size).center().child(glyph(&icons::USER, size * 0.5, t.c.primary.fg));
    PICTURE.with(|pic| {
        specs(t, vec![
            spec(t, "Picture, 40", Portrait::new("portrait-image-40", "Acme", 40.0).image(pic).build(t)),
            spec(t, "Picture, 76", Portrait::new("portrait-image-76", "Acme", 76.0).image(pic).build(t)),
            spec(t, "Drawn face on a ground, 40", Portrait::new("portrait-face-40", "Lee", 40.0).face(drawn(40.0)).ground(t.c.primary.bg).build(t)),
            spec(t, "Drawn face on a ground, 76", Portrait::new("portrait-face-76", "Lee", 76.0).face(drawn(76.0)).ground(t.identity().solid).build(t)),
            spec(t, "Initials on the identity colour, 56", Portrait::new("portrait-ground-56", "Asha Rao", 56.0).ground(t.identity().solid).build(t)),
        ])
    })
}

/// Each state at two sizes, in a box the ring's size: nothing moves between them.
fn states<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let all = [(Activity::None, "None"), (Activity::Working, "Working"), (Activity::NeedsYou, "Needs you"), (Activity::Failed, "Failed"), (Activity::Done, "Done"), (Activity::Info, "Has news")];
    let column = |size: f32| {
        let mut r = row().gap(t.space.at(4.0)).items_start().flex_wrap();
        for (a, word) in all {
            let tile = Portrait::new(format!("portrait-{size}-{}", slug(word)), "Asha Rao", size).state(a).ringed(true).under(t.c.page.bg).build(t);
            r = r.child(spec(t, word, tile));
        }
        r
    };
    let on_surface = row().gap(t.space.at(4.0)).padding(t.space.at(3.0)).radius(t.radius.lg).fill(t.c.surface.bg).items_start().children(all.iter().map(|(a, word)| {
        Portrait::new(format!("portrait-surface-{}", slug(word)), "Asha Rao", 28.0).state(*a).ringed(true).under(t.c.surface.bg).build(t)
    }));
    let room = portrait::ringed(28.0);
    col().gap(t.space.at(6.0)).child(column(28.0)).child(column(56.0)).child(spec(t, &format!("28 pt tiles on a surface, the dot ringed in its colour; each takes {room} pt"), on_surface))
        .child(text("The ring turns while working and holds still under Reduce Motion.").font(weight(t.ty.sm, t.ty.regular)).color(t.ink_muted()).a11y_hidden())
}
