use rune::El;

use super::super::field::Field;
use super::super::form::{Form, Status, email, min_len, problems, required};
use super::super::gallery::{Access, Ctx, GalleryState, StoryItem, spec, specs};
use super::super::input::Input;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("form", "Form", "form").story("Validation and submit states", all::<S>)
}

fn input<S: 'static>(t: &rune_ui_style::Theme, g: &GalleryState, acc: Access<S>, key: &str, name: &str, placeholder: &str, invalid: bool) -> El {
    let k = key.to_owned();
    Input::<S>::new(format!("{key}-input"), g.edit(key), move |s: &mut S| acc(s).edit_mut(&k)).name(name.to_owned()).placeholder(placeholder.to_owned()).invalid(invalid).build(t)
}

fn errors(g: &GalleryState) -> [Option<String>; 3] {
    let (name, mail, team) = (g.edit("form-name").text.clone(), g.edit("form-email").text.clone(), g.edit("form-team").text.clone());
    [required(&name, "Name").or_else(|| min_len(&name, 2, "Name")), required(&mail, "Email").or_else(|| email(&mail)), min_len(&team, 2, "Team")]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let tried = g.flag("form-tried");
    let errs = if tried { errors(g) } else { [None, None, None] };
    let status = if g.flag("form-done") { Status::Done("Account created. Check your inbox.".to_owned()) } else { problems(&errs).map_or(Status::Idle, Status::Failed) };
    let live = Form::<S>::new("form-demo", "Create an account").description("Two required fields; the team is optional.")
        .field(Field::new("form-name", "Name").required().error(errs[0].clone()).child(input(t, g, acc, "form-name", "Name", "Asha Rao", errs[0].is_some())).build(t))
        .field(Field::new("form-email", "Email").required().error(errs[1].clone()).child(input(t, g, acc, "form-email", "Email", "name@example.com", errs[1].is_some())).build(t))
        .field(Field::new("form-team", "Team").optional().description("Who you work with, if anyone.").error(errs[2].clone()).child(input(t, g, acc, "form-team", "Team", "Acme design", errs[2].is_some())).build(t))
        .status(status)
        .cancel("Cancel", move |s: &mut S| {
            let g = acc(s);
            for k in ["form-name", "form-email", "form-team"] {
                g.edit_mut(k).clear();
            }
            g.flags.insert("form-tried".to_owned(), false);
            g.flags.insert("form-done".to_owned(), false);
        })
        .submit("Create account", move |s: &mut S| {
            let g = acc(s);
            g.flags.insert("form-tried".to_owned(), true);
            let ok = problems(&errors(g)).is_none();
            g.flags.insert("form-done".to_owned(), ok);
        })
        .build(t);
    let busy = Form::<S>::new("form-busy", "Change password").field(Field::new("form-busy-old", "Current password").required().child(Input::<S>::new("form-busy-input", g.edit("secret"), move |s: &mut S| acc(s).edit_mut("secret")).name("Current password").secure().build(t)).build(t)).status(Status::Submitting).cancel("Cancel", |_: &mut S| {}).submit("Save", |_: &mut S| {}).build(t);
    let failed = Form::<S>::new("form-failed", "Invite a teammate").width(340.0).field(Field::new("form-failed-mail", "Email").required().error(Some("Enter an email address like name@example.com".to_owned())).child(input(t, g, acc, "form-failed-mail", "Email", "name@example.com", true)).build(t)).status(Status::Failed("1 field needs attention".to_owned())).submit("Send invite", |_: &mut S| {}).build(t);
    specs(t, vec![spec(t, "Press Create account empty: messages, a count, then success", live), spec(t, "Submitting", busy), spec(t, "Failed", failed)])
}
