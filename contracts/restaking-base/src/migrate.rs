use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
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
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_contract: OldRestakingBaseContract = env::state_read().expect("failed");
        log!("Read old contract");

        Self {
            owner: old_contract.owner,
            uuid: old_contract.uuid,
            sequence: old_contract.sequence,
            stakers: old_contract.stakers,
            staking_pools: old_contract.staking_pools,
            consumer_chains: old_contract.consumer_chains,
            cc_register_fee: old_contract.cc_register_fee,
            staking_pool_whitelist_account: old_contract.staking_pool_whitelist_account,
            slash_guarantee: old_contract.slash_guarantee,
            slashes: old_contract.slashes,
            accounts: old_contract.accounts,
            is_contract_running: true,
        }
    }
}
