/*
#[derive(sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRecord {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub kind: String,
    pub is_active: bool,
    pub precision: i16,
    pub coingecko_id: String,
}
*/

#[derive(sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyOfUserRecord {
    pub id: i32,
    pub user_id: String,
    pub currency_id: i32,
}
