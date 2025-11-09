pub const MANAGER_SEED : &[u8] = b"manager";
pub const MARKET_SEED :&[u8] = b"market";
pub const VAULT_SEED: &[u8] = b"vault";
pub const OUTCOME_POOL_SEED:&[u8] = b"outcome_pool";
pub const ORDERBOOK_SEED : &[u8] = b"orderbook";
pub const BIDS_SEED : &[u8] = b"bids";
pub const ASKS_SEEDS : &[u8] = b"asks";
pub const EVRNT_QUEUE_SEED : &[u8] = b"event_queue";
pub const POSITION_SEED: &[u8] = b"position";
pub const ORDER_SEED:&[u8] = b"order";

pub const MAX_ORDER_PER_TRADER : usize   = 16;
pub const MAX_SLAB_NODES :  usize = 1024; //these are max orders stored on a single slab    
pub const MAX_EVENTS: usize = 128;
pub const FEE_BASIS_POINTS: u16 = 50; //0.5%

pub const EMPTY_INDEX:i32 = -1;
