mod upsert {
    use crate::entities::custodian::{Custodian, CustodianKind};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub name: String,
        pub custodian: String,
        pub account: Option<String>,
        pub kind: String,
        pub color_code: String,
        pub description: Option<String>,
    }

    /*impl From<(Request, Option<i32>, &str)> for Custodian {
        fn from((request, id, user_id): (Request, Option<i32>, &str)) -> Result<Custodian, String> {
            Ok(Custodian {
                id: id.unwrap_or_default(),
                user_id: user_id.to_string(),
                name: request.name,
                custodian: request.custodian,
                account: request.account,
                kind: CustodianKind::from_string(&request.kind)?,
                color_code: request.color_code,
                description: request.description
            })
        }
    }*/

    // TODO: enum issue, it forces to return a Result
    impl Request {
        pub fn to_entity(self, id: Option<i32>, user_id: String) -> Result<Custodian, String> {
            Ok(Custodian {
                id: id.unwrap_or_default(),
                user_id,
                name: self.name,
                custodian: self.custodian,
                account: self.account,
                kind: CustodianKind::from_string(&self.kind)?,
                color_code: self.color_code,
                description: self.description,
            })
        }
    }
}

pub mod create {
    use crate::{endpoints::models::custodian_models::upsert, entities::custodian::Custodian};

    //pub type Request = upsert::Request; //with an alias I cannot override to_entity

    // Newtype for create requests
    #[derive(serde::Deserialize)]
    #[serde(transparent)] // This makes deserialization work directly into the inner field
    pub struct Request(pub upsert::Request);

    impl Request {
        pub fn to_entity(self, user_id: String) -> Result<Custodian, String> {
            self.0.to_entity(None, user_id) // None
        }
    }
}

pub mod update {
    use crate::{endpoints::models::custodian_models::upsert, entities::custodian::Custodian};

    #[derive(serde::Deserialize)]
    #[serde(transparent)]
    pub struct Request(pub upsert::Request);

    impl Request {
        pub fn to_entity(self, id: i32, user_id: String) -> Result<Custodian, String> {
            self.0.to_entity(Some(id), user_id) // real id
        }
    }
}
