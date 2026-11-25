use rune_ui_style::{ButtonVariant, Tone};

use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::toast::{Toast, Toaster};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toast", "Toast", "display").story("Messages", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let mut toaster = Toaster::new("toaster");
    for key in &g.toasts {
        let k = key.clone();
        toaster = toaster.toast(Toast::<S>::new(key.clone(), "Saved").description("All changes are on disk.").tone(Tone::Success).on_close(move |s: &mut S| acc(s).toasts.retain(|x| *x != k)).build(t));
    }
    toaster.mount(t, c.layer);
    specs(t, vec![
        spec(t, "Plain", Toast::<S>::new("toast-plain", "Saved").description("All changes are on disk.").build(t)),
        spec(t, "Meaning, action, close", Toast::<S>::new("toast-undo", "Task deleted").description("It is in the bin for 30 days.").tone(Tone::Success).action("Undo", move |s: &mut S| acc(s).presses += 1).on_close(move |s: &mut S| acc(s).presses += 1).build(t)),
        spec(t, "Live (press it: toasts stack at the window's bottom right)", Button::<S>::new("toast-open", "Show a toast").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| {
            let g = acc(s);
            let n = g.presses;
            g.presses += 1;
            g.toasts.push(format!("toast-live-{n}"));
        }).build(t)),
    ])
}
