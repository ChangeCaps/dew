mod todo;

use std::{future::Future, sync::OnceLock, time::Duration};

use api::v1::{Todo, TodoStatus};
use eyre::Context;
use ori::prelude::*;
use uuid::Uuid;

#[cfg(feature = "local")]
const API_URL: &str = "http://localhost:7890/api/v1";

#[cfg(not(feature = "local"))]
const API_URL: &str = "https://64.226.73.125:7890/api/v1";

const CERT: &[u8] = include_bytes!("../cert.pem");

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get().unwrap()
}

#[ori::main]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    ori::log::install()?;

    let mut window = Window::new().title("Dew");

    if is_desktop!() {
        window = window.size(400, 800).resizable(false);
    }

    let style = Styles::new()
        .with(TextStyle::FONT_SIZE, pt(16.0))
        .with(TextInputStyle::FONT_SIZE, pt(16.0))
        .with(fa::IconStyle::SIZE, pt(16.0));

    let client = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(CERT)?)
        .build()?;

    let _ = CLIENT.set(client);

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

impl Data {
    fn any_completed(&self) -> bool {
        (self.todos.iter()).any(|todo| todo.status == TodoStatus::Completed)
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    let mut todos = vstack_vec().align(Align::Fill);

    for i in 0..data.todos.len() {
        todos.push(focus(todo::view(), move |data: &mut Data, lens| {
            data.todos.get_mut(i).map(lens);
        }));
    }

    let todos = vscroll(todos).inset(4.0);
    let todos = container(todos).border_radius(8.0).mask(true);

    let view = vstack![todo_input(), flex(todos)]
        .align(Align::Fill)
        .gap(8.0);

    let delete_completed = match data.any_completed() {
        true => Some(pad(20.0, bottom_left(delete_completed_todos_button()))),
        false => None,
    };

    let view = zstack![view, delete_completed];

    pad([16.0, 32.0], view)
}

fn delete_completed_todos_button() -> impl View<Data> {
    let delete = fa::icon("trash").color(Theme::SURFACE).size(64.0);
    let delete = button(delete)
        .color(Theme::DANGER)
        .border_radius(24.0)
        .padding(12.0);

    on_click(delete, |cx, data: &mut Data| {
        data.todos.retain(|todo| todo.status == TodoStatus::Active);
        cx.cmd_async(spawn(delete_completed_todos()));
        cx.rebuild();
    })
}

fn todo_input() -> impl View<Data> {
    with_data(Todo::default, |todo| {
        let input = text_input()
            .on_input(|_, todo: &mut Todo, title| todo.title = title)
            .on_submit(|cx, todo, title| {
                todo.title = title;
                on_todo_input_clicked(cx, todo);
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

        let add = on_click(add, on_todo_input_clicked);

        hstack![flex(input), add].align(Align::Stretch)
    })
}

fn on_todo_input_clicked(cx: &mut EventCx, todo: &mut Todo) {
    cx.cmd(PushTodo(todo.clone()));
    cx.cmd_async(spawn(add_todo(todo.clone())));
    cx.rebuild();

    *todo = Todo::default();
}

struct PushTodo(Todo);
struct UpdateTodos(Vec<Todo>);

pub async fn get_generation() -> eyre::Result<u64> {
    let response = client()
        .get(format!("{}/generation", API_URL))
        .send()
        .await?;
    let generation = response.json().await?;
    Ok(generation)
}

async fn get_todos() -> eyre::Result<UpdateTodos> {
    let response = client().get(format!("{}/todos", API_URL)).send().await?;
    let todos = response.json().await?;
    Ok(UpdateTodos(todos))
}

pub async fn add_todo(todo: Todo) -> eyre::Result<()> {
    let response = client()
        .post(format!("{}/todos", API_URL))
        .json(&todo)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

pub async fn delete_completed_todos() -> eyre::Result<()> {
    let response = client()
        .delete(format!("{}/todos/completed", API_URL))
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

pub async fn set_todo_status(id: Uuid, status: TodoStatus) -> eyre::Result<()> {
    let response = client()
        .post(format!("{}/todos/{}/status", API_URL, id))
        .json(&status)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

pub async fn set_todo_title(id: Uuid, title: String) -> eyre::Result<()> {
    let response = client()
        .post(format!("{}/todos/{}/title", API_URL, id))
        .json(&title)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

pub async fn spawn<T: Send + 'static>(
    fut: impl Future<Output = eyre::Result<T>> + Send + 'static,
) -> eyre::Result<T> {
    tokio::spawn(fut)
        .await
        .wrap_err("Tokio error")
        .and_then(|r| r)
}

async fn todos_updater(proxy: CommandProxy) -> eyre::Result<()> {
    let mut generation = None;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let Ok(new_generation) = get_generation().await else {
            continue;
        };

        if generation != Some(new_generation) {
            generation = Some(new_generation);
            proxy.cmd_async(get_todos());
        }
    }
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn init(&mut self, cx: &mut DelegateCx<Data>, _data: &mut Data) {
        cx.cmd_async(spawn(get_todos()));
        cx.cmd_async(spawn(todos_updater(cx.proxy())));
    }

    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(update) = event.cmd::<eyre::Result<_>>() {
            match update {
                Ok(UpdateTodos(todos)) => {
                    data.todos = todos.clone();
                    cx.rebuild();
                }
                Err(err) => {
                    error!("{:?}", err);
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
