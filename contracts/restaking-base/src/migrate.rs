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
    pub stakers: LookupMap<AccountId, Staker>,
    /// The map from pool account id to staking pool struct
    pub staking_pools: UnorderedMap<PoolId, OldStakingPool>,
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
pub struct OldStakingPool {
    pub pool_id: AccountId,
    /// Total minted share balance in this staking pool
    pub total_share_balance: ShareBalance,
    /// Total staked near balance in this staking pool
    pub total_staked_balance: Balance,
    /// The set of all stakers' ids
    pub stakers: UnorderedSet<AccountId>,
    /// When restaking base contract interactive with staking pool contract, it'll lock this staking pool until all cross contract call finished
    pub locked: bool,
}

impl From<OldStakingPool> for StakingPool {
    fn from(value: OldStakingPool) -> Self {
        Self {
            pool_id: value.pool_id,
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            stakers: value.stakers,
            locked: value.locked,
            unlock_epoch: 0,
        }
    }
}

#[near_bindgen]
impl RestakingBaseContract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let mut old_contract: OldRestakingBaseContract = env::state_read().expect("failed");
        let new_pool: Vec<StakingPool> = old_contract
            .staking_pools
            .values()
            .into_iter()
            .map(Into::into)
            .collect_vec();

        for pool in new_pool.iter() {
            old_contract.staking_pools.remove(&pool.pool_id);
        }

        let mut new_staking_pool_map: UnorderedMap<PoolId, StakingPool> =
            UnorderedMap::new(StorageKey::StakingPools);

        for pool in new_pool.iter() {
            new_staking_pool_map.insert(&pool.pool_id, pool);
        }

        Self {
            owner: old_contract.owner,
            uuid: old_contract.uuid,
            sequence: old_contract.sequence,
            stakers: old_contract.stakers,
            staking_pools: new_staking_pool_map,
            consumer_chains: old_contract.consumer_chains,
            cc_register_fee: old_contract.cc_register_fee,
            staking_pool_whitelist_account: old_contract.staking_pool_whitelist_account,
            slash_guarantee: old_contract.slash_guarantee,
            slashes: old_contract.slashes,
            accounts: old_contract.accounts,
            is_contract_running: old_contract.is_contract_running,
        }

        // for (pool_id, pool) in old_contract.s

        // let old_staking_pools: UnorderedMap<PoolId, OldStakingPool> = self.staking_pools
        //     UnorderedMap::new(StorageKey::StakingPools);
        // for (pool_id, pool) in old_staking_pools.iter() {
        //     let new_pool = pool.into();
        //     log!("Pool_id: {}", pool_id);
        //     self.internal_save_staking_pool(&new_pool);
        // }
    }
}
