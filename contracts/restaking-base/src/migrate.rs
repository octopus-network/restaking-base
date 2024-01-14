use std::collections::HashMap;

use near_sdk::{env::log, EpochHeight, Timestamp};

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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
struct RestakingBaseContractForUnstakeBatch {
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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
struct RestakingBaseContractForPendingWithdrawal {
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
    pub accounts: LookupMap<AccountId, OldAccount>,
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
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
    /// Record staking pool unlock epoch
    pub unlock_epoch: EpochHeight,
}

impl From<OldStakingPool> for StakingPool {
    fn from(value: OldStakingPool) -> Self {
        Self {
            pool_id: value.pool_id.clone(),
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            stakers: value.stakers,
            locked: value.locked,
            unlock_epoch: value.unlock_epoch,
            last_unstake_epoch: 0,
            last_unstake_batch_id: None,
            current_unstake_batch_id: 0.into(),
            batched_unstake_amount: 0,
            submitted_unstake_batches: UnorderedMap::new(StorageKey::SubmittedUnstakeBatches {
                pool_id: value.pool_id,
            }),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OldAccount {
    // todo staking pool can't get all shares keeper
    pub legacy_shares: HashMap<PoolId, ShareBalance>,

    // todo need more suitable datastruct
    pub pending_withdrawals: UnorderedMap<WithdrawalCertificate, OldPendingWithdrawal>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct OldPendingWithdrawal {
    pub withdrawal_certificate: WithdrawalCertificate,
    pub pool_id: PoolId,
    pub amount: Balance,
    pub unlock_epoch: EpochHeight,
    pub unlock_time: Timestamp,
    pub beneficiary: AccountId,
    pub allow_other_withdraw: bool,
}

impl From<OldPendingWithdrawal> for PendingWithdrawal {
    fn from(value: OldPendingWithdrawal) -> Self {
        Self {
            withdrawal_certificate: value.withdrawal_certificate,
            pool_id: value.pool_id,
            amount: value.amount,
            unlock_epoch: value.unlock_epoch,
            unlock_time: value.unlock_time,
            beneficiary: value.beneficiary,
            allow_other_withdraw: value.allow_other_withdraw,
            unstake_batch_id: None,
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

    #[private]
    #[init(ignore_state)]
    pub fn migrate_unstake_batch() -> Self {
        let mut old_contract: RestakingBaseContractForUnstakeBatch =
            env::state_read().expect("failed");
        let staking_pools = old_contract.staking_pools.values().collect_vec();
        let mut new_staking_pools: UnorderedMap<PoolId, StakingPool> =
            UnorderedMap::new(StorageKey::StakingPools);
        old_contract.staking_pools.clear();
        for e in staking_pools {
            new_staking_pools.insert(&e.pool_id.clone(), &e.into());
        }

        Self {
            owner: old_contract.owner,
            uuid: old_contract.uuid,
            sequence: old_contract.sequence,
            stakers: old_contract.stakers,
            staking_pools: new_staking_pools,
            consumer_chains: old_contract.consumer_chains,
            cc_register_fee: old_contract.cc_register_fee,
            staking_pool_whitelist_account: old_contract.staking_pool_whitelist_account,
            slash_guarantee: old_contract.slash_guarantee,
            slashes: old_contract.slashes,
            accounts: old_contract.accounts,
            is_contract_running: old_contract.is_contract_running,
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate_pending_withdrawals(accounts: Vec<AccountId>) -> Self {
        let mut old_contract: RestakingBaseContractForPendingWithdrawal =
            env::state_read().expect("failed");
        let mut new_accounts: HashMap<AccountId, Account> = HashMap::new();
        for account_id in accounts {
            if !old_contract.accounts.contains_key(&account_id) {
                continue;
            }
            let mut old_account = old_contract.accounts.get(&account_id).unwrap();

            let old_pending_withdrawals = old_account.pending_withdrawals.values().collect_vec();

            old_account.pending_withdrawals.clear();

            let mut new_account = Account {
                legacy_shares: old_account.legacy_shares,
                pending_withdrawals: UnorderedMap::<WithdrawalCertificate, PendingWithdrawal>::new(
                    StorageKey::PendingWithdrawals {
                        account_id: account_id.clone(),
                    },
                ),
            };

            for pending_withdrawal in old_pending_withdrawals {
                new_account.pending_withdrawals.insert(
                    &pending_withdrawal.withdrawal_certificate,
                    &pending_withdrawal.clone().into(),
                );
            }
            old_contract.accounts.remove(&account_id);
            new_accounts.insert(account_id, new_account);
        }

        let mut new_accounts_map = LookupMap::new(StorageKey::Accounts);
        for (account_id, account) in new_accounts {
            new_accounts_map.insert(&account_id, &account);
        }

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
            accounts: new_accounts_map,
            is_contract_running: old_contract.is_contract_running,
        }
    }
}
