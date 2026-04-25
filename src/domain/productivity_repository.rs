use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use super::productivity::{
    ActiveTaskResponse, AddQueueItemRequest, AddScheduleEventRequest, LayoutResponse,
    PatchQueueItemRequest, ProductivityPeriod, ProductivityStatsResponse, QueueItemResponse,
    ScheduleEventResponse, StartTaskRequest, StopTaskResponse,
};

#[async_trait]
pub trait ProductivityRepository: Send + Sync {
    async fn get_active_task(&self) -> Result<Option<ActiveTaskResponse>, String>;

    async fn start_task(&self, request: &StartTaskRequest) -> Result<ActiveTaskResponse, String>;

    async fn stop_task(&self, task_id: Uuid) -> Result<StopTaskResponse, String>;

    async fn get_stats(
        &self,
        period: ProductivityPeriod,
    ) -> Result<ProductivityStatsResponse, String>;

    async fn list_queue(&self) -> Result<Vec<QueueItemResponse>, String>;

    async fn add_queue_item(&self, request: &AddQueueItemRequest) -> Result<QueueItemResponse, String>;

    async fn patch_queue_item(
        &self,
        id: Uuid,
        request: &PatchQueueItemRequest,
    ) -> Result<QueueItemResponse, String>;

    async fn list_schedule(&self, date: NaiveDate) -> Result<Vec<ScheduleEventResponse>, String>;

    async fn add_schedule_event(
        &self,
        request: &AddScheduleEventRequest,
    ) -> Result<ScheduleEventResponse, String>;

    async fn get_layout(&self) -> Result<LayoutResponse, String>;

    async fn put_layout(&self, layout: &LayoutResponse) -> Result<(), String>;
}
