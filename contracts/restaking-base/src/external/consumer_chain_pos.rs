use crate::*;
use near_sdk::AccountId;

use crate::types::Key;

#[ext_contract(ext_consumer_chain_pos)]
pub trait ConsumerChainPos {
    fn bond(staker_id: AccountId, key: Key) -> PromiseOrValue<bool>;
    fn change_key(staker_id: AccountId, key: Key) -> Promise;
}
