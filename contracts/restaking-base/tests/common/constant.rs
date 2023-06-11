use super::*;

pub const TEST_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../../../../res/test_token.wasm");
pub const STAKING_POOL_WASM_BYTES: &[u8] = include_bytes!("../../../../res/staking_pool.wasm");
pub const STAKING_POOL_WHITELIST_WASM_BYTES: &[u8] =
    include_bytes!("../../../../res/whitelist.wasm");
pub const MOCK_CONSUMER_CHAIN_POS_WASM_BYTES: &[u8] =
    include_bytes!("../../../../res/mock_consumer_chain_pos.wasm");
pub const RESTAKING_BASE_WASM_BYTES: &[u8] =
    include_bytes!("../../../../res/restaking_base_contract.wasm");

pub const CC_REGISTER_FEE: Balance = parse_near!("10 near");
pub const SLASH_GUARANTEE: Balance = parse_near!("10 near");
