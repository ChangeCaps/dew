use std::{future::Future, sync::OnceLock};

use dew_api::v1::{Todo, TodoStatus};
use eyre::Context;
use uuid::Uuid;

use crate::API_URL;

pub static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get().unwrap()
}

pub struct PushTodo(pub Todo);
pub struct UpdateTodos(pub Vec<Todo>);

pub async fn get_generation() -> eyre::Result<u64> {
    spawn(async move {
        let response = client()
            .get(format!("{}/generation", API_URL))
            .send()
            .await?;

        Ok(response.json().await?)
    })
    .await
}

pub async fn get_todos() -> eyre::Result<UpdateTodos> {
    spawn(async move {
        let response = client().get(format!("{}/todos", API_URL)).send().await?;
        let todos = response.json().await?;
        Ok(UpdateTodos(todos))
    })
    .await
}

pub async fn add_todo(todo: Todo) -> eyre::Result<()> {
    spawn(async move {
        let response = client()
            .post(format!("{}/todos", API_URL))
            .json(&todo)
            .send()
            .await?;

        response.error_for_status()?;

        Ok(())
    })
    .await
}

pub async fn delete_completed_todos() -> eyre::Result<()> {
    spawn(async move {
        let response = client()
            .delete(format!("{}/todos/completed", API_URL))
            .send()
            .await?;

        response.error_for_status()?;

        Ok(())
    })
    .await
}

pub async fn set_todo_status(id: Uuid, status: TodoStatus) -> eyre::Result<()> {
    spawn(async move {
        let response = client()
            .post(format!("{}/todos/{}/status", API_URL, id))
            .json(&status)
            .send()
            .await?;

        response.error_for_status()?;

        Ok(())
    })
    .await
}

pub async fn set_todo_title(id: Uuid, title: String) -> eyre::Result<()> {
    spawn(async move {
        let response = client()
            .post(format!("{}/todos/{}/title", API_URL, id))
            .json(&title)
            .send()
            .await?;

        response.error_for_status()?;

        Ok(())
    })
    .await
}

pub async fn spawn<T: Send + 'static>(
    fut: impl Future<Output = eyre::Result<T>> + Send + 'static,
) -> eyre::Result<T> {
    tokio::spawn(fut)
        .await
        .wrap_err("Tokio error")
        .and_then(|r| r)
}
