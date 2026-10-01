use chrono::{DateTime, Utc};
//#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub hashed_password: String,
    pub creation_date: DateTime<Utc>,
    pub currency_id: i32,
    pub role: String,
}
