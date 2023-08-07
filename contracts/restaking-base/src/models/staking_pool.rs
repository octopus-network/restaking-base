use crate::types::{ShareBalance, U256};
use crate::*;
use near_sdk::Balance;

#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingPool {
    pub pool_id: AccountId,
    #[serde(default)]
    #[serde(with = "u128_dec_format")]
    pub total_staked_shares: ShareBalance,
    #[serde(default)]
    #[serde(with = "u128_dec_format")]
    pub total_staked_balance: Balance,
    // todo 是否需要一个这样的索引
    // pub shares: UnorderedMap<AccountId, ShareBalance>
}

impl StakingPool {
    /// Returns the number of "stake" shares rounded down corresponding to the given staked balance
    /// amount.
    ///
    /// price = total_staked / total_shares
    /// Price is fixed
    /// (total_staked + amount) / (total_shares + num_shares) = total_staked / total_shares
    /// (total_staked + amount) * total_shares = total_staked * (total_shares + num_shares)
    /// amount * total_shares = total_staked * num_shares
    /// num_shares = amount * total_shares / total_staked
    pub fn share_balance_from_staked_amount_rounded_down(
        &self,
        stake_amount: Balance,
    ) -> ShareBalance {
        if self.total_staked_balance == 0 {
            return stake_amount;
        }

        (U256::from(self.total_staked_shares) * U256::from(stake_amount)
            / U256::from(self.total_staked_balance))
        .as_u128()
    }

    /// Returns the number of "stake" shares rounded up corresponding to the given staked balance
    /// amount.
    ///
    /// Rounding up division of `a / b` is done using `(a + b - 1) / b`.
    pub fn num_shares_from_staked_amount_rounded_up(&self, amount: Balance) -> ShareBalance {
        assert!(
            self.total_staked_balance > 0,
            "The total staked balance can't be 0"
        );
        ((U256::from(self.total_staked_shares) * U256::from(amount)
            + U256::from(self.total_staked_balance - 1))
            / U256::from(self.total_staked_balance))
        .as_u128()
    }

    /// Returns the staked amount rounded down corresponding to the given number of "stake" shares.
    pub fn staked_amount_from_shares_balance_rounded_down(
        &self,
        share_balance: ShareBalance,
    ) -> Balance {
        if self.total_staked_shares == 0 {
            return share_balance;
        }

        (U256::from(self.total_staked_balance) * U256::from(share_balance)
            / U256::from(self.total_staked_shares))
        .as_u128()
    }

    /// Returns the staked amount rounded up corresponding to the given number of "stake" shares.
    ///
    /// Rounding up division of `a / b` is done using `(a + b - 1) / b`.
    pub(crate) fn staked_amount_from_shares_balance_rounded_up(
        &self,
        share_balance: ShareBalance,
    ) -> Balance {
        assert!(
            self.total_staked_shares > 0,
            "The total number of stake shares can't be 0"
        );
        ((U256::from(self.total_staked_balance) * U256::from(share_balance)
            + U256::from(self.total_staked_shares - 1))
            / U256::from(self.total_staked_shares))
        .as_u128()
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_get_staking_pool_or_panic(&self, pool_id: &PoolId) -> StakingPool {
        self.staking_pools
            .get(pool_id)
            .expect(format!("Failed to get staking pool by {}", pool_id).as_str())
    }

    pub(crate) fn internal_save_staking_pool(
        &mut self,
        pool_id: &PoolId,
        staking_pool: &StakingPool,
    ) {
        self.staking_pools.insert(pool_id, &staking_pool);
    }

    pub(crate) fn internal_use_staking_pool_or_panic<F, R>(
        &mut self,
        pool_id: &PoolId,
        mut f: F,
    ) -> R
    where
        F: FnMut(&mut StakingPool) -> R,
    {
        let mut staking_pool = self.internal_get_staking_pool_or_panic(pool_id);
        let r = f(&mut staking_pool);
        self.internal_save_staking_pool(pool_id, &staking_pool);
        r
    }

    pub(crate) fn internal_get_staking_pool_by_staker_or_panic(
        &self,
        staker_id: &StakerId,
    ) -> StakingPool {
        let pool_id = &self.internal_get_staker_selected_pool_or_panic(staker_id);
        self.staking_pools
            .get(pool_id)
            .expect(format!("Failed to get staking pool by {}", staker_id).as_str())
    }
}
