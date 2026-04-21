use serde::Deserialize;

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

pub struct TasksApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl TasksApi<'_> {
    pub async fn get_status(&self, task_id: &str) -> Result<TaskStatus, ShurikenError> {
        self.0.get(&format!("/api/v2/tasks/{task_id}")).await
    }
}
