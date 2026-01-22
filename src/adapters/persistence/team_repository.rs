use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    repository::TeamRepository,
    team::{
        Team, TeamResponse, TeamMemberResponse, ListTeamsQuery, 
        PaginatedTeamsResponse, CheckMemberRequest, CheckMemberResponse
    },
};

pub struct PostgresTeamRepository {
    pool: PgPool,
}

impl PostgresTeamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRepository for PostgresTeamRepository {
    async fn create_team(&self, team: &Team) -> Result<TeamResponse, String> {
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        // Insert team
        let team_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO teams (team_name, idea_description, impact_description, status)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&team.team_name)
        .bind(&team.idea_description)
        .bind(&team.impact_description)
        .bind::<String>(team.status.clone().into())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Insert team members
        let mut members = Vec::new();
        for member in &team.members {
            let member_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO team_members (team_id, name, tms_id, email, is_team_lead)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id
                "#,
            )
            .bind(&team_id)
            .bind(&member.name)
            .bind(&member.tms_id)
            .bind(&member.email)
            .bind(&member.is_team_lead)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            members.push(TeamMemberResponse {
                id: member_id,
                name: member.name.clone(),
                tms_id: member.tms_id.clone(),
                email: member.email.clone(),
                is_team_lead: member.is_team_lead,
            });
        }

        tx.commit().await.map_err(|e| e.to_string())?;

        // Fetch the created team with timestamps
        let team_row = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, team_name, idea_description, impact_description, status, admin_remarks, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
        )
        .bind(&team_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(TeamResponse {
            id: team_row.0,
            team_name: team_row.1,
            idea_description: team_row.2,
            impact_description: team_row.3,
            status: team_row.4.into(),
            admin_remarks: team_row.5,
            members,
            created_at: team_row.6,
            updated_at: team_row.7,
        })
    }

    async fn get_team_by_id(&self, id: Uuid) -> Result<Option<TeamResponse>, String> {
        let team_row = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, team_name, idea_description, impact_description, status, admin_remarks, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(team_row) = team_row {
            let members = sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
                r#"
                SELECT id, name, tms_id, email, is_team_lead
                FROM team_members
                WHERE team_id = $1
                ORDER BY is_team_lead DESC, name ASC
                "#,
            )
            .bind(&id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|row| TeamMemberResponse {
                id: row.0,
                name: row.1,
                tms_id: row.2,
                email: row.3,
                is_team_lead: row.4,
            })
            .collect();

            Ok(Some(TeamResponse {
                id: team_row.0,
                team_name: team_row.1,
                idea_description: team_row.2,
                impact_description: team_row.3,
                status: team_row.4.into(),
                admin_remarks: team_row.5,
                members,
                created_at: team_row.6,
                updated_at: team_row.7,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_team_by_name(&self, name: &str) -> Result<Option<TeamResponse>, String> {
        let team_row = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, team_name, idea_description, impact_description, status, admin_remarks, created_at, updated_at
            FROM teams
            WHERE team_name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(team_row) = team_row {
            let members = sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
                r#"
                SELECT id, name, tms_id, email, is_team_lead
                FROM team_members
                WHERE team_id = $1
                ORDER BY is_team_lead DESC, name ASC
                "#,
            )
            .bind(&team_row.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|row| TeamMemberResponse {
                id: row.0,
                name: row.1,
                tms_id: row.2,
                email: row.3,
                is_team_lead: row.4,
            })
            .collect();

            Ok(Some(TeamResponse {
                id: team_row.0,
                team_name: team_row.1,
                idea_description: team_row.2,
                impact_description: team_row.3,
                status: team_row.4.into(),
                admin_remarks: team_row.5,
                members,
                created_at: team_row.6,
                updated_at: team_row.7,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_teams(&self, query: &ListTeamsQuery) -> Result<PaginatedTeamsResponse, String> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10).min(100); // Max 100 per page
        let offset = (page - 1) * page_size;

        // Build query based on filters
        let (total, teams_rows) = match (&query.search, &query.team_name) {
            (Some(search), Some(team_name)) if !search.is_empty() && !team_name.is_empty() => {
                let search_pattern = format!("%{}%", search);
                let team_name_pattern = format!("%{}%", team_name);
                
                let total: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*)
                    FROM teams t
                    WHERE (t.team_name ILIKE $1 OR t.idea_description ILIKE $1 OR t.impact_description ILIKE $1)
                    AND t.team_name ILIKE $2
                    "#,
                )
                .bind(&search_pattern)
                .bind(&team_name_pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                let teams = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
                    r#"
                    SELECT t.id, t.team_name, t.idea_description, t.impact_description, t.status, t.admin_remarks, t.created_at, t.updated_at
                    FROM teams t
                    WHERE (t.team_name ILIKE $1 OR t.idea_description ILIKE $1 OR t.impact_description ILIKE $1)
                    AND t.team_name ILIKE $2
                    ORDER BY t.created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(&search_pattern)
                .bind(&team_name_pattern)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                (total, teams)
            }
            (Some(search), _) if !search.is_empty() => {
                let search_pattern = format!("%{}%", search);
                
                let total: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*)
                    FROM teams t
                    WHERE t.team_name ILIKE $1 OR t.idea_description ILIKE $1 OR t.impact_description ILIKE $1
                    "#,
                )
                .bind(&search_pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                let teams = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
                    r#"
                    SELECT t.id, t.team_name, t.idea_description, t.impact_description, t.status, t.admin_remarks, t.created_at, t.updated_at
                    FROM teams t
                    WHERE t.team_name ILIKE $1 OR t.idea_description ILIKE $1 OR t.impact_description ILIKE $1
                    ORDER BY t.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(&search_pattern)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                (total, teams)
            }
            (_, Some(team_name)) if !team_name.is_empty() => {
                let team_name_pattern = format!("%{}%", team_name);
                
                let total: i64 = sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*)
                    FROM teams t
                    WHERE t.team_name ILIKE $1
                    "#,
                )
                .bind(&team_name_pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                let teams = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
                    r#"
                    SELECT t.id, t.team_name, t.idea_description, t.impact_description, t.status, t.admin_remarks, t.created_at, t.updated_at
                    FROM teams t
                    WHERE t.team_name ILIKE $1
                    ORDER BY t.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(&team_name_pattern)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                (total, teams)
            }
            _ => {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM teams")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;

                let teams = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
                    r#"
                    SELECT t.id, t.team_name, t.idea_description, t.impact_description, t.status, t.admin_remarks, t.created_at, t.updated_at
                    FROM teams t
                    ORDER BY t.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                (total, teams)
            }
        };

        // Fetch members for each team
        let mut team_responses = Vec::new();
        for team_row in teams_rows {
            let members = sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
                r#"
                SELECT id, name, tms_id, email, is_team_lead
                FROM team_members
                WHERE team_id = $1
                ORDER BY is_team_lead DESC, name ASC
                "#,
            )
            .bind(&team_row.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|row| TeamMemberResponse {
                id: row.0,
                name: row.1,
                tms_id: row.2,
                email: row.3,
                is_team_lead: row.4,
            })
            .collect();

            team_responses.push(TeamResponse {
                id: team_row.0,
                team_name: team_row.1,
                idea_description: team_row.2,
                impact_description: team_row.3,
                status: team_row.4.into(),
                admin_remarks: team_row.5,
                members,
                created_at: team_row.6,
                updated_at: team_row.7,
            });
        }

        let total_pages = (total as f64 / page_size as f64).ceil() as u32;

        Ok(PaginatedTeamsResponse {
            teams: team_responses,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    async fn check_member(&self, request: &CheckMemberRequest) -> Result<CheckMemberResponse, String> {
        let result = if let Some(email) = &request.email {
            sqlx::query_as::<_, (Uuid, String)>(
                r#"
                SELECT t.id, t.team_name
                FROM teams t
                INNER JOIN team_members tm ON tm.team_id = t.id
                WHERE tm.email = $1
                LIMIT 1
                "#,
            )
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?
        } else if let Some(tms_id) = &request.tms_id {
            sqlx::query_as::<_, (Uuid, String)>(
                r#"
                SELECT t.id, t.team_name
                FROM teams t
                INNER JOIN team_members tm ON tm.team_id = t.id
                WHERE tm.tms_id = $1
                LIMIT 1
                "#,
            )
            .bind(tms_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?
        } else {
            return Err("Either email or tms_id must be provided".to_string());
        };

        if let Some((team_id, team_name)) = result {
            Ok(CheckMemberResponse {
                is_registered: true,
                team_name: Some(team_name),
                team_id: Some(team_id),
            })
        } else {
            Ok(CheckMemberResponse {
                is_registered: false,
                team_name: None,
                team_id: None,
            })
        }
    }

    async fn count_teams(&self) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM teams")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(count)
    }

    async fn member_exists_in_team(&self, email: &str, tms_id: &str) -> Result<bool, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM team_members
            WHERE email = $1 OR tms_id = $2
            "#,
        )
        .bind(email)
        .bind(tms_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count > 0)
    }

    async fn update_team_status(&self, id: Uuid, status: String, remarks: Option<String>) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE teams
            SET status = $1, admin_remarks = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(status)
        .bind(remarks)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

