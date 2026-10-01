use crate::repositories::errors::DatabaseError;
use rust_decimal::Decimal;
use sqlx::types::BigDecimal;
use std::str::FromStr;

pub fn from_rust_decimal(d: Decimal) -> Result<BigDecimal, DatabaseError> {
    BigDecimal::from_str(&d.to_string())
        .map_err(|e| format!("Failed to convert Decimal '{}' to BigDecimal. {}", d, e).into())
}

pub fn to_rust_decimal(bd: BigDecimal) -> Result<Decimal, DatabaseError> {
    Decimal::from_str(&bd.to_string())
        .map_err(|e| format!("Failed to convert BigDecimal '{}' to Decimal. {}", bd, e).into())
}

pub fn parse_decimal_for_sqlx(value: Option<BigDecimal>) -> Result<Decimal, sqlx::Error> {
    match value {
        Some(bd) => Decimal::from_str(&bd.to_string())
            .map_err(|e| sqlx::Error::Decode(Box::new(e))),
        None => Ok(Decimal::ZERO),
    }
}