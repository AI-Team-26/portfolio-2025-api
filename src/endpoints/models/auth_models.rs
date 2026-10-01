pub mod signup {
    use crate::endpoints::models::common;

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub username: String,
        pub password: String,
        pub currency_id: i32,
    }

    pub type Response = common::SuccessErrorResponse;
}

pub mod login {
    use crate::{entities::session::Session, utils::datetime::UtcDateTime};

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub username: String,
        pub password: String,
    }

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub access_token: String,
        pub access_token_expires_at: UtcDateTime,
        pub refresh_token: String,
        pub refresh_token_expires_at: UtcDateTime,
        pub user: ResponseUser,
    }

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResponseUser {
        pub id: String,
        pub username: String,
    }

    impl From<Session> for Response {
        fn from(session: Session) -> Self {
            Response {
                access_token: session.access_token,
                refresh_token: session.refresh_token,
                access_token_expires_at: session.access_token_expires_at,
                refresh_token_expires_at: session.refresh_token_expires_at,
                user: ResponseUser {
                    id: session.user.id,
                    username: session.user.username,
                },
            }
        }
    }
}

pub mod refresh_token {
    use crate::{
        repositories::schemas::session_record::SessionRecord, utils::datetime::UtcDateTime,
    };

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub refresh_token: String,
    }

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub access_token: String,
        pub access_token_expires_at: UtcDateTime,
        pub refresh_token: String,
        pub refresh_token_expires_at: UtcDateTime,
    }

    impl From<SessionRecord> for Response {
        fn from(session: SessionRecord) -> Self {
            Response {
                access_token: session.access_token,
                refresh_token: session.refresh_token,
                access_token_expires_at: session.access_token_expires_at,
                refresh_token_expires_at: session.refresh_token_expires_at,
            }
        }
    }
}
