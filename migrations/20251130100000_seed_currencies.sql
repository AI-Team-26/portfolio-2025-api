-- Seed initial currencies for the application
-- These are the currencies required by the application to function properly
-- precision = native smallest unit decimals (ISO 4217 minor units for fiat)

INSERT INTO Currency (symbol, name, kind, is_active, precision, coingecko_id, is_major) VALUES
-- Fiat currencies
('USD', 'US Dollar', 'Fiat', true, 2, NULL, true),
('EUR', 'Euro', 'Fiat', true, 2, NULL, true),
('GBP', 'British Pound', 'Fiat', true, 2, NULL, true),
('JPY', 'Japanese Yen', 'Fiat', true, 0, NULL, true),
('AUD', 'Australian Dollar', 'Fiat', true, 2, NULL, true),
('CHF', 'Swiss Franc', 'Fiat', true, 2, NULL, false),
('CAD', 'Canadian Dollar', 'Fiat', true, 2, NULL, false),
('CNY', 'Chinese Yuan', 'Fiat', true, 2, NULL, false),

-- Crypto currencies (native decimals: satoshi 1e-8, wei 1e-18, lamport 1e-9,
-- lovelace 1e-6, Planck 1e-10, drop 1e-6, LINK 1e-18)
('BTC', 'Bitcoin', 'Crypto', true, 8, 'bitcoin', true),
('ETH', 'Ethereum', 'Crypto', true, 18, 'ethereum', true),
('XRP', 'XRP', 'Crypto', true, 6, 'ripple', true),
('XRD', 'Radix', 'Crypto', true, 18, 'radix', false),
('SOL', 'Solana', 'Crypto', true, 9, 'solana', false),
('ADA', 'Cardano', 'Crypto', true, 6, 'cardano', false),
('DOT', 'Polkadot', 'Crypto', true, 10, 'polkadot', false),
('LINK', 'Chainlink', 'Crypto', true, 18, 'chainlink', false),

-- Stablecoins
('USDT', 'Tether', 'Stablecoin', true, 6, 'tether', false),
('USDC', 'USD Coin', 'Stablecoin', true, 6, 'usd-coin', false),
('DAI', 'Dai', 'Stablecoin', true, 18, 'dai', false)
ON CONFLICT (symbol) DO NOTHING;
