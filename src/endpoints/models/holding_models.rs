mod upsert {
    use rust_decimal::Decimal;

    use crate::{
        repositories::schemas::holding_record::HoldingRecord, utils::datetime::UtcDateTime,
    };

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub custodian_id: i32,
        pub currency_id: i32,

        pub date: UtcDateTime,
        pub action: String,
        pub amount: Decimal,
        pub note: Option<String>,
    }

    impl Request {
        pub fn into_record(self, id: Option<i32>, user_id: &str) -> HoldingRecord {
            HoldingRecord {
                id: id.unwrap_or_default(),
                user_id: user_id.to_string(),
                custodian_id: self.custodian_id,
                currency_id: self.currency_id,
                date: self.date,
                action: self.action,
                amount: self.amount,
                note: self.note,
            }
        }
    }
}

pub mod create {
    use crate::{
        endpoints::models::holding_models::upsert,
        repositories::schemas::holding_record::HoldingRecord,
    };

    pub type Request = upsert::Request;

    impl From<(Request, &str)> for HoldingRecord {
        fn from((request, user_id): (Request, &str)) -> HoldingRecord {
            request.into_record(None, user_id)
        }
    }
}

pub mod update {
    use crate::{
        endpoints::models::holding_models::upsert,
        repositories::schemas::holding_record::HoldingRecord,
    };

    pub type Request = upsert::Request;

    impl From<(i32, Request, &str)> for HoldingRecord {
        fn from((id, request, user_id): (i32, Request, &str)) -> HoldingRecord {
            request.into_record(Some(id), user_id)
        }
    }
}

pub mod search {

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Params {
        pub only_latest_balance: bool,
    }

    use crate::{
        repositories::schemas::holding_record::HoldingRecord, utils::datetime::UtcDateTime,
    };
    use rust_decimal::Decimal;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub id: i32,
        pub custodian_id: i32,
        pub currency_id: i32,
        pub date: UtcDateTime,
        pub action: String,
        pub amount: Decimal,
        pub note: Option<String>,
        pub amount_in_main_currency: Option<Decimal>,
    }

    impl From<(HoldingRecord, Option<Decimal>)> for Response {
        fn from((record, amount_in_main_currency): (HoldingRecord, Option<Decimal>)) -> Self {
            Self {
                id: record.id,
                custodian_id: record.custodian_id,
                currency_id: record.currency_id,
                date: record.date,
                action: record.action,
                amount: record.amount,
                note: record.note,
                amount_in_main_currency,
            }
        }
    }
}
