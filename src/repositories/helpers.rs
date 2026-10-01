use rust_decimal::Decimal;
use sqlx::types::BigDecimal;
use std::str::FromStr;

pub fn from_rust_decimal(d: Decimal) -> Result<BigDecimal, String> {
    BigDecimal::from_str(&d.to_string())
        .map_err(|e| format!("Failed to convert Decimal '{}' to BigDecimal. {}", d, e))
}

pub fn to_rust_decimal(bd: BigDecimal) -> Result<Decimal, String> {
    Decimal::from_str(&bd.to_string())
        .map_err(|e| format!("Failed to convert BigDecimal '{}' to Decimal. {}", bd, e))
}

pub fn parse_decimal(value: Option<BigDecimal>) -> Result<Decimal, sqlx::Error> {
    match value {
        Some(bd) => to_rust_decimal(bd).map_err(sqlx::Error::InvalidArgument),
        None => Ok(Decimal::ZERO), // or return an error if NULL is not allowed
    }
}
