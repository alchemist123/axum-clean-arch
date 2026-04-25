use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::productivity::{
    ActiveTaskResponse, AddQueueItemRequest, AddScheduleEventRequest, LayoutResponse,
    PatchQueueItemRequest, ProductivityPeriod, ProductivityStatsResponse, QueueItemResponse,
    ScheduleEventResponse, StartTaskRequest, StopTaskResponse, WorkType, WidgetSize,
};
use crate::domain::productivity_repository::ProductivityRepository;

fn active_task_id_string(id: Uuid) -> String {
    id.to_string()
}

fn parse_work_type(s: &str) -> Result<WorkType, String> {
    match s.to_ascii_lowercase().as_str() {
        "office" => Ok(WorkType::Office),
        "personal" => Ok(WorkType::Personal),
        _ => Err(format!("workType must be office or personal, got: {}", s)),
    }
}

fn work_type_db(w: WorkType) -> &'static str {
    match w {
        WorkType::Office => "office",
        WorkType::Personal => "personal",
    }
}

fn period_bounds(period: ProductivityPeriod) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
    let end = Utc::now();
    let start = match period {
        ProductivityPeriod::Today => {
            let d = end.date_naive();
            d.and_hms_opt(0, 0, 0)
                .map(|t| t.and_utc())
                .expect("valid midnight")
        }
        ProductivityPeriod::Week => end - Duration::days(7),
        ProductivityPeriod::Month => end - Duration::days(30),
    };
    (start, end)
}

pub struct PostgresProductivityRepository {
    pool: PgPool,
}

impl PostgresProductivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductivityRepository for PostgresProductivityRepository {
    async fn get_active_task(&self) -> Result<Option<ActiveTaskResponse>, String> {
        let row = sqlx::query_as::<
            _,
            (Uuid, String, Option<String>, Option<String>, String, chrono::DateTime<chrono::Utc>),
        >(
            r#"
            SELECT id, task_name, jira_id, project_key, work_type, start_time
            FROM productivity_time_logs
            WHERE end_time IS NULL
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some((id, task_name, jira_id, project_key, work_type, start_time)) = row {
            return Ok(Some(ActiveTaskResponse {
                id: active_task_id_string(id),
                task_name,
                jira_id: jira_id.filter(|s| !s.is_empty()),
                project_key: project_key.unwrap_or_default(),
                work_type,
                start_time,
            }));
        }
        Ok(None)
    }

    async fn start_task(&self, request: &StartTaskRequest) -> Result<ActiveTaskResponse, String> {
        if request.task_name.trim().is_empty() {
            return Err("taskName must not be empty".to_string());
        }

        let work_type = parse_work_type(&request.work_type).map_err(|e| e.to_string())?;
        let has_active: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM productivity_time_logs WHERE end_time IS NULL)",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if has_active {
            return Err("An active task is already running; stop it before starting a new one".to_string());
        }

        let jira = request
            .jira_id
            .as_deref()
            .and_then(opt_string);
        let project = opt_string(&request.project_key);
        let now = Utc::now();
        let (id, start_time) = sqlx::query_as::<
            _,
            (Uuid, chrono::DateTime<chrono::Utc>),
        >(
            r#"
            INSERT INTO productivity_time_logs
                (task_name, jira_id, project_key, work_type, start_time)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, start_time
            "#,
        )
        .bind(&request.task_name)
        .bind(&jira)
        .bind(&project)
        .bind(work_type_db(work_type))
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ActiveTaskResponse {
            id: active_task_id_string(id),
            task_name: request.task_name.clone(),
            jira_id: jira.clone(),
            project_key: request.project_key.clone(),
            work_type: work_type_db(work_type).to_string(),
            start_time,
        })
    }

    async fn stop_task(&self, task_id: Uuid) -> Result<StopTaskResponse, String> {
        let row = sqlx::query_as::<
            _,
            (chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>),
        >(
            r#"
            SELECT start_time, end_time
            FROM productivity_time_logs
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let (start, previous_end) = row.ok_or_else(|| "Time log not found".to_string())?;
        if previous_end.is_some() {
            return Err("Task is not active".to_string());
        }

        let end = Utc::now();
        let duration_ms = (end - start).num_milliseconds();
        if duration_ms < 0 {
            return Err("Invalid time range for task".to_string());
        }

        sqlx::query(
            r#"
            UPDATE productivity_time_logs
            SET end_time = $1, duration_ms = $2
            WHERE id = $3 AND end_time IS NULL
            "#,
        )
        .bind(end)
        .bind(duration_ms)
        .bind(task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(StopTaskResponse {
            id: active_task_id_string(task_id),
            duration_ms,
            end_time: end,
        })
    }

    async fn get_stats(
        &self,
        period: ProductivityPeriod,
    ) -> Result<ProductivityStatsResponse, String> {
        let (range_start, range_end) = period_bounds(period);

        let tasks_executed: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM productivity_time_logs
            WHERE end_time IS NOT NULL
              AND end_time >= $1
              AND end_time <= $2
            "#,
        )
        .bind(range_start)
        .bind(range_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let deep_work_ms: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(duration_ms), 0)
            FROM productivity_time_logs
            WHERE end_time IS NOT NULL
              AND work_type = 'office'
              AND end_time >= $1
              AND end_time <= $2
            "#,
        )
        .bind(range_start)
        .bind(range_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let tasks_goal: i32 = sqlx::query_scalar(
            "SELECT tasks_goal FROM productivity_settings WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let goal = tasks_goal.max(0) as u32;
        let done = tasks_executed.max(0) as u32;
        let efficiency_ratio = if goal == 0 {
            0u32
        } else {
            let pct = (done * 100) / goal;
            pct.min(100)
        };

        let deep_work_hours = deep_work_ms.unwrap_or(0) as f64 / 3_600_000.0;

        Ok(ProductivityStatsResponse {
            tasks_executed: done,
            tasks_goal: goal,
            deep_work_hours,
            efficiency_ratio,
        })
    }

    async fn list_queue(&self) -> Result<Vec<QueueItemResponse>, String> {
        let rows = sqlx::query_as::<_, (Uuid, String, bool)>(
            r#"
            SELECT id, text, done
            FROM productivity_queue_items
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|(id, text, done)| QueueItemResponse {
                id: id.to_string(),
                text,
                done,
            })
            .collect())
    }

    async fn add_queue_item(&self, request: &AddQueueItemRequest) -> Result<QueueItemResponse, String> {
        if request.text.trim().is_empty() {
            return Err("text must not be empty".to_string());
        }

        let max_pos: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(position) FROM productivity_queue_items",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let pos = max_pos.map(|p| p + 1).unwrap_or(0);

        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO productivity_queue_items (text, done, position)
            VALUES ($1, false, $2)
            RETURNING id
            "#,
        )
        .bind(&request.text)
        .bind(pos)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(QueueItemResponse {
            id: id.to_string(),
            text: request.text.clone(),
            done: false,
        })
    }

    async fn patch_queue_item(
        &self,
        id: Uuid,
        request: &PatchQueueItemRequest,
    ) -> Result<QueueItemResponse, String> {
        let row = sqlx::query_as::<_, (String, bool)>(
            r#"
            UPDATE productivity_queue_items
            SET done = $1
            WHERE id = $2
            RETURNING text, done
            "#,
        )
        .bind(request.done)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let (text, done) = row.ok_or_else(|| "Queue item not found".to_string())?;

        Ok(QueueItemResponse {
            id: id.to_string(),
            text,
            done,
        })
    }

    async fn list_schedule(
        &self,
        date: chrono::NaiveDate,
    ) -> Result<Vec<ScheduleEventResponse>, String> {
        let rows = sqlx::query_as::<_, (Uuid, String, chrono::NaiveDate, String, String)>(
            r#"
            SELECT id, title, event_date, time_str, event_type
            FROM productivity_schedule_events
            WHERE event_date = $1
            ORDER BY time_str ASC
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(
                |(id, title, event_date, time_str, event_type)| ScheduleEventResponse {
                    id: id.to_string(),
                    title,
                    date: event_date.format("%Y-%m-%d").to_string(),
                    time: time_str,
                    event_type,
                },
            )
            .collect())
    }

    async fn add_schedule_event(
        &self,
        request: &AddScheduleEventRequest,
    ) -> Result<ScheduleEventResponse, String> {
        if request.title.trim().is_empty() {
            return Err("title must not be empty".to_string());
        }
        let d = chrono::NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
            .map_err(|_| "date must be YYYY-MM-DD".to_string())?;
        let event_type = request
            .event_type
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Video".to_string());

        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO productivity_schedule_events (title, event_date, time_str, event_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&request.title)
        .bind(d)
        .bind(&request.time)
        .bind(&event_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ScheduleEventResponse {
            id: id.to_string(),
            title: request.title.clone(),
            date: request.date.clone(),
            time: request.time.clone(),
            event_type,
        })
    }

    async fn get_layout(&self) -> Result<LayoutResponse, String> {
        let row = sqlx::query("SELECT layout_order, layout_sizes FROM productivity_layout WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let order: serde_json::Value = r.try_get("layout_order").map_err(|e| e.to_string())?;
            let sizes: serde_json::Value = r.try_get("layout_sizes").map_err(|e| e.to_string())?;
            let order: Vec<String> = serde_json::from_value(order).map_err(|e| e.to_string())?;
            let sizes: HashMap<String, WidgetSize> =
                serde_json::from_value(sizes).map_err(|e| e.to_string())?;
            return Ok(LayoutResponse { order, sizes });
        }

        Ok(LayoutResponse {
            order: vec![
                "clock".to_string(),
                "task-logger".to_string(),
                "stats".to_string(),
                "calendar".to_string(),
                "planning".to_string(),
                "report".to_string(),
            ],
            sizes: HashMap::new(),
        })
    }

    async fn put_layout(&self, layout: &LayoutResponse) -> Result<(), String> {
        let order = serde_json::to_value(&layout.order).map_err(|e| e.to_string())?;
        let sizes = serde_json::to_value(&layout.sizes).map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO productivity_layout (id, layout_order, layout_sizes, updated_at)
            VALUES (1, $1, $2, NOW())
            ON CONFLICT (id) DO UPDATE
            SET layout_order = $1, layout_sizes = $2, updated_at = NOW()
            "#,
        )
        .bind(order)
        .bind(sizes)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn opt_string(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
