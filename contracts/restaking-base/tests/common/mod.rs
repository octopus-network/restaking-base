pub mod constant;
pub mod initialization;
pub mod util;

pub use crate::contracts::staking_pool::StakingPoolContract;
pub use crate::contracts::{
    restaking_base::RestakingBaseContract, staking_pool::RewardFeeFraction,
};
pub use constant::*;
pub use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds};
pub use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
pub use near_sdk::collections::UnorderedMap;
pub use near_sdk::json_types::{Base58PublicKey, U128};
pub use near_sdk::serde::{Deserialize, Serialize};
pub use near_sdk::serde_json::json;
pub use near_sdk::ONE_YOCTO;
pub use near_sdk::{
    env, ext_contract, near_bindgen, Balance, EpochHeight, Promise, PromiseResult, PublicKey,
};
pub use near_units::parse_gas;
pub use near_units::parse_near;
pub use std::str::FromStr;
pub use util::*;
pub use workspaces::network::Sandbox;
pub use workspaces::result::ExecutionFinalResult;
pub use workspaces::Account;
pub use workspaces::AccountId;
pub use workspaces::Worker;

pub use crate::common::initialization::*;
pub use restaking_base_contract::models::consumer_chain::*;
pub use restaking_base_contract::types::*;

pub trait NearContract {
    fn get_deploy_account(&self) -> &Account;
}
