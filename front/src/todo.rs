use api::v1::{Todo, TodoStatus};
use ori::prelude::*;

use crate::{set_todo_status, set_todo_title, spawn};

pub fn view() -> impl View<Todo> {
    with_state_default(|state: &mut TodoState, todo: &mut Todo| {
        let icon = modified_icon(state);

        let mut title = text_input()
            .on_input(on_title_input)
            .on_submit(on_title_submit)
            .text(&todo.title);

        if matches!(todo.status, TodoStatus::Completed) {
            title = title.color(Theme::CONTRAST_LOW);
        }

        let completed = checkbox(todo.status == TodoStatus::Completed).size(32.0);
        let completed = on_click(completed, on_completed_clicked);

        let view = hstack![icon, flex(title), completed]
            .justify(Justify::SpaceBetween)
            .align(Align::Center)
            .gap(8.0);

        container(pad(12.0, view)).border_width([0.0, 0.0, 1.0, 0.0])
    })
}

fn modified_icon<T>(state: &TodoState) -> impl View<T> {
    if !state.title_modified {
        return size(10.0, None);
    }

    let icon = fa::icon("circle")
        .solid(true)
        .color(Theme::WARNING)
        .size(10.0);

    size(10.0, Some(icon))
}

fn on_title_input(cx: &mut EventCx, (state, todo): &mut (TodoState, Todo), title: String) {
    todo.title = title;

    state.title_modified = true;
    cx.rebuild();

    // don't save the title until the user has stopped typing
}

fn on_title_submit(cx: &mut EventCx, (state, todo): &mut (TodoState, Todo), title: String) {
    todo.title = title;
    state.title_modified = false;

    cx.rebuild();
    cx.cmd_async(spawn(set_todo_title(todo.id, todo.title.clone())));
}

fn on_completed_clicked(cx: &mut EventCx, (_, todo): &mut (TodoState, Todo)) {
    todo.status = if todo.status == TodoStatus::Completed {
        TodoStatus::Active
    } else {
        TodoStatus::Completed
    };

    cx.cmd_async(spawn(set_todo_status(todo.id, todo.status)));
    cx.rebuild();
}

#[derive(Default)]
struct TodoState {
    title_modified: bool,
}
