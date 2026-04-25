use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkType {
    Office,
    Personal,
}

/// Query period for `GET /stats`
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductivityPeriod {
    #[default]
    Today,
    Week,
    Month,
}

// --- API DTOs (camelCase per spec) ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTaskResponse {
    pub id: String,
    pub task_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jira_id: Option<String>,
    pub project_key: String,
    pub work_type: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTaskRequest {
    pub task_name: String,
    #[serde(default)]
    pub jira_id: Option<String>,
    pub project_key: String,
    pub work_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTaskResponse {
    pub id: String,
    pub duration_ms: i64,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductivityStatsResponse {
    pub tasks_executed: u32,
    pub tasks_goal: u32,
    pub deep_work_hours: f64,
    pub efficiency_ratio: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemResponse {
    pub id: String,
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddQueueItemRequest {
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchQueueItemRequest {
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSize {
    pub span_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutResponse {
    pub order: Vec<String>,
    pub sizes: std::collections::HashMap<String, WidgetSize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEventResponse {
    pub id: String,
    pub title: String,
    /// `YYYY-MM-DD`
    pub date: String,
    pub time: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddScheduleEventRequest {
    pub title: String,
    /// `YYYY-MM-DD`
    pub date: String,
    pub time: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ScheduleQuery {
    /// `YYYY-MM-DD` — if absent, use today's UTC date
    #[serde(default)]
    pub date: Option<String>,
}
