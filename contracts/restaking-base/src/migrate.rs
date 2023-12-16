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
    pub consumer_chains: UnorderedMap<ConsumerChainId, OldConsumerChain>,
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
pub struct OldConsumerChain {
    pub consumer_chain_id: ConsumerChainId,
    /// Staker need to wait some period for unbonding consumer chain
    pub unbonding_period: DurationOfSeconds,
    /// The url of consumer chain's website
    pub website: String,
    /// The account id of governance
    pub governance: AccountId,
    /// The set of bonding stakers' ids
    pub bond_stakers: UnorderedSet<StakerId>,
    /// The account id of treasury, the slash token will send to this account
    pub treasury: AccountId,
    pub status: ConsumerChainStatus,
    pub pos_account_id: AccountId,
    pub blacklist: UnorderedSet<AccountId>,
    pub register_fee: Balance,
}

impl From<OldConsumerChain> for ConsumerChain {
    fn from(value: OldConsumerChain) -> Self {
        Self {
            consumer_chain_id: value.consumer_chain_id,
            unbonding_period: value.unbonding_period,
            website: value.website,
            governance: value.governance,
            bonding_stakers: value.bond_stakers,
            treasury: value.treasury,
            status: value.status,
            pos_account_id: value.pos_account_id,
            blacklist: value.blacklist,
            register_fee: value.register_fee,
        }
    }
}

#[near_bindgen]
impl RestakingBaseContract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let mut old_contract: OldRestakingBaseContract = env::state_read().expect("failed");
        log!("Read old contract");
        let mut buffer_consumer_chains: Vec<ConsumerChain> = vec![];
        let old_consumer_chains = old_contract.consumer_chains.iter().collect_vec();

        for (chain_id, consumer_chain) in old_consumer_chains {
            old_contract.consumer_chains.remove(&chain_id);
            buffer_consumer_chains.push(consumer_chain.into())
        }

        let mut consumer_chains: UnorderedMap<String, ConsumerChain> =
            UnorderedMap::new(StorageKey::ConsumerChains);
        for consumer_chain in buffer_consumer_chains {
            consumer_chains.insert(&consumer_chain.consumer_chain_id, &consumer_chain);
        }

        Self {
            owner: old_contract.owner,
            uuid: old_contract.uuid,
            sequence: old_contract.sequence,
            stakers: old_contract.stakers,
            staking_pools: old_contract.staking_pools,
            consumer_chains: consumer_chains,
            cc_register_fee: old_contract.cc_register_fee,
            staking_pool_whitelist_account: old_contract.staking_pool_whitelist_account,
            slash_guarantee: old_contract.slash_guarantee,
            slashes: old_contract.slashes,
            accounts: old_contract.accounts,
            is_contract_running: true,
        }
    }
}
