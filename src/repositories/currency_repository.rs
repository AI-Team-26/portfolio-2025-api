use crate::entities::currency::Currency;
use crate::entities::currency::CurrencyKind;
use sqlx::PgPool;

#[derive(Clone)]
pub struct CurrencyRepository {
    db_pool: PgPool, // PgPool is internally reference-counted and designed to be cloned cheaply.
}

impl CurrencyRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, currency: Currency) -> Result<i32, String> {
        let row = sqlx::query!(
            r#"
                INSERT INTO Currency (symbol, name, kind, is_active, precision, is_major, coingecko_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            "#,
            currency.symbol,
            currency.name,
            currency.kind as CurrencyKind,
            currency.is_active,
            currency.precision,
            currency.is_major,
            currency.coingecko_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.id)
    }

    pub async fn update(&self, currency: &Currency) -> Result<(), String> {
        //let _test = sqlx::query!("SELECT COUNT(*) FROM Currency").fetch_one(&self.db_pool).await;
        let result = sqlx::query!(
            r#"
                UPDATE Currency 
                SET symbol = $1, name = $2, kind = $3, is_active = $4, precision = $5, is_major = $6, coingecko_id = $7
                WHERE id = $8
            "#,
            currency.symbol,
            currency.name,
            currency.kind.clone() as CurrencyKind,
            currency.is_active,
            currency.precision,
            currency.is_major,
            currency.coingecko_id,
            currency.id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("No currency updated".to_string());
        }
        Ok(())
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        sqlx::query!(
            r#"
                delete from Currency WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        // no need to check rows affected because if 0 it was not found because already deleted
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Currency>, String> {
        let currencies = sqlx::query_as!(Currency,
            r#"
            SELECT id, symbol, name, kind as "kind!: CurrencyKind", is_active, precision, is_major, coingecko_id
            FROM Currency
            "#)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e:sqlx::Error| e.to_string())?;

        Ok(currencies)
    }
}
