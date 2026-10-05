pub mod auth {
    use std::time::Duration;

    pub const ACCESS_TOKEN_LIFETIME: Duration = Duration::from_secs(30 * 60); // 30 minutes
    pub const REFRESH_TOKEN_LIFETIME: Duration = Duration::from_secs(30 * 24 * 60 * 60); // 30 days

    pub mod error_codes {
        pub const MISSING_AUTH_HTTP_HEADER: &str = "MISSING_AUTH_HTTP_HEADER";
        pub const INVALID_OR_EXPIRED_TOKEN: &str = "INVALID_OR_EXPIRED_TOKEN";
    }
}

pub mod external_services {
    pub const COINGECKO: &str = "CoinGecko";
    //pub const COINMARKETCAP:&str = "CoinMarketCap";
}
