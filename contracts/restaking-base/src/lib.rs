pub mod constants;
pub mod contract_interface;
pub mod external;
pub mod models;
pub mod types;
pub mod utils;

use crate::models::account::*;
use crate::models::consumer_chain::ConsumerChain;
use crate::models::consumer_chain::*;
use crate::models::pending_withdrawal::*;
use crate::models::slash::*;
use crate::models::staker::Staker;
use crate::models::staker::*;
use crate::models::staking_pool::*;
use crate::utils::*;
use models::account::Account;
use models::pending_withdrawal::PendingWithdrawal;
use models::slash::Slash;
use models::staker::StakingChangeResult;
use models::staking_pool::StakingPool;
use models::storage_manager::StorageManager;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, StorageUsage,
};
use near_sdk::{log, PromiseOrValue};
use types::{ConsumerChainId, PoolId, SlashId, StakerId, WithdrawalCertificatetId};

use crate::constants::gas_constants::*;
use crate::external::consumer_chain_pos::ext_consumer_chain_pos;
use crate::{
    constants::NUM_EPOCHS_TO_UNLOCK,
    contract_interface::staking::{StakeView, StakerAction, StakingCallBack},
    external::staking_pool_whitelist::ext_whitelist,
    types::ShareBalance,
};
use crate::{
    contract_interface::restaking::*, external::staking_pool::ext_staking_pool, types::ValidaotrSet,
};
use itertools::Itertools;
use near_sdk::Gas;
use near_sdk::{serde_json::json, ONE_YOCTO};
use near_sdk::{PromiseResult, Timestamp};
use std::cmp::{max, min};
use std::ops::Mul;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct RestakingBaseContract {
    pub owner: AccountId,
    pub uuid: u64,
    pub sequence: u64,
    pub stakers: LookupMap<AccountId, Staker>,
    // todo 如果一个pool从白名单移除， 需要处理
    pub staking_pools: UnorderedMap<PoolId, StakingPool>,
    pub consumer_chains: UnorderedMap<ConsumerChainId, ConsumerChain>,
    pub cc_register_fee: Balance,
    pub staking_pool_whitelist_account: AccountId,
    pub slash_guarantee: Balance,
    pub slashes: LookupMap<SlashId, Slash>,
    pub accounts: LookupMap<AccountId, Account>,
    pub storage_managers: LookupMap<AccountId, StorageManager>,
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
            storage_managers: LookupMap::new(StorageKey::StorageManagers),
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
    StorageManagers,
}
