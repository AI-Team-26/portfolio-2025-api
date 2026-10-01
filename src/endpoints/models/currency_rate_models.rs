use crate::{
    repositories::schemas::currency_rate_record::CurrencyRateRecord, utils::datetime::Date,
};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRate {
    pub base_currency_id: i32,
    pub quote_currency_id: i32,
    pub date: Date,
    pub rate: Decimal,
    pub source: String,
}

impl From<CurrencyRateRecord> for CurrencyRate {
    fn from(record: CurrencyRateRecord) -> Self {
        CurrencyRate {
            base_currency_id: record.base_currency_id,
            quote_currency_id: record.quote_currency_id,
            date: record.date,
            rate: record.rate,
            source: record.source,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AtDateQuery {
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct SinglePairQuery {
    pub base: i32,
    pub quote: i32,
    pub date: Option<String>,
}
