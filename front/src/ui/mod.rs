pub mod todo;
pub mod todo_input;

use dew_api::v1::TodoStatus;
use ori::prelude::*;
use ori_font_awesome as fa;

use crate::{api, Data};

pub fn view(data: &mut Data) -> impl View<Data> {
    let mut todos = vstack_vec().align(Align::Fill);

    for i in 0..data.todos.len() {
        todos.push(focus(todo::view(), move |data: &mut Data, lens| {
            data.todos.get_mut(i).map(lens);
        }));
    }

    let todos = vscroll(todos).inset(4.0);
    let todos = container(todos).border_radius(8.0).mask(true);

    let view = vstack![todo_input::view(), flex(todos)]
        .align(Align::Fill)
        .gap(8.0);

    let delete_completed = match any_todos_completed(data) {
        true => Some(pad(20.0, bottom_left(delete_completed_todos_button()))),
        false => None,
    };

    let view = zstack![view, delete_completed];

    pad([16.0, 32.0], view)
}

fn any_todos_completed(data: &Data) -> bool {
    (data.todos.iter()).any(|todo| todo.status == TodoStatus::Completed)
}

fn delete_completed_todos_button() -> impl View<Data> {
    let delete = fa::icon("trash").color(Theme::SURFACE).size(64.0);
    let delete = button(delete)
        .color(Theme::DANGER)
        .border_radius(24.0)
        .padding(12.0);

    on_click(delete, |cx, data: &mut Data| {
        data.todos.retain(|todo| todo.status == TodoStatus::Active);
        cx.cmd_async(api::delete_completed_todos());
        cx.rebuild();
    })
}
