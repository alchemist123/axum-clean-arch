use std::sync::Arc;
use uuid::Uuid;

use chrono::Utc;

use crate::{
    app_error::AppError,
    domain::productivity::{
        ActiveTaskResponse, AddQueueItemRequest, AddScheduleEventRequest, LayoutResponse,
        PatchQueueItemRequest, ProductivityPeriod, ProductivityStatsResponse, QueueItemResponse,
        ScheduleEventResponse, ScheduleQuery, StartTaskRequest, StopTaskResponse,
    },
    domain::productivity_repository::ProductivityRepository,
};

pub struct ProductivityUseCase {
    repository: Arc<dyn ProductivityRepository + Send + Sync>,
}

impl ProductivityUseCase {
    pub fn new(repository: Arc<dyn ProductivityRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn get_active_task(
        &self,
    ) -> Result<Option<ActiveTaskResponse>, AppError> {
        self.repository
            .get_active_task()
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn start_task(&self, request: StartTaskRequest) -> Result<ActiveTaskResponse, AppError> {
        if request.task_name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "taskName is required".to_string(),
            ));
        }
        self.repository
            .start_task(&request)
            .await
            .map_err(|e| {
                if e.contains("already running") {
                    return AppError::Conflict(e);
                }
                if e.contains("workType") || e.contains("office") {
                    return AppError::ValidationError(e);
                }
                AppError::Database(e)
            })
    }

    pub async fn stop_task(&self, task_id: Uuid) -> Result<StopTaskResponse, AppError> {
        self.repository
            .stop_task(task_id)
            .await
            .map_err(|e| {
                if e.contains("not found") {
                    return AppError::NotFound(e);
                }
                if e == "Task is not active" {
                    return AppError::Conflict(e);
                }
                AppError::Database(e)
            })
    }

    pub async fn get_stats(
        &self,
        period: Option<ProductivityPeriod>,
    ) -> Result<ProductivityStatsResponse, AppError> {
        self.repository
            .get_stats(period.unwrap_or_default())
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn list_queue(&self) -> Result<Vec<QueueItemResponse>, AppError> {
        self.repository
            .list_queue()
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn add_queue_item(
        &self,
        request: AddQueueItemRequest,
    ) -> Result<QueueItemResponse, AppError> {
        self.repository
            .add_queue_item(&request)
            .await
            .map_err(|e| {
                if e.contains("empty") {
                    return AppError::ValidationError(e);
                }
                AppError::Database(e)
            })
    }

    pub async fn patch_queue_item(
        &self,
        id: Uuid,
        request: PatchQueueItemRequest,
    ) -> Result<QueueItemResponse, AppError> {
        self.repository
            .patch_queue_item(id, &request)
            .await
            .map_err(|e| {
                if e.contains("not found") {
                    return AppError::NotFound(e);
                }
                AppError::Database(e)
            })
    }

    pub async fn list_schedule(
        &self,
        query: ScheduleQuery,
    ) -> Result<Vec<ScheduleEventResponse>, AppError> {
        let date = if let Some(d) = query.date {
            chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("date must be YYYY-MM-DD".to_string()))?
        } else {
            Utc::now().date_naive()
        };

        self.repository
            .list_schedule(date)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn add_schedule_event(
        &self,
        request: AddScheduleEventRequest,
    ) -> Result<ScheduleEventResponse, AppError> {
        self.repository
            .add_schedule_event(&request)
            .await
            .map_err(|e| {
                if e.contains("empty") || e.contains("YYYY-MM-DD") {
                    return AppError::ValidationError(e);
                }
                AppError::Database(e)
            })
    }

    pub async fn get_layout(&self) -> Result<LayoutResponse, AppError> {
        self.repository
            .get_layout()
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn put_layout(&self, layout: LayoutResponse) -> Result<(), AppError> {
        if layout.order.is_empty() {
            return Err(AppError::ValidationError(
                "order must not be empty".to_string(),
            ));
        }
        self.repository
            .put_layout(&layout)
            .await
            .map_err(|e| AppError::Database(e))
    }
}
