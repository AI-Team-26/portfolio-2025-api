//use sqlx::postgres::PgArguments;
use crate::repositories::errors::DatabaseError;
use crate::repositories::helpers::{from_rust_decimal, to_rust_decimal};
use crate::repositories::repository_traits::BaseRepository;
use crate::repositories::schemas::holding_record::HoldingRecord;
use sqlx::PgPool;

#[derive(Clone)]
pub struct HoldingRepository {
    db_pool: PgPool,
}

impl BaseRepository for HoldingRepository {}

impl HoldingRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, record: HoldingRecord) -> Result<i32, String> {
        // let _ = sqlx::query!("SELECT id, username, role FROM usholdings WHERE id = $1", user.id); // used to "refresh" SQLx checks
        let row = sqlx::query!(
            r#"
                INSERT INTO Holdings (user_id, custodian_id, currency_id, date, action, amount, note)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            "#,
            record.user_id,
            record.custodian_id,
            record.currency_id,
            record.date,
            record.action,
            from_rust_decimal(record.amount)?,
            record.note
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.id)
    }

    pub async fn update(&self, record: HoldingRecord) -> Result<(), DatabaseError> {
        let result = sqlx::query!(
            r#"
                UPDATE Holdings SET
                    custodian_id = $3,
                    currency_id = $4,
                    date = $5,
                    action = $6,
                    amount = $7,
                    note = $8                
                WHERE id = $1 AND user_id = $2
            "#,
            record.id,
            record.user_id,
            record.custodian_id,
            record.currency_id,
            record.date,
            record.action,
            from_rust_decimal(record.amount).map_err(|e| DatabaseError::generic(e))?,
            record.note
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| DatabaseError::generic(e.to_string()))?;

        self.check_result(result)
    }

    pub async fn delete(&self, id: i32, user_id: &str) -> Result<(), DatabaseError> {
        let result = sqlx::query!(
            r#"delete from Holdings where id = $1 and user_id = $2"#,
            id,
            user_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| DatabaseError::generic(e.to_string()))?;

        self.check_result(result)
    }

    pub async fn single_for_user(&self, id: i32, user_id: &str) -> Result<HoldingRecord, String> {
        let row =
            sqlx::query!(
                "SELECT id, user_id, custodian_id, currency_id, date, action, amount, note FROM Holdings 
                WHERE user_id=$1 AND id=$2 ",
                    user_id, id)
                .fetch_one(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Holdings of user. {}", e))?;

        let record = HoldingRecord {
            id: row.id,
            user_id: row.user_id,
            custodian_id: row.custodian_id,
            currency_id: row.currency_id,
            date: row.date,
            action: row.action,
            amount: to_rust_decimal(row.amount.ok_or("Amount is NULL")?)?,
            note: row.note,
        };

        Ok(record)
    }

    /*async fn execute_query_for_list(&self, query: sqlx::query::Query<'_, Postgres, PgArguments>) {
        let rows = query.fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Failed to get Holdings of user. {}", e))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(HoldingRecord {
                id: row.id,
                user_id: row.user_id,
                custodian_id: row.custodian_id,
                currency_id: row.currency_id,
                date: row.date,
                action: row.action,
                amount: to_rust_decimal(row.amount.ok_or("Amount is NULL")?)?,
                note: row.note,
            });
        }

        Ok(items)
    }*/

    /*async fn execute_query_for_list(&self, query: &str) {
        let rows =
            sqlx::query("")
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Holdings of user. {}", e))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(HoldingRecord {
                id: row.id,
                user_id: row.user_id,
                custodian_id: row.custodian_id,
                currency_id: row.currency_id,
                date: row.date,
                action: row.action,
                amount: to_rust_decimal(row.amount.ok_or("Amount is NULL")?)?,
                note: row.note,
            });
        }

        Ok(items)
    }*/

    pub async fn list_last_balance(&self, user_id: &str) -> Result<Vec<HoldingRecord>, String> {
        let query = sqlx::query!(
            "SELECT DISTINCT ON (custodian_id, currency_id) 
                id, user_id, custodian_id, currency_id, date, action, amount, note
            FROM holdings
            WHERE user_id = $1
            ORDER BY custodian_id, currency_id, date DESC;",
            user_id
        );

        let rows = query
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Failed to get Holdings of user. {}", e))?;

        //self.execute_query_for_list(&query)

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(HoldingRecord {
                id: row.id,
                user_id: row.user_id,
                custodian_id: row.custodian_id,
                currency_id: row.currency_id,
                date: row.date,
                action: row.action,
                amount: to_rust_decimal(row.amount.ok_or("Amount is NULL")?)?,
                note: row.note,
            });
        }

        Ok(items)
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<HoldingRecord>, String> {
        /* SQLx uses its BigDecimal type... instead of rust_decimal::Decimal, so a manual mapping is required */
        /*
        sqlx::query_as!(
            HoldingRecord,
            "SELECT id, user_id, custodian_id, currency_id, date, action, amount, note FROM Holdings WHERE user_id = $1",
            user_id)
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Holdings of user. {}", e))
        */

        let rows =
            sqlx::query!(
                "SELECT id, user_id, custodian_id, currency_id, date, action, amount, note FROM Holdings WHERE user_id = $1",
                    user_id)
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("Failed to get Holdings of user. {}", e))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(HoldingRecord {
                id: row.id,
                user_id: row.user_id,
                custodian_id: row.custodian_id,
                currency_id: row.currency_id,
                date: row.date,
                action: row.action,
                amount: to_rust_decimal(row.amount.ok_or("Amount is NULL")?)?,
                note: row.note,
            });
        }

        Ok(items)
    }
}
