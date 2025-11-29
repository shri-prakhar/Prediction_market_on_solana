pub const MANAGER_SEED: &[u8] = b"manager";
pub const MARKET_SEED: &[u8] = b"market";
pub const VAULT_YES_SEED: &[u8] = b"vault_yes";
pub const VAULT_NO_SEED: &[u8] = b"vault_no";
pub const VAULT_USDC_SEED: &[u8] = b"vault_usdc";
pub const ORDERBOOK_SEED: &[u8] = b"orderbook";
pub const BIDS_SEED: &[u8] = b"bids";
pub const ASKS_SEEDS: &[u8] = b"asks";
pub const EVENT_QUEUE_SEED: &[u8] = b"event_queue";
pub const REQUEST_QUEUE_SEED: &[u8] = b"request_queue";
pub const POSITION_SEED: &[u8] = b"position";
pub const OPEN_ORDER_SEED: &[u8] = b"open_order";
pub const FEE_VAULT_USDC: &[u8] = b"fee_vault_usdc";
// pub const MAX_ORDER_PER_TRADER: usize = 16;
// pub const MAX_SLAB_NODES: usize = 1024; //these are max orders stored on a single slab
// pub const MAX_EVENTS: usize = 128;
// pub const FEE_BASIS_POINTS: u16 = 50; //0.5%
// pub const CRANKER_REWARD_BPS: u16 = 200; //2% of the fees

// pub const EMPTY_INDEX: i32 = -1;
pub const PRICE_PRICISION_SCALE: u128 = 1_000_000u128; // this u128 is type casting this 1_000_000 to u128 type

pub const MAX_REQUESTS: usize = 512;
pub const MAX_EVENTS: usize = 1024;
pub const MAX_PRICE_NODES: usize = 1024;
pub const MAX_ORDER_ENTRIES: usize = 2048;
pub const MAX_OPEN_ORDER_SLOTS: usize = 32;
pub const MAX_MARKET_QUESTION: usize = 256;
pub const MAX_MARKET_DESC: usize = 1024;

pub const FEE_BPS: u16 = 30; //0.3%
pub const CRANKER_REWARD_BPS: u16 = 50; //0.5%
pub const PRICE_PRECISION_SCALE: u128 = 1_000_000;
