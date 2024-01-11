use near_sdk::Timestamp;

use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldRestakingBaseContract {
    /// The owner of contract
    pub owner: AccountId,
    /// Universally Unique Identifier for some entity
    pub uuid: u64,
    /// Any staking change action will make sequence increase
    pub sequence: u64,
    /// The map from account id to staker struct
    pub stakers: LookupMap<AccountId, OldStaker>,
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
    pub is_contract_running: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OldStaker {
    pub staker_id: StakerId,
    /// The staking pool which staker is select to stake
    pub select_staking_pool: Option<PoolId>,
    /// The share of staker owned in staking pool
    pub shares: ShareBalance,
    /// The map from consumer chain id to unbonding period
    pub bonding_consumer_chains: UnorderedMap<ConsumerChainId, DurationOfSeconds>,
    /// The max period of bonding unlock
    pub max_bonding_unlock_period: DurationOfSeconds,
    /// If execute unbond it'll record unlock time
    pub unbonding_unlock_time: Timestamp,
}

impl From<OldStaker> for Staker {
    fn from(value: OldStaker) -> Self {
        Self {
            staker_id: value.staker_id.clone(),
            select_staking_pool: value.select_staking_pool,
            shares: value.shares,
            bonding_consumer_chains: value.bonding_consumer_chains,
            max_bonding_unlock_period: value.max_bonding_unlock_period,
            unbonding_unlock_time: value.unbonding_unlock_time,
            unbonding_consumer_chains: UnorderedMap::new(
                StorageKey::StakerUnbondingConsumerChains {
                    staker_id: value.staker_id,
                },
            ),
        }
    }
}

#[near_bindgen]
impl RestakingBaseContract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate(staker_list: Vec<AccountId>) -> Self {
        let mut old_contract: OldRestakingBaseContract = env::state_read().expect("failed");

        let mut new_stakers: LookupMap<AccountId, Staker> = LookupMap::new(StorageKey::Stakers);

        for staker_id in staker_list {
            if let Some(old_staker) = old_contract.stakers.remove(&staker_id) {
                new_stakers.insert(&staker_id, &old_staker.into());
            }
        }

        Self {
            owner: old_contract.owner,
            uuid: old_contract.uuid,
            sequence: old_contract.sequence,
            stakers: new_stakers,
            staking_pools: old_contract.staking_pools,
            consumer_chains: old_contract.consumer_chains,
            cc_register_fee: old_contract.cc_register_fee,
            staking_pool_whitelist_account: old_contract.staking_pool_whitelist_account,
            slash_guarantee: old_contract.slash_guarantee,
            slashes: old_contract.slashes,
            accounts: old_contract.accounts,
            is_contract_running: old_contract.is_contract_running,
        }
    }
}
