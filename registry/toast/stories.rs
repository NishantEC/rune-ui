use rune_ui_style::{ButtonVariant, Tone};

use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::toast::{Position, Toast, ToastHost, ToastSpec, Toaster, Toasts};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toast", "Toast", "display").story("Messages", all::<S>).story("A host: timed, paused, with an action", host::<S>)
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

fn host<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let hosted: ToastHost = g.slot("toast-host");
    let centre = g.flag("toast-centre");
    Toasts::<S>::new("toasts", &hosted, move |s: &mut S| acc(s).slot_mut::<ToastHost>("toast-host"))
        .position(if centre { Position::BottomCenter } else { Position::BottomTrailing })
        .on_action(move |s: &mut S, tag: &str| acc(s).choose("toast-said", tag))
        .mount(t, c.layer);
    let said = g.choice("toast-said", "");
    let show = Button::<S>::new("toast-settle", "Settle a thread").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| {
        acc(s).slot_mut::<ToastHost>("toast-host").push(ToastSpec::new("Thread settled").description("Fix the login redirect").action("Undo", "undo-settle"));
    }).build(t);
    let place = Button::<S>::new("toast-place", if centre { "Show at the bottom trailing edge" } else { "Show at the bottom centre" }).variant(ButtonVariant::Ghost).on_press(move |s: &mut S| acc(s).toggle("toast-centre")).build(t);
    let stack = rune::col().gap(t.space.at(2.0)).items_start()
        .child(Toast::<S>::new("toast-stack-1", "Thread settled").description("Fix the login redirect").action("Undo", |_: &mut S| {}).on_close(|_: &mut S| {}).build(t))
        .child(Toast::<S>::new("toast-stack-2", "Archived 3 threads").tone(Tone::Success).action("Undo", |_: &mut S| {}).on_close(|_: &mut S| {}).build(t))
        .child(Toast::<S>::new("toast-stack-3", "Could not reach the server").description("Check the connection and try again.").tone(Tone::Danger).on_close(|_: &mut S| {}).build(t));
    let result = if said.is_empty() { "No action pressed yet".to_owned() } else { format!("Action pressed: {said}") };
    specs(t, vec![
        spec(t, "A stack: newest last, up to four shown", stack),
        spec(t, &format!("Live: 8 seconds, held while the pointer or focus is on them. {result}"), rune::row().gap(t.space.at(2.0)).child(show).child(place)),
    ])
}
