use dew_api::v1::Todo;
use ori::prelude::*;
use ori_font_awesome as fa;

use crate::{api, Data};

pub fn view() -> impl View<Data> {
    with_data(Todo::default, |todo| {
        let input = text_input()
            .on_input(|_, todo: &mut Todo, title| todo.title = title)
            .on_submit(|cx, todo, title| {
                todo.title = title;
                on_send_clicked(cx, todo);
            })
            .text(&todo.title);

        let input = container(pad([16.0, 12.0], input))
            .border_radius([8.0, 0.0, 0.0, 8.0])
            .mask(true);

        let add = fa::icon("paper-plane").color(Theme::SURFACE);
        let add = button(center(add))
            .border_radius([0.0, 8.0, 8.0, 0.0])
            .padding([16.0, 8.0])
            .color(Theme::SUCCESS);

        let add = on_click(add, on_send_clicked);

        hstack![flex(input), add].align(Align::Stretch)
    })
}

fn on_send_clicked(cx: &mut EventCx, todo: &mut Todo) {
    cx.cmd(api::PushTodo(todo.clone()));
    cx.cmd_async(api::add_todo(todo.clone()));
    cx.rebuild();

    *todo = Todo::default();
}
