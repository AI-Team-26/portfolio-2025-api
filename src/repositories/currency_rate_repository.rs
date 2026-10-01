use crate::{
    repositories::{helpers::from_rust_decimal, schemas::currency_rate_record::CurrencyRateRecord},
    utils::datetime::Date,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CurrencyRateRepository {
    db_pool: PgPool, // PgPool is internally reference-counted and designed to be cloned cheaply.
}

impl CurrencyRateRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, record: &CurrencyRateRecord) -> Result<(), String> {
        sqlx::query!(
            // Postgres UPSERT
            r#"
                INSERT INTO CurrencyRates (base_currency_id, quote_currency_id, date, source, rate)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (base_currency_id, quote_currency_id, date, source) DO UPDATE SET
                    rate = EXCLUDED.rate,
                    created_at = CURRENT_TIMESTAMP
            "#,
            record.base_currency_id,
            record.quote_currency_id,
            record.date,
            record.source,
            from_rust_decimal(record.rate)?
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn search(
        &self,
        base_currency_id: i32,
        quote_currency_id: i32,
        date: Option<Date>,
    ) -> Result<Vec<CurrencyRateRecord>, String> {
        let rates = sqlx::query_as::<_, CurrencyRateRecord>(
            r#"
            SELECT base_currency_id, quote_currency_id, date, source, rate, created_at
            FROM CurrencyRates
            WHERE base_currency_id = $1 AND quote_currency_id = $2 AND ($3::DATE IS NULL OR date = $3)
            "#)
            .bind(base_currency_id)
            .bind(quote_currency_id)
            .bind(date)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e:sqlx::Error| e.to_string())?;

        Ok(rates)
    }

    pub async fn list_at_date(&self, date: Date) -> Result<Vec<CurrencyRateRecord>, String> {
        let rates = sqlx::query_as::<_, CurrencyRateRecord>(
            r#"
            SELECT base_currency_id, quote_currency_id, date, source, rate::numeric, created_at
            FROM CurrencyRates
            WHERE date = $1
            "#,
        )
        .bind(date)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

        Ok(rates)
    }
}
