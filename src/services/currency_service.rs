use crate::entities::currency::CurrencyKind;
use dashmap::DashMap;
use std::sync::Arc; // Atomic Reference Counter
                    //e std::sync::{LazyLock, RwLock}; // Rust doesn't allow "static mut" :-(
use crate::endpoints::models::currency_models::CurrencyOfUser;
use crate::repositories::currency_of_user_repository::CurrencyOfUserRepository;
use crate::{entities::currency::Currency, repositories::currency_repository::CurrencyRepository};

// Using Arc<DashMap<...>> for thread-safe, lock-free shared state.
// DashMap provides internal synchronization, allowing concurrent reads and writes
// without requiring &mut self or manual Mutex/RwLock handling.
// This lets async methods safely update caches from multiple tasks or threads

#[derive(Clone)]
pub struct CurrencyService {
    repository: CurrencyRepository,
    repository_of_user: CurrencyOfUserRepository,
    // Thread-safe cache
    pub currencies: Arc<DashMap<i32, Currency>>,
    pub crypto_and_stable_currencies: Arc<DashMap<i32, Currency>>,
    pub fiat_currencies: Arc<DashMap<i32, Currency>>,
    map_by_symbol: Arc<DashMap<String, Currency>>,
    map_by_coingecko: Arc<DashMap<String, Currency>>,
}

/*
static CURRENCIES: LazyLock<RwLock<HashMap<i32, Currency>>> =
    LazyLock::new(|| RwLock::new(HashMap::new())); */

impl CurrencyService {
    /*ub async fn new(repository: CurrencyRepository) -> Self {
        let instance = Self { repository };

        // Load the cache
        instance.load_cache().await;

        instance
    }*/

    pub fn new(
        repository: CurrencyRepository,
        repository_of_user: CurrencyOfUserRepository,
    ) -> Self {
        Self {
            repository,
            repository_of_user,
            currencies: Arc::new(DashMap::new()), // Initialize empty
            crypto_and_stable_currencies: Arc::new(DashMap::new()), // Initialize empty
            fiat_currencies: Arc::new(DashMap::new()), // Initialize empty
            map_by_symbol: Arc::new(DashMap::new()), // Initialize empty
            map_by_coingecko: Arc::new(DashMap::new()), // Initialize empty
        }
    }

    // Initialize cache at startup (called only once)
    pub async fn init_cache(&self) -> Result<(), String> {
        let items: Vec<Currency> = self.repository.list().await?;

        self.currencies.clear();
        for currency in items {
            self.currencies.insert(currency.id, currency.clone());
            match &currency.kind {
                CurrencyKind::Fiat => self.fiat_currencies.insert(currency.id, currency.clone()),
                CurrencyKind::Crypto | CurrencyKind::Stablecoin => self
                    .crypto_and_stable_currencies
                    .insert(currency.id, currency.clone()),
            };

            // symbol map
            self.map_by_symbol
                .insert(currency.symbol.clone().to_uppercase(), currency.clone()); // Uppercase
            if let Some(id) = &currency.coingecko_id {
                self.map_by_coingecko.insert(id.clone(), currency.clone()); // Uppercase
            }
        }
        Ok(())
    }

    pub fn get(&self, id: i32) -> Currency {
        match self.currencies.get(&id) {
            Some(c) => c.value().clone(),
            None => panic!("Currency not found with id={}.", id),
        }
    }

    pub fn try_get(&self, id: i32) -> Option<Currency> {
        self.currencies.get(&id).map(|entry| entry.clone())
    }

    #[allow(non_snake_case)]
    pub fn try_get_by_symbol_CI(&self, symbol: &str) -> Option<Currency> {
        self.map_by_symbol
            .get(&symbol.to_uppercase())
            .map(|entry| entry.clone())
    }

    #[allow(non_snake_case)]
    pub fn try_get_by_Coingecko_id(&self, id: &str) -> Option<Currency> {
        self.map_by_coingecko.get(id).map(|entry| entry.clone())
    }

    pub fn all(&self) -> Vec<Currency> {
        self.currencies
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn _all_crypto_and_stable(&self) -> Vec<Currency> {
        self.currencies
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn _all_fiat(&self) -> Vec<Currency> {
        self.currencies
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // Database operations with cache updates
    pub async fn create(&self, mut item: Currency) -> Result<i32, String> {
        let id = self.repository.create(item.clone()).await?;

        // update the item.id
        item.id = id;

        // Update cache - thread-safe, no locks!
        self.currencies.insert(id, item);

        Ok(id)
    }

    pub async fn update(&self, item: Currency) -> Result<(), String> {
        self.repository.update(&item).await?;

        // Update the cache
        self.currencies.insert(item.id, item);

        Ok(())
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        self.repository.delete(id).await?;

        // Update the cache
        self.currencies.remove(&id);

        Ok(())
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<CurrencyOfUser>, String> {
        let used_by_user: Vec<i32> = self
            .repository_of_user
            .list(user_id)
            .await?
            .into_iter()
            .map(|record| record.currency_id)
            .collect();

        let items = self
            .currencies
            .iter()
            .filter(|currency| currency.is_active)
            .map(|currency| CurrencyOfUser {
                id: currency.id,
                symbol: currency.symbol.clone(),
                name: currency.name.clone(),
                kind: currency.kind.clone().into(),
                is_major: currency.is_major,
                is_used: used_by_user.contains(&currency.id),
            })
            .collect();

        Ok(items)
    }

    pub async fn enable_currency_for_user(
        &self,
        user_id: String,
        currency_id: i32,
        enable: bool,
    ) -> Result<(), String> {
        match enable {
            true => self.repository_of_user.create(user_id, currency_id).await,
            false => self.repository_of_user.delete(user_id, currency_id).await,
        }
    }
}
