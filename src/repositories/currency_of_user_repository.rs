use crate::repositories::errors::DatabaseError;
use crate::repositories::schemas::currency_record::CurrencyOfUserRecord;
use sqlx::PgPool;

#[derive(Clone)]
pub struct CurrencyOfUserRepository {
    db_pool: PgPool, // PgPool is internally reference-counted and designed to be cloned cheaply.
}

impl CurrencyOfUserRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<CurrencyOfUserRecord>, DatabaseError> {
        let items = sqlx::query_as!(
            CurrencyOfUserRecord,
            "SELECT id, user_id, currency_id FROM CurrenciesOfUser WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| DatabaseError::generic(format!("Failed to get Currencies of user. {e}")))?;

        Ok(items)
    }

    pub async fn create(&self, user_id: String, currency_id: i32) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
                INSERT INTO CurrenciesOfUser (user_id, currency_id)
                VALUES ($1, $2)
            "#,
            user_id,
            currency_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| DatabaseError::generic(format!("Failed to create CurrencyOfUser. {e}")))?;

        Ok(())
    }

    pub async fn delete(&self, user_id: String, currency_id: i32) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
                delete from CurrenciesOfUser WHERE user_id = $1 AND currency_id = $2
            "#,
            user_id,
            currency_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| DatabaseError::generic(format!("Failed to delete CurrencyOfUser. {e}")))?;

        // no need to check rows affected because if 0 it was not found because already deleted
        Ok(())
    }
}
