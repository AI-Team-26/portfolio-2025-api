use crate::{
    repositories::helpers::parse_decimal,
    utils::datetime::{Date, UtcDateTime},
};
use rust_decimal::Decimal;
use sqlx::FromRow;
use sqlx::Row;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRateRecord {
    pub base_currency_id: i32,
    pub quote_currency_id: i32,
    pub date: Date,
    pub source: String,
    pub rate: Decimal,
    pub created_at: UtcDateTime,
}

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for CurrencyRateRecord {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(CurrencyRateRecord {
            base_currency_id: row.get("base_currency_id"),
            quote_currency_id: row.get("quote_currency_id"),
            date: row.get("date"),
            source: row.get("source"),
            rate: parse_decimal(row.get("rate"))?,
            created_at: row.get("created_at"),
        })
    }
}

impl CurrencyRateRecord {
    pub fn display(&self) -> String {
        format!("{}/{}", self.base_currency_id, self.quote_currency_id)
    }
}
