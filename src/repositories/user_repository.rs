use crate::entities::user::User;
use crate::repositories::schemas::user_record::UserRecord;
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserRepository {
    db_pool: PgPool,
}

impl UserRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, user: User) -> Result<(), String> {
        // let _ = sqlx::query!("SELECT id, username, role FROM users WHERE id = $1", user.id); // used to "refresh" SQLx checks
        sqlx::query!(
            r#"
                INSERT INTO Users (id, username, hashed_password, creation_date, currency_id, role)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.username,
            user.hashed_password,
            user.creation_date,
            user.currency.id,
            user.role
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<UserRecord>, String> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, hashed_password, creation_date, currency_id, role
            FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| format!("Failed to get User by id. {}", e))
    }

    pub async fn find_by_username(&self, username: String) -> Result<Option<UserRecord>, String> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, hashed_password, creation_date, currency_id, role 
            FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| format!("Failed to get User by username. {}", e))
    }

    pub async fn update_currency(&self, user_id: String, currency_id: i32) -> Result<(), String> {
        sqlx::query!(
            r#"
                UPDATE users
                SET currency_id = $1
                WHERE id = $2
            "#,
            currency_id,
            user_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| format!("Failed to update user currency. {}", e))?;

        Ok(())
    }
}
