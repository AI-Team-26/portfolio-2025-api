use crate::{entities::currency::Currency, utils::datetime::UtcDateTime};

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub hashed_password: String,
    pub creation_date: UtcDateTime,
    pub currency: Currency,
    pub role: String,
}

impl User {
    pub fn _is_admin(&self) -> bool {
        self.role == "Admin"
    }
}
