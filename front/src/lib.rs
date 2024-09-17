mod api;
mod ui;

use std::time::Duration;

use dew_api::v1::Todo;
use ori::prelude::*;
use ori_font_awesome as fa;
use tokio::time;

#[cfg(feature = "local")]
const API_URL: &str = "http://localhost:7890/api/v1";

#[cfg(not(feature = "local"))]
const API_URL: &str = "https://64.226.73.125:7890/api/v1";

const CERT: &[u8] = include_bytes!("../cert.pem");

pub struct Data {
    pub todos: Vec<Todo>,
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

    let _ = api::CLIENT.set(client);

    let app = App::build()
        .delegate(Delegate)
        .window(window, ui::view)
        .style(style);

    let mut data = Data { todos: vec![] };

    ori::run(app, &mut data)?;

    Ok(())
}

struct Delegate;

impl AppDelegate<Data> for Delegate {
    fn init(&mut self, cx: &mut DelegateCx<Data>, _data: &mut Data) {
        cx.cmd_async(api::get_todos());
        cx.cmd_async(api::spawn(todos_update_loop(cx.proxy())));
    }

    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(update) = event.cmd::<eyre::Result<_>>() {
            match update {
                Ok(api::UpdateTodos(todos)) => {
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

        if let Some(api::PushTodo(todo)) = event.cmd() {
            data.todos.insert(0, todo.clone());
            cx.rebuild();

            return true;
        }

        false
    }
}

async fn todos_update_loop(proxy: CommandProxy) -> eyre::Result<()> {
    let mut generation = None;

    loop {
        time::sleep(Duration::from_secs(5)).await;

        let Ok(new_generation) = api::get_generation().await else {
            continue;
        };

        if generation != Some(new_generation) {
            generation = Some(new_generation);
            proxy.cmd_async(api::get_todos());
        }
    }
}
