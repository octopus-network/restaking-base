mod contract_interface;
mod models;
mod types;
mod utils;
mod external;
mod constants;

use models::account::Account;
use models::pending_withdrawal::PendingWithdrawal;
use models::slash::Slash;
use models::staking_pool::StakingPool;
use models::storage_manager::StorageManager;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    ext_contract,
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    StorageUsage, assert_one_yocto,
};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use crate::models::consumer_chain::ConsumerChain;
use crate::models::staker::Staker;
use crate::utils::*;
use near_sdk::PromiseOrValue;
use types::{PoolId, StakerId, ConsumerChainId, SlashId, WithdrawalReceiptId};
use near_sdk::json_types::U64;
use near_sdk::json_types::U128;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Stakers,
    StakerBondingConsumerChains {
        staker_id: StakerId
    },
    ConsumerChainBondingStakers {
        consumer_chain_id: ConsumerChainId
    },
    StakingPools,
    ConsumerChains,
    ConsumerChainBlackList {
        consumer_chain_id: ConsumerChainId
    },
    Slashes,
    Accounts,
    PendingWithdrawals {account_id: AccountId},
    StorageManagers
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct RestakingBaseContract {
    pub owner: AccountId,
    pub uuid: u64,
    pub stakers: LookupMap<AccountId, Staker>,
    // todo 如果一个pool从白名单移除， 需要处理
    pub staking_pools: LookupMap<PoolId, StakingPool>,
    pub consumer_chains: UnorderedMap<ConsumerChainId, ConsumerChain>,
    pub cc_register_fee: Balance,
    pub staking_pool_whitelist_account: AccountId,
    pub slash_guarantee: Balance,
    pub slashes: LookupMap<SlashId, Slash>,
    pub accounts: LookupMap<AccountId, Account>,
    pub storage_managers: LookupMap<AccountId, StorageManager>
}

#[near_bindgen]
impl RestakingBaseContract {
    #[init]
    pub fn new(owner: AccountId, cc_register_fee: Balance, staking_pool_whitelist_account: AccountId, slash_guarantee: Balance) -> Self {
        Self {
            owner,
            uuid: 0,
            stakers: LookupMap::new(StorageKey::Stakers),
            staking_pools: LookupMap::new(StorageKey::StakingPools),
            consumer_chains: UnorderedMap::new(StorageKey::ConsumerChains),
            cc_register_fee,
            staking_pool_whitelist_account,
            slash_guarantee,
            slashes: LookupMap::new(StorageKey::Slashes),
            accounts: LookupMap::new(StorageKey::Accounts),
            storage_managers: LookupMap::new(StorageKey::StorageManagers),
        }
    }

    pub(crate) fn transfer_near(&self, account_id: AccountId , amount: Balance) {
        assert!(amount > 0, "Failed to send near because the amount is 0.");
        Promise::new(account_id).transfer(amount);
    }

    pub(crate) fn next_uuid(&mut self) -> u64 {
        self.uuid+=1;
        self.uuid
    }

}