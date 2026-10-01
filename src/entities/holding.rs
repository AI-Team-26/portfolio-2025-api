use crate::{
    entities::{currency::Currency, custodian::Custodian, user::User},
    repositories::schemas::holding_record::HoldingRecord,
    utils::datetime::UtcDateTime,
};
use rust_decimal::Decimal;
use sqlx::Type;

pub struct Holding {
    pub id: i32,

    pub user: User,
    pub custodian: Custodian,
    pub currency: Currency,

    pub date: UtcDateTime,
    pub action: HoldingAction,
    pub amount: Decimal,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Type, serde::Serialize)]
pub enum HoldingAction {
    #[sqlx(rename = "Balance At")]
    BalanceAt, // A snapshot as reported at a date
    Deposit,
    //Withdrawal,
    //Adjustment,
}

impl From<Holding> for HoldingRecord {
    fn from(entity: Holding) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user.id,
            custodian_id: entity.custodian.id,
            currency_id: entity.currency.id,
            date: entity.date,
            action: entity.action.to_string(),
            amount: entity.amount,
            note: entity.note,
        }
    }
}

impl HoldingAction {
    /*
    pub fn from_string(action: &str) -> Result<Self, String> {
        match action {
            "Balance At" => Ok(HoldingAction::BalanceAt),
            "Deposit" => Ok(HoldingAction::Deposit),
            _ => Err(format!("Invalid holding action: '{}'", action)),
        }
    }*/

    pub fn to_string(&self) -> String {
        match self {
            HoldingAction::BalanceAt => "Balance At".to_string(),
            HoldingAction::Deposit => "Depositt".to_string(),
        }
    }
}
