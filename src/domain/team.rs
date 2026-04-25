use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Option<Uuid>,
    pub name: String,
    pub tms_id: String,
    pub email: String,
    pub is_team_lead: bool,
    pub team_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamStatus {
    Pending,
    Approved,
    Rejected,
}

impl From<String> for TeamStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Approved" => TeamStatus::Approved,
            "Rejected" => TeamStatus::Rejected,
            _ => TeamStatus::Pending,
        }
    }
}

impl Into<String> for TeamStatus {
    fn into(self) -> String {
        match self {
            TeamStatus::Approved => "Approved".to_string(),
            TeamStatus::Rejected => "Rejected".to_string(),
            TeamStatus::Pending => "Pending".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Team {
    pub id: Option<Uuid>,
    #[validate(length(min = 1, max = 255, message = "Team name must be between 1 and 255 characters"))]
    pub team_name: String,
    #[validate(length(min = 10, max = 2000, message = "Description must be between 10 and 2000 characters"))]
    pub idea_description: String,
    #[validate(length(min = 10, max = 2000, message = "Impact statement must be between 10 and 2000 characters"))]
    pub impact_description: String,
    pub status: TeamStatus,
    pub admin_remarks: Option<String>,
    pub members: Vec<TeamMember>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Team {
    pub fn validate_team(&self) -> Result<(), String> {
        // Validate that there's exactly one team lead
        let team_leads: Vec<&TeamMember> = self.members.iter().filter(|m| m.is_team_lead).collect();
        if team_leads.len() != 1 {
            return Err("Team must have exactly one team lead".to_string());
        }

        // Validate that there's at least one member
        if self.members.is_empty() {
            return Err("Team must have at least one member".to_string());
        }

        // Validate member emails are unique within the team
        let emails: Vec<&String> = self.members.iter().map(|m| &m.email).collect();
        let unique_emails: std::collections::HashSet<&String> = emails.iter().cloned().collect();
        if emails.len() != unique_emails.len() {
            return Err("Member emails must be unique within the team".to_string());
        }

        // Validate member tms_ids are unique within the team
        let tms_ids: Vec<&String> = self.members.iter().map(|m| &m.tms_id).collect();
        let unique_tms_ids: std::collections::HashSet<&String> = tms_ids.iter().cloned().collect();
        if tms_ids.len() != unique_tms_ids.len() {
            return Err("Member TMS IDs must be unique within the team".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterTeamRequest {
    pub team_name: String,
    pub idea_description: String,
    pub impact_description: String,
    pub members: Vec<RegisterMemberRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterMemberRequest {
    pub name: String,
    pub tms_id: String,
    pub email: String,
    pub is_team_lead: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub team_name: String,
    pub idea_description: String,
    pub impact_description: String,
    pub status: TeamStatus,
    pub admin_remarks: Option<String>,
    pub members: Vec<TeamMemberResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub name: String,
    pub tms_id: String,
    pub email: String,
    pub is_team_lead: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTeamsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub search: Option<String>,
    pub team_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedTeamsResponse {
    pub teams: Vec<TeamResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckMemberRequest {
    pub email: Option<String>,
    pub tms_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckMemberResponse {
    pub is_registered: bool,
    pub team_name: Option<String>,
    pub team_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamCountResponse {
    pub total_teams: i64,
}

