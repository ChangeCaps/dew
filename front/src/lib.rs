use std::{future::Future, time::Duration};

use api::v1::{Todo, TodoStatus};
use eyre::Context;
use ori::prelude::*;
use uuid::Uuid;

const API_URL: &str = "http://64.226.73.125:7890/api/v1";

#[ori::main]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    ori::log::install()?;

    let mut window = Window::new().title("Dew");

    if is_desktop!() {
        window = window.size(400, 800).resizable(false);
    }

    let style = style! {
        "Text" {
            "font_size": pt(16.0),
        },

        "TextInput" {
            "font_size": pt(16.0),
        },

        "Icon" {
            "size": pt(16.0),
        },
    };

    let app = App::build()
        .delegate(Delegate)
        .window(window, ui)
        .style(style);

    let mut data = Data { todos: vec![] };

    ori::run(app, &mut data)?;

    Ok(())
}

struct Data {
    todos: Vec<Todo>,
}

fn ui(data: &mut Data) -> impl View<Data> {
    let mut todos = vstack_vec().align(Align::Fill);

    for (i, todo) in data.todos.iter().enumerate() {
        todos.push(focus(
            move |data: &mut Data, lens| lens(&mut data.todos[i]),
            todo_item(todo),
        ));
    }

    let todos = vscroll(todos).inset(4.0);
    let todos = container(todos).border_radius(8.0).mask(true);

    let view = vstack![todo_input(), flex(todos)]
        .align(Align::Fill)
        .gap(8.0);

    let delete_completed = if data
        .todos
        .iter()
        .any(|todo| todo.status == TodoStatus::Completed)
    {
        let delete = fa::icon("trash").color(Theme::SURFACE).size(64.0);
        let delete = button(delete)
            .color(Theme::DANGER)
            .border_radius(24.0)
            .padding(12.0);

        let delete = on_click(delete, |cx, _| {
            cx.cmd_async(spawn(delete_completed_todos()));
        });

        Some(pad(20.0, bottom_left(delete)))
    } else {
        None
    };

    let view = zstack![view, delete_completed];

    pad([16.0, 32.0], view)
}

fn todo_item(todo: &Todo) -> impl View<Todo> {
    let mut title = text_input()
        .on_submit(|cx, todo: &mut Todo, title| {
            todo.title = title;
            cx.rebuild();
            cx.cmd_async(spawn(set_todo_title(todo.id, todo.title.clone())));
        })
        .text(&todo.title);

    match todo.status {
        TodoStatus::Active => {}
        TodoStatus::Completed => {
            title.color = Theme::CONTRAST_LOW.into();
        }
    }

    let completed = checkbox(todo.status == TodoStatus::Completed).size(32.0);
    let completed = on_click(completed, move |cx, todo: &mut Todo| {
        todo.status = if todo.status == TodoStatus::Completed {
            TodoStatus::Active
        } else {
            TodoStatus::Completed
        };

        cx.cmd_async(spawn(set_todo_status(todo.id, todo.status)));
        cx.rebuild();
    });

    let view = hstack![flex(title), completed]
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    container(pad(12.0, view)).border_width([0.0, 0.0, 1.0, 0.0])
}

fn todo_input() -> impl View<Data> {
    with_data(Todo::default, |todo| {
        let input = text_input()
            .on_input(|_, todo: &mut Todo, title| todo.title = title)
            .on_submit(|cx, todo, title| {
                todo.title = title;
                on_input_todo(cx, todo);
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

        let add = on_click(add, on_input_todo);

        hstack![flex(input), add].align(Align::Stretch)
    })
}

fn on_input_todo(cx: &mut EventCx, todo: &mut Todo) {
    cx.cmd(PushTodo(todo.clone()));
    cx.cmd_async(spawn(add_todo(todo.clone())));
    cx.rebuild();

    *todo = Todo::default();
}

struct PushTodo(Todo);
struct UpdateTodos(Vec<Todo>);

async fn get_generation() -> eyre::Result<u64> {
    let response = reqwest::get(format!("{}/generation", API_URL)).await?;
    let generation = response.json().await?;
    Ok(generation)
}

async fn get_todos() -> eyre::Result<UpdateTodos> {
    let response = reqwest::get(format!("{}/todos", API_URL)).await?;
    let todos = response.json().await?;
    Ok(UpdateTodos(todos))
}

async fn add_todo(todo: Todo) -> eyre::Result<()> {
    let response = reqwest::Client::new()
        .post(format!("{}/todos", API_URL))
        .json(&todo)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

async fn delete_completed_todos() -> eyre::Result<()> {
    let response = reqwest::Client::new()
        .delete(format!("{}/todos/completed", API_URL))
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

async fn set_todo_status(id: Uuid, status: TodoStatus) -> eyre::Result<()> {
    let response = reqwest::Client::new()
        .post(format!("{}/todos/{}/status", API_URL, id))
        .json(&status)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

async fn set_todo_title(id: Uuid, title: String) -> eyre::Result<()> {
    let response = reqwest::Client::new()
        .post(format!("{}/todos/{}/title", API_URL, id))
        .json(&title)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

async fn spawn<T: Send + 'static>(
    fut: impl Future<Output = eyre::Result<T>> + Send + 'static,
) -> eyre::Result<T> {
    tokio::spawn(fut)
        .await
        .wrap_err("Tokio error")
        .and_then(|r| r)
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn init(&mut self, cx: &mut DelegateCx<Data>, _data: &mut Data) {
        cx.cmd_async(spawn(get_todos()));

        let proxy = cx.proxy();
        cx.cmd_async::<eyre::Result<()>>(spawn(async move {
            let mut generation = None;

            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let new_generation = get_generation().await?;

                if generation != Some(new_generation) {
                    generation = Some(new_generation);
                    proxy.cmd_async(get_todos());
                }
            }
        }));
    }

    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(update) = event.cmd::<eyre::Result<_>>() {
            match update {
                Ok(UpdateTodos(todos)) => {
                    data.todos = todos.clone();
                    cx.rebuild();
                }
                Err(err) => {
                    error!("failed to update todos: {}", err);
                }
            }

            return true;
        }

        if let Some(Err(result)) = event.cmd::<eyre::Result<()>>() {
            error!("{:?}", result);

            return true;
        }

        if let Some(PushTodo(todo)) = event.cmd() {
            data.todos.insert(0, todo.clone());
            cx.rebuild();

            return true;
        }

        false
    }
}
