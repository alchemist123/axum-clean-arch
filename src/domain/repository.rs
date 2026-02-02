use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::team::{Team, TeamResponse, ListTeamsQuery, PaginatedTeamsResponse, CheckMemberRequest, CheckMemberResponse};

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn create_team(&self, team: &Team) -> Result<TeamResponse, String>;
    async fn get_team_by_id(&self, id: Uuid) -> Result<Option<TeamResponse>, String>;
    async fn get_team_by_name(&self, name: &str) -> Result<Option<TeamResponse>, String>;
    async fn list_teams(&self, query: &ListTeamsQuery) -> Result<PaginatedTeamsResponse, String>;
    async fn check_member(&self, request: &CheckMemberRequest) -> Result<CheckMemberResponse, String>;
    async fn count_teams(&self) -> Result<i64, String>;
    async fn member_exists_in_team(&self, email: &str, tms_id: &str) -> Result<bool, String>;
    async fn update_team_status(&self, id: Uuid, status: String, remarks: Option<String>) -> Result<(), String>;
    async fn delete_team(&self, id: Uuid) -> Result<(), String>;
}

