pub mod constants;
pub mod contract_interface;
pub mod events;
pub mod external;
pub mod migrate;
pub mod models;
pub mod types;
pub mod utils;

use crate::constants::gas_constants::*;
use crate::events::*;
use crate::external::consumer_chain_pos::ext_consumer_chain_pos;
use crate::models::consumer_chain::ConsumerChain;
use crate::models::consumer_chain::*;
use crate::models::staker::Staker;
use crate::models::staker::*;
use crate::models::staking_pool::*;
use crate::utils::*;
use crate::{
    constants::NUM_EPOCHS_TO_UNLOCK,
    contract_interface::staking::{StakerAction, StakingCallback},
    contract_interface::view::*,
    external::staking_pool_whitelist::ext_whitelist,
    types::ShareBalance,
};
use crate::{contract_interface::restaking::*, external::staking_pool::ext_staking_pool};
use itertools::Itertools;
use models::account::Account;
use models::pending_withdrawal::PendingWithdrawal;
use models::slash::Slash;
use models::staker::StakingChangeResult;
use models::staking_pool::StakingPool;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Gas;
use near_sdk::PromiseResult;
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise,
};
use near_sdk::{log, PromiseOrValue};
use near_sdk::{serde_json::json, ONE_YOCTO};
use std::cmp::{max, min};
use std::ops::Mul;
use types::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct RestakingBaseContract {
    /// The owner of contract
    pub owner: AccountId,
    /// Universally Unique Identifier for some entity
    pub uuid: u64,
    /// Any staking change action will make sequence increase
    pub sequence: u64,
    /// The map from account id to staker struct
    pub stakers: LookupMap<AccountId, Staker>,
    /// The map from pool account id to staking pool struct
    pub staking_pools: UnorderedMap<PoolId, StakingPool>,
    /// The map from consumer chain id to consumer chain struct
    pub consumer_chains: UnorderedMap<ConsumerChainId, ConsumerChain>,
    /// The fee of register consumer chain
    pub cc_register_fee: Balance,
    /// The staking pool whitelist account
    pub staking_pool_whitelist_account: AccountId,
    /// The guarantee of slash
    pub slash_guarantee: Balance,
    /// The map from slash id to slash struct
    pub slashes: LookupMap<SlashId, Slash>,
    /// The map from account id to account struct
    pub accounts: LookupMap<AccountId, Account>,
}

#[near_bindgen]
impl RestakingBaseContract {
    #[init]
    pub fn new(
        owner: AccountId,
        cc_register_fee: U128,
        staking_pool_whitelist_account: AccountId,
        slash_guarantee: U128,
    ) -> Self {
        Self {
            owner,
            uuid: 0,
            sequence: 0,
            stakers: LookupMap::new(StorageKey::Stakers),
            staking_pools: UnorderedMap::new(StorageKey::StakingPools),
            consumer_chains: UnorderedMap::new(StorageKey::ConsumerChains),
            cc_register_fee: cc_register_fee.0,
            staking_pool_whitelist_account,
            slash_guarantee: slash_guarantee.0,
            slashes: LookupMap::new(StorageKey::Slashes),
            accounts: LookupMap::new(StorageKey::Accounts),
        }
    }

    pub(crate) fn transfer_near(&self, account_id: AccountId, amount: Balance) {
        assert!(amount > 0, "Failed to send near because the amount is 0.");
        log!("transfer {} to {}", amount, account_id);
        Promise::new(account_id).transfer(amount);
    }

    pub(crate) fn next_uuid(&mut self) -> u64 {
        self.uuid += 1;
        self.uuid
    }

    pub(crate) fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Stakers,
    StakerBondingConsumerChains { staker_id: StakerId },
    ConsumerChainBondingStakers { consumer_chain_id: ConsumerChainId },
    StakingPools,
    StakingPoolStakers { pool_id: PoolId },
    ConsumerChains,
    ConsumerChainBlackList { consumer_chain_id: ConsumerChainId },
    Slashes,
    Accounts,
    PendingWithdrawals { account_id: AccountId },
}
