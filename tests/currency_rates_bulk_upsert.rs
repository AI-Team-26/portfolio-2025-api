// Integration tests for the bulk currency-rate upsert path.
// Uses testcontainers to spin up a real Postgres instance.
// Run with: cargo test --test currency_rates_bulk_upsert
// Prerequisite: Docker must be running so testcontainers can start the container.

use portfolio_api::repositories::currency_rate_repository::CurrencyRateRepository;
use portfolio_api::repositories::schemas::currency_rate_record::CurrencyRateRecord;
use portfolio_api::utils::datetime::{now, today};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Instant;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// Spawn a Postgres container and return a ready connection pool.
async fn postgres_pool() -> PgPool {
    let pg_image = Postgres::default()
        .with_user("portfolio_user")
        .with_password("portfolio_password")
        .with_db_name("portfolio");

    let container = pg_image
        .start()
        .await
        .expect("Failed to start Postgres container");
    let port = container.get_host_port_ipv4(5432).await.expect("port");
    let host = container.get_host().await.expect("host");

    let url = format!("postgres://portfolio_user:portfolio_password@{host}:{port}/portfolio");

    let pool = PgPool::connect(&url)
        .await
        .expect("Failed to connect to Postgres container");

    // Apply migrations (incl. currency seed) so the schema is ready
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn record(base_currency_id: i32, quote_currency_id: i32, rate: &str) -> CurrencyRateRecord {
    CurrencyRateRecord {
        base_currency_id,
        quote_currency_id,
        date: today(),
        source: "coingecko".to_string(),
        rate: Decimal::from_str(rate).unwrap(),
        created_at: now(),
    }
}

async fn currency_id(pool: &PgPool, symbol: &str) -> i32 {
    let row: (i32,) = sqlx::query_as("SELECT id FROM Currency WHERE symbol = $1")
        .bind(symbol)
        .fetch_one(pool)
        .await
        .expect("seeded currency not found");
    row.0
}

#[tokio::test]
async fn create_many_is_idempotent_and_refreshes_existing_rows() {
    let pool = postgres_pool().await;
    let repo = CurrencyRateRepository::new(pool.clone());

    let btc = currency_id(&pool, "BTC").await;
    let eth = currency_id(&pool, "ETH").await;
    let eur = currency_id(&pool, "EUR").await;
    let gbp = currency_id(&pool, "GBP").await;

    // first job run of the day: inserts
    let first_run = vec![record(btc, eur, "42000.5"), record(eth, gbp, "2500.25")];
    assert_eq!(repo.create_many(&first_run).await.unwrap(), 2);

    // second run same day with fresher rates: must update, never duplicate
    let second_run = vec![record(btc, eur, "43000.75"), record(eth, gbp, "2600.0")];
    assert_eq!(repo.create_many(&second_run).await.unwrap(), 2);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM CurrencyRates WHERE date = $1")
        .bind(today())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 2, "rerun must not duplicate rows");

    // latest rate wins
    let rates = repo.search(btc, eur, Some(today())).await.unwrap();
    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].rate, Decimal::from_str("43000.75").unwrap());
}

#[tokio::test]
async fn create_many_empty_batch_is_noop() {
    let pool = postgres_pool().await;
    let repo = CurrencyRateRepository::new(pool);
    assert_eq!(repo.create_many(&[]).await.unwrap(), 0);
}

/// Evidence for the N+1 fix: times one-INSERT-per-row (old behavior) vs a single
/// multi-row upsert over the same dataset and logs both durations to CI output.
#[tokio::test]
async fn bulk_upsert_roundtrip_comparison() {
    let pool = postgres_pool().await;
    let repo = CurrencyRateRepository::new(pool.clone());

    let bases: Vec<i32> = sqlx::query_scalar::<_, i32>("SELECT id FROM Currency ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    let mut quotes = Vec::with_capacity(2);
    for symbol in ["EUR", "GBP"] {
        quotes.push(currency_id(&pool, symbol).await);
    }

    let batch: Vec<CurrencyRateRecord> = bases
        .iter()
        .flat_map(|b| quotes.iter().map(move |q| record(*b, *q, "1.5")))
        .collect();

    // old behavior: one round-trip per row (single-row batches)
    let t0 = Instant::now();
    for r in &batch {
        repo.create_many(std::slice::from_ref(r)).await.unwrap();
    }
    let per_row = t0.elapsed();

    // new behavior: single multi-row upsert (same keys -> updates)
    let t1 = Instant::now();
    let affected = repo.create_many(&batch).await.unwrap();
    let bulk = t1.elapsed();
    assert_eq!(affected, batch.len());

    eprintln!(
        "round-trip comparison over {} rows: per-row inserts {:?}, single bulk upsert {:?}",
        batch.len(),
        per_row,
        bulk
    );
    // one round-trip must beat N sequential ones; guards against regressing to per-row inserts
    assert!(
        bulk < per_row,
        "bulk upsert ({:?}) should be faster than per-row inserts ({:?})",
        bulk,
        per_row
    );
}
