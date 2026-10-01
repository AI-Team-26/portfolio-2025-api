use crate::repositories::schemas::session_record::{
    SessionRecord, SessionWithUser, UpdateForAccess, UpdateForRefresh,
};
use crate::warn;
use sqlx::PgPool;
#[derive(Clone)]
pub struct SessionRepository {
    db_pool: PgPool,
}

impl SessionRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, item: SessionRecord) -> Result<i32, String> {
        let row = sqlx::query!(
            r#"
            INSERT INTO Sessions (user_id, access_token, access_token_expires_at, refresh_token, refresh_token_expires_at, created_at, creation_ip_address, creation_user_agent)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id
            "#,
            item.user_id,
            item.access_token,
            item.access_token_expires_at,
            item.refresh_token,
            item.refresh_token_expires_at,
            item.created_at,
            item.creation_ip_address,
            item.creation_user_agent
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.id)
    }

    pub async fn update_for_access(
        &self,
        update: UpdateForAccess,
    ) -> Result<Option<SessionWithUser>, String> {
        warn!("update_for_access");
        sqlx::query_as!(
            SessionWithUser,
            r#"
            UPDATE Sessions
            SET 
                access_token_expires_at = $2,
                refresh_token_expires_at = $3,
                last_access_at = $4
            FROM Users
            WHERE Sessions.access_token = $1
            AND Sessions.access_token_expires_at > now()
            AND Users.id = Sessions.user_id
            RETURNING
                Sessions.user_id,
                Users.username,
                Sessions.access_token_expires_at,
                Sessions.refresh_token_expires_at
            "#,
            update.access_token,
            update.access_token_expires_at,
            update.refresh_token_expires_at,
            update.last_access_at
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn update_for_refresh(
        &self,
        update: UpdateForRefresh,
    ) -> Result<Option<SessionRecord>, String> {
        warn!("update_for_refresh");
        sqlx::query_as!(
            SessionRecord,
            r#"
            Update Sessions SET 
                access_token = $2,
                access_token_expires_at = $3,
                refresh_token = $4, 
                refresh_token_expires_at = $5, 
                last_refresh_at = $6
            WHERE refresh_token = $1 
                AND refresh_token_expires_at > now()
            RETURNING id, user_id, access_token, access_token_expires_at, refresh_token, refresh_token_expires_at, created_at, last_access_at, last_refresh_at, creation_ip_address, creation_user_agent
            "#,
            update.old_refresh_token,
            update.access_token,
            update.access_token_expires_at,
            update.refresh_token,
            update.refresh_token_expires_at,
            update.last_refresh_at
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| e.to_string())
    }
    /*
    pub async fn find_by_access_token(&self, access_token: &str) -> Result<Option<SessionRecord>, String> {
        //let _ = sqlx::query_as!(SessionRecord, "SELECT id, access_token, access_token_expires_at, refresh_token, refresh_token_expires_at  FROM Sessions");
        sqlx::query_as!(
            SessionRecord,
            r#"
            SELECT id, user_id, access_token, access_token_expires_at, refresh_token, refresh_token_expires_at, created_at, creation_ip_address, creation_user_agent
            FROM Sessions WHERE access_token = $1
            "#,
            access_token)
                .fetch_optional(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Session by access token. {}", e))
    }
    */

    pub async fn exists_by_refresh_token(&self, refresh_token: &str) -> Result<bool, String> {
        // First check if the record exists
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM Sessions WHERE TRIM(refresh_token) = TRIM($1))"#,
            refresh_token
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| format!("Failed to check if session exists: {}", e))?;

        if !exists.unwrap_or(false) {
            return Ok(false);
        }

        Ok(true)
    }
    pub async fn find_by_refresh_token(
        &self,
        _refresh_token: &str,
    ) -> Result<Option<SessionRecord>, String> {
        sqlx::query_as!(
            SessionRecord,
            r#"
            SELECT id, user_id, access_token, access_token_expires_at, refresh_token, refresh_token_expires_at, created_at, last_access_at, last_refresh_at, creation_ip_address, creation_user_agent
            FROM Sessions WHERE id = 72           
            "#)
                .fetch_optional(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Session by refresh token. {}", e))
    }
}
