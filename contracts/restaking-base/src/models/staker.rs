use std::cmp::max;

use crate::types::{DurationOfSeconds, PoolId, Sequence, ShareBalance};
use crate::*;
use near_sdk::{collections::UnorderedSet, Timestamp};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Staker {
    pub staker_id: StakerId,
    pub select_staking_pool: Option<PoolId>,
    pub shares: ShareBalance,

    pub bonding_consumer_chains: UnorderedMap<ConsumerChainId, DurationOfSeconds>,
    pub max_bonding_unlock_period: Timestamp,
    pub unbonding_unlock_time: Timestamp,
    // pub withdrawal
}

impl Staker {
    pub fn new(staker_id: StakerId) -> Self {
        Staker {
            staker_id: staker_id.clone(),
            select_staking_pool: None,
            shares: 0,
            bonding_consumer_chains: UnorderedMap::new(StorageKey::StakerBondingConsumerChains {
                staker_id: staker_id.clone(),
            }),
            max_bonding_unlock_period: 0,
            unbonding_unlock_time: env::block_timestamp(),
        }
    }

    pub fn bond(&mut self, consumer_chain_id: &ConsumerChainId, unbond_period: DurationOfSeconds) {
        assert!(
            self.unbonding_unlock_time <= env::block_timestamp(),
            "Failed to bond by {}, the bonding unlock time({}) should not great then block time({}).",
            self.staker_id,
            self.unbonding_unlock_time,
            env::block_timestamp()
        );

        self.max_bonding_unlock_period = max(self.max_bonding_unlock_period, unbond_period);
        self.bonding_consumer_chains
            .insert(consumer_chain_id, &unbond_period);
    }

    pub fn unbond(&mut self, consumer_chain_id: &ConsumerChainId) {
        let unbond_period = self
            .bonding_consumer_chains
            .remove(&consumer_chain_id)
            .expect(
                format!(
                    "{} not found in staker bonding_consumer_chains.",
                    consumer_chain_id
                )
                .as_str(),
            );

        self.unbonding_unlock_time = max(
            self.unbonding_unlock_time,
            env::block_timestamp() + unbond_period,
        );
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_get_staker_or_panic(&self, staker_id: &StakerId) -> Staker {
        self.stakers
            .get(staker_id)
            .expect(format!("Failed to get staker by {}", staker_id).as_str())
    }

    pub(crate) fn internal_save_staker(&mut self, staker_id: &StakerId, staker: &Staker) {
        self.stakers.insert(staker_id, &staker);
    }

    pub(crate) fn internal_use_staker_or_panic<F, R>(&mut self, staker_id: &StakerId, mut f: F) -> R
    where
        F: FnMut(&mut Staker) -> R,
    {
        let mut staker = self.internal_get_staker_or_panic(staker_id);
        let r = f(&mut staker);
        self.internal_save_staker(staker_id, &staker);
        r
    }

    pub(crate) fn get_staker_staked_balance(&self, staker_id: &StakerId) -> Balance {
        let staker = self.internal_get_staker_or_panic(staker_id);
        let pool = self.internal_get_staking_pool_by_staker_or_panic(staker_id);
        return pool.staked_amount_from_shares_balance_rounded_down(staker.shares);
    }
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakerView {
    pub staker_id: StakerId,
    pub select_staking_pool: Option<PoolId>,
    pub shares: U128,
    pub max_bonding_unlock_period: U64,
    pub unbonding_unlock_time: U64,
}

impl From<Staker> for StakerView {
    fn from(value: Staker) -> Self {
        StakerView {
            staker_id: value.staker_id,
            select_staking_pool: value.select_staking_pool,
            shares: value.shares.into(),
            max_bonding_unlock_period: value.max_bonding_unlock_period.into(),
            unbonding_unlock_time: value.unbonding_unlock_time.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingChangeResult {
    pub sequence: Sequence,
    pub new_total_staked_balance: U128,
}
