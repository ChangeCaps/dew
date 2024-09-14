use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TodoStatus {
    Active,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub status: TodoStatus,
    pub created: DateTime<Utc>,
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            status: TodoStatus::Active,
            created: Utc::now(),
        }
    }
}
