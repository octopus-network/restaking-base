use near_sdk::Balance;

pub const NO_DEPOSIT: Balance = 0;

// pub const NO_TGAS: u64 =0;
pub const TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST: u64 = 130;
pub const TGAS_FOR_IS_WHITELISTED: u64 = 5;
pub const TGAS_FOR_PING: u64 = 30;
pub const TGAS_FOR_DEPOSIT_AND_STAKE: u64 = 30;
pub const TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE: u64 = 5;
pub const TGAS_FOR_CHANGE_KEY: u64 = 30;
pub const TGAS_FOR_BOND: u64 = 50;
pub const TGAS_FOR_BOND_CALLBACK: u64 = 10;
pub const TGAS_FOR_UNSTAKE: u64 = 50;
pub const TGAS_FOR_UNSTAKE_BATCH_CALLBACK: u64 = 15;
pub const TGAS_FOR_INCREASE_STAKE_AFTER_PING: u64 = 10
    + TGAS_FOR_DEPOSIT_AND_STAKE
    + TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE
    + TGAS_FOR_INCREASE_STAKE_CALL_BACK;
pub const TGAS_FOR_INCREASE_STAKE_CALL_BACK: u64 = 15;
pub const TGAS_FOR_DECREASE_STAKE_AFTER_PING: u64 =
    10 + TGAS_FOR_DECREASE_STAKE_CALL_BACK + TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE;
pub const TGAS_FOR_DECREASE_STAKE_CALL_BACK: u64 = 10;

pub const TGAS_FOR_UNSTAKE_AFTER_PING: u64 =
    10 + TGAS_FOR_UNSTAKE_CALL_BACK + TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE;
pub const TGAS_FOR_UNSTAKE_CALL_BACK: u64 = 50;
pub const TGAS_FOR_WITHDRAW: u64 = 30;
pub const TGAS_FOR_WITHDRAW_UNSTAKE_BATCH_CALLBACK: u64 = 30;
pub const TGAS_FOR_SINGLE_WITHDRAW_CALLBACK: u64 = 10;
pub const TGAS_FOR_PING_CALLBACK: u64 = 10;
