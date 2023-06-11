use near_sdk::Balance;

pub const NO_DEPOSIT: Balance = 0;

// pub const NO_TGAS: u64 =0;
pub const TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST: u64 = 30;
pub const TGAS_FOR_IS_WHITELISTED: u64 = 10;
pub const TGAS_FOR_PING: u64 = 10;
pub const TGAS_FOR_DEPOSIT_AND_STAKE: u64 = 10;
pub const TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE: u64 = 5;
pub const TGAS_FOR_CHANGE_KEY: u64 = 30;
pub const TGAS_FOR_UNSTAKE: u64 = 30;
pub const TGAS_FOR_WITHDRAW: u64 = 30;
