use crate::{
    entities::user::User, repositories::schemas::session_record::SessionRecord,
    utils::datetime::UtcDateTime,
};

#[derive(Clone)]
pub struct Session {
    pub id: i32,
    pub user: User,
    pub access_token: String,
    pub access_token_expires_at: UtcDateTime,
    pub refresh_token: String,
    pub refresh_token_expires_at: UtcDateTime,
    pub created_at: UtcDateTime,
    pub last_access_at: Option<UtcDateTime>,
    pub last_refresh_at: Option<UtcDateTime>,
    pub creation_ip_address: String,
    pub creation_user_agent: String,
}

impl From<(SessionRecord, User)> for Session {
    fn from((record, user): (SessionRecord, User)) -> Self {
        Session {
            id: record.id,
            user,
            access_token: record.access_token,
            access_token_expires_at: record.access_token_expires_at,
            refresh_token: record.refresh_token,
            refresh_token_expires_at: record.refresh_token_expires_at,
            created_at: record.created_at,
            last_access_at: record.last_access_at,
            last_refresh_at: record.last_refresh_at,
            creation_ip_address: record.creation_ip_address,
            creation_user_agent: record.creation_user_agent,
        }
    }
}

impl Session {
    /*
    pub fn is_access_token_active(&self) -> bool {
        self.access_token_expires_at > now()
    }

    pub fn is_refresh_token_active(&self) -> bool {
        self.refresh_token_expires_at > now()
    }
    */
}
