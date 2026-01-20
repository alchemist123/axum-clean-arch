use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app_error::AppError,
    domain::{
        repository::TeamRepository,
        team::{
            Team, RegisterTeamRequest, TeamResponse, ListTeamsQuery, 
            PaginatedTeamsResponse, CheckMemberRequest, CheckMemberResponse, TeamCountResponse
        },
    },
};

pub struct TeamUseCase {
    repository: Arc<dyn TeamRepository + Send + Sync>,
}

impl TeamUseCase {
    pub fn new(repository: Arc<dyn TeamRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn register_team(&self, request: RegisterTeamRequest) -> Result<TeamResponse, AppError> {
        // Validate team name length
        if request.team_name.is_empty() || request.team_name.len() > 255 {
            return Err(AppError::ValidationError(
                "Team name must be between 1 and 255 characters".to_string(),
            ));
        }

        // Validate idea description length
        if request.idea_description.len() > 500 {
            return Err(AppError::ValidationError(
                "Idea description must be at most 500 characters".to_string(),
            ));
        }

        // Validate impact description length
        if request.impact_description.len() > 500 {
            return Err(AppError::ValidationError(
                "Impact description must be at most 500 characters".to_string(),
            ));
        }

        // Check if team name already exists
        let existing_team = self
            .repository
            .as_ref()
            .get_team_by_name(&request.team_name)
            .await
            .map_err(|e| AppError::Database(e))?;

        if existing_team.is_some() {
            return Err(AppError::Conflict(
                "Team name already exists".to_string(),
            ));
        }

        // Check if any member is already in another team
        for member in &request.members {
            let exists = self
                .repository
                .as_ref()
                .member_exists_in_team(&member.email, &member.tms_id)
                .await
                .map_err(|e| AppError::Database(e))?;

            if exists {
                return Err(AppError::Conflict(format!(
                    "Member with email {} or TMS ID {} is already registered in another team",
                    member.email, member.tms_id
                )));
            }
        }

        // Convert request to domain Team
        let team = Team {
            id: None,
            team_name: request.team_name,
            idea_description: request.idea_description,
            impact_description: request.impact_description,
            members: request
                .members
                .into_iter()
                .map(|m| crate::domain::team::TeamMember {
                    id: None,
                    name: m.name,
                    tms_id: m.tms_id,
                    email: m.email,
                    is_team_lead: m.is_team_lead,
                    team_id: None,
                })
                .collect(),
            created_at: None,
            updated_at: None,
        };

        // Validate team structure
        team.validate_team().map_err(|e| AppError::ValidationError(e))?;

        // Create team in repository
        self.repository
            .as_ref()
            .create_team(&team)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn list_teams(&self, query: ListTeamsQuery) -> Result<PaginatedTeamsResponse, AppError> {
        self.repository
            .as_ref()
            .list_teams(&query)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn check_member(&self, request: CheckMemberRequest) -> Result<CheckMemberResponse, AppError> {
        if request.email.is_none() && request.tms_id.is_none() {
            return Err(AppError::ValidationError(
                "Either email or tms_id must be provided".to_string(),
            ));
        }

        self.repository
            .as_ref()
            .check_member(&request)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn get_team_count(&self) -> Result<TeamCountResponse, AppError> {
        let count = self
            .repository
            .as_ref()
            .count_teams()
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(TeamCountResponse { total_teams: count })
    }

    pub async fn get_team_by_id(&self, id: Uuid) -> Result<TeamResponse, AppError> {
        let team = self
            .repository
            .as_ref()
            .get_team_by_id(id)
            .await
            .map_err(|e| AppError::Database(e))?;

        team.ok_or_else(|| AppError::NotFound("Team not found".to_string()))
    }
}

