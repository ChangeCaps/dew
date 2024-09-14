use std::sync::{atomic::Ordering, Arc};

use api::v1::{Todo, TodoStatus};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use tracing::info;
use uuid::Uuid;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/todos", get(get_todos))
        .route("/todos", post(add_todo))
        .route("/todos/completed", delete(delete_completed_todos))
        .route("/todos/:id/status", post(set_todo_status))
        .route("/todos/:id/title", post(set_todo_title))
        .route("/generation", get(get_generation))
}

async fn get_generation(State(state): State<Arc<AppState>>) -> Json<u64> {
    Json(state.generation.load(Ordering::Relaxed))
}

async fn get_todos(State(state): State<Arc<AppState>>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().await;
    let mut todos: Vec<_> = todos.values().cloned().collect();
    todos.sort_unstable_by(|a, b| a.created.cmp(&b.created).reverse());
    Json(todos)
}

async fn add_todo(State(state): State<Arc<AppState>>, Json(todo): Json<Todo>) -> Json<Todo> {
    let mut todos = state.todos.lock().await;
    todos.insert(todo.id, todo.clone());
    state.increment_generation();

    info!(
        id = %todo.id,
        title = %todo.title,
        "created todo"
    );

    Json(todo)
}

async fn set_todo_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(status): Json<TodoStatus>,
) -> Json<Todo> {
    let mut todos = state.todos.lock().await;
    let todo = todos.get_mut(&id).expect("todo not found");
    todo.status = status;

    state.increment_generation();

    info!(
        id = %todo.id,
        status = ?todo.status,
        "updated todo status"
    );

    Json(todo.clone())
}

async fn set_todo_title(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(title): Json<String>,
) -> Json<Todo> {
    let mut todos = state.todos.lock().await;
    let todo = todos.get_mut(&id).expect("todo not found");
    todo.title = title;

    state.increment_generation();

    info!(
        id = %todo.id,
        title = ?todo.title,
        "updated todo title"
    );

    Json(todo.clone())
}

async fn delete_completed_todos(State(state): State<Arc<AppState>>) {
    let mut todos = state.todos.lock().await;
    todos.retain(|_, todo| todo.status != TodoStatus::Completed);

    state.increment_generation();
}
