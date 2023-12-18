use std::cmp::max;

use crate::types::{DurationOfSeconds, PoolId, Sequence, ShareBalance};
use crate::*;
use near_sdk::Duration;
use near_sdk::Timestamp;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Staker {
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

    pub fn bond(
        &mut self,
        consumer_chain_id: &ConsumerChainId,
        unbonding_period: DurationOfSeconds,
    ) {
        assert!(
            self.unbonding_unlock_time <= env::block_timestamp(),
            "Failed to bond by {}, the unbonding unlock time({}) should not greater then block time({}).",
            self.staker_id,
            self.unbonding_unlock_time,
            env::block_timestamp()
        );

        self.max_bonding_unlock_period = max(self.max_bonding_unlock_period, unbonding_period);
        self.bonding_consumer_chains
            .insert(consumer_chain_id, &unbonding_period);
    }

    pub fn update_unbonding_period(
        &mut self,
        consumer_chain_id: &ConsumerChainId,
        unbonding_period: DurationOfSeconds,
    ) {
        self.bonding_consumer_chains
            .insert(consumer_chain_id, &unbonding_period);
        self.max_bonding_unlock_period = self
            .bonding_consumer_chains
            .iter()
            .map(|e| e.1)
            .max()
            .unwrap_or(0);
    }

    pub fn unbond(&mut self, consumer_chain_id: &ConsumerChainId) {
        let unbonding_period = self
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
            env::block_timestamp() + seconds_to_nanoseconds(unbonding_period),
        );

        self.max_bonding_unlock_period = self
            .bonding_consumer_chains
            .iter()
            .map(|e| e.1)
            .max()
            .unwrap_or(0);
    }

    pub fn get_unlock_time(&self) -> Timestamp {
        max(
            self.unbonding_unlock_time,
            env::block_timestamp() + seconds_to_nanoseconds(self.max_bonding_unlock_period),
        )
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
pub struct StakerInfo {
    pub staker_id: StakerId,
    pub select_staking_pool: Option<PoolId>,
    pub shares: U128,
    pub max_bonding_unlock_period: U64,
    pub unbonding_unlock_time: U64,
}

impl From<&Staker> for StakerInfo {
    fn from(value: &Staker) -> Self {
        StakerInfo {
            staker_id: value.staker_id.clone(),
            select_staking_pool: value.select_staking_pool.clone(),
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
    pub withdrawal_certificate: Option<WithdrawalCertificate>,
}
