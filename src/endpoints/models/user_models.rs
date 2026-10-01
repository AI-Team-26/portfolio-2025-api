pub mod update {
    #[derive(serde::Deserialize, validator::Validate)]
    #[serde(rename_all = "camelCase")]
    pub struct Request {
        pub currency_id: i32,
    }
}
