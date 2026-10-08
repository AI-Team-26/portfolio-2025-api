use crate::{
    repositories::{helpers::from_rust_decimal, schemas::currency_rate_record::CurrencyRateRecord},
    utils::datetime::Date,
};
use sqlx::{types::BigDecimal, PgPool, Postgres, QueryBuilder};

#[derive(Clone)]
pub struct CurrencyRateRepository {
    db_pool: PgPool, // PgPool is internally reference-counted and designed to be cloned cheaply.
}

impl CurrencyRateRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Bulk upsert of all records in a single round-trip.
    /// Uses DO UPDATE on conflict: periodic reruns refresh today's rates instead of keeping stale values.
    /// Requires unique (base, quote, date, source) keys within the batch, guaranteed by the caller building from keyed maps.
    pub async fn create_many(&self, records: &[CurrencyRateRecord]) -> Result<usize, String> {
        if records.is_empty() {
            return Ok(0);
        }

        // convert upfront so the SQL-building closure stays infallible
        let rates: Vec<BigDecimal> = records
            .iter()
            .map(|r| from_rust_decimal(r.rate))
            .collect::<Result<Vec<_>, _>>()?;

        let mut qb = QueryBuilder::<Postgres>::new(
            "INSERT INTO CurrencyRates (base_currency_id, quote_currency_id, date, source, rate)",
        );
        qb.push_values(
            records.iter().zip(rates.iter()),
            |mut values, (record, rate)| {
                values
                    .push_bind(record.base_currency_id)
                    .push_bind(record.quote_currency_id)
                    .push_bind(record.date)
                    .push_bind(record.source.as_str())
                    .push_bind(rate);
            },
        );
        qb.push(
            "ON CONFLICT (base_currency_id, quote_currency_id, date, source) DO UPDATE SET \
             rate = EXCLUDED.rate, created_at = CURRENT_TIMESTAMP",
        );

        qb.build()
            .execute(&self.db_pool)
            .await
            .map(|result| result.rows_affected() as usize)
            .map_err(|e| e.to_string())
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
