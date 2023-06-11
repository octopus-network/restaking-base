use crate::*;
use near_sdk::AccountId;

/// External interface for the staking pool whitelist contract.
#[ext_contract(ext_whitelist)]
pub trait ExtWhitelist {
    fn is_whitelisted(&mut self, staking_pool_account_id: AccountId) -> bool;
}
