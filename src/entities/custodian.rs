use sqlx::{FromRow, Type};

pub const KINDS: &[&str] = &[
    "Bank",              // HSBC, Revolut
    "Exchange",          // Binance
    "Fintech Platform",  // Wise, Nexo
    "Pension",           // Aviva
    "Blockchain Wallet", // Ethereum address, Bitcoin address, Radix account
    "Broker",            // Trading 212, eToro
    "Payment Platform",  // PayPal, Stripe
    "Other",
];
// let owned: Vec<String> = KINDS.iter().map(|s| s.to_string()).collect();

#[derive(FromRow, Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Custodian {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub custodian: String,
    pub account: Option<String>,
    pub kind: CustodianKind,
    pub color_code: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Type, serde::Serialize)]
#[sqlx(type_name = "VARCHAR")]
pub enum CustodianKind {
    Bank,
    Exchange,
    #[sqlx(rename = "Fintech Platform")]
    FintechPlatform,
    Pension,
    #[sqlx(rename = "Blockchain Wallet")]
    BlockchainWallet,
    Broker,
    #[sqlx(rename = "Payment Platform")]
    PaymentPlatform,
    Other,
}

impl CustodianKind {
    pub fn from_string(kind: &str) -> Result<Self, String> {
        match kind {
            "Bank" => Ok(CustodianKind::Bank),
            "Exchange" => Ok(CustodianKind::Exchange),
            "Fintech Platform" => Ok(CustodianKind::FintechPlatform), // Alias
            "Pension" => Ok(CustodianKind::Pension),
            "Blockchain Wallet" => Ok(CustodianKind::BlockchainWallet),
            "Payment Platform" => Ok(CustodianKind::PaymentPlatform),
            "Broker" => Ok(CustodianKind::Broker),
            _ => Err(format!("Invalid custodian kind: '{}'", kind)),
        }
    }
}

impl From<CustodianKind> for String {
    fn from(kind: CustodianKind) -> Self {
        match kind {
            CustodianKind::Bank => "Bank".to_string(),
            CustodianKind::Exchange => "Exchange".to_string(),
            CustodianKind::FintechPlatform => "Fintech Platform".to_string(),
            CustodianKind::Pension => "Pension".to_string(),
            CustodianKind::BlockchainWallet => "Blockchain Wallet".to_string(),
            CustodianKind::PaymentPlatform => "Payment Platform".to_string(),
            CustodianKind::Broker => "Broker".to_string(),
            CustodianKind::Other => "Other".to_string(),
        }
    }
}
