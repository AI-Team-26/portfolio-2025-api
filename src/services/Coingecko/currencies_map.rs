use once_cell::sync::Lazy;
use std::collections::HashMap;

// map Coingecko ids for crypto/stable, to be used as coin "id" parameter in the api
// The full list can be obtained from GET /coins/list
// <currency symbol>:<Coingecko id_name>
pub static _COINGECKO_COIN_ID: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let records = vec![("BTC", "bitcoin"), ("ETH", "ethereum")];

    records.into_iter().collect()
});

// Map Coingecko ids to be used as "vs_currencies" parameter in the api
// The full list can be obtained from GET /simple/supported_vs_currencies
// <currency symbol>:<Coingecko id_symbol>
pub static COINGECKO_QUOTE_IDS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let records = vec![
        ("EUR", "eur"),
        //("USD", "usd"),
        ("GBP", "gbp"),
        //("BTC", "btc"),  // not "bitcoin" !!
        //("ETH", "eth"),  // not "ethereum" !!
    ];

    records.into_iter().collect()
});
