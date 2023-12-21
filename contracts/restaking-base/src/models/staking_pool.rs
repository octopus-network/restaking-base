use std::collections::HashSet;

use crate::types::{ShareBalance, U256};
use crate::*;
use near_sdk::{Balance, EpochHeight};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StakingPool {
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

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingPoolInfo {
    pub pool_id: AccountId,
    #[serde(with = "u128_dec_format")]
    pub total_share_balance: ShareBalance,
    #[serde(with = "u128_dec_format")]
    pub total_staked_balance: Balance,
    pub locked: bool,
    #[serde(with = "u64_dec_format")]
    pub unlock_epoch: EpochHeight,
}

impl From<&mut StakingPool> for StakingPoolInfo {
    fn from(value: &mut StakingPool) -> Self {
        Self {
            pool_id: value.pool_id.clone(),
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            locked: value.locked,
            unlock_epoch: value.unlock_epoch,
        }
    }
}

impl From<StakingPool> for StakingPoolInfo {
    fn from(value: StakingPool) -> Self {
        Self {
            pool_id: value.pool_id,
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            locked: value.locked,
            unlock_epoch: value.unlock_epoch,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingPoolDetail {
    pub pool_id: AccountId,
    #[serde(with = "u128_dec_format")]
    pub total_share_balance: ShareBalance,
    #[serde(with = "u128_dec_format")]
    pub total_staked_balance: Balance,
    pub stakers: HashSet<AccountId>,
    pub locked: bool,
    pub unlock_epoch: EpochHeight,
}

impl StakingPool {
    pub fn new(pool_id: AccountId, first_staker: AccountId) -> Self {
        let mut pool = Self {
            pool_id: pool_id.clone(),
            total_share_balance: 0,
            total_staked_balance: 0,
            stakers: UnorderedSet::new(StorageKey::StakingPoolStakers { pool_id: pool_id }),
            locked: false,
            unlock_epoch: 0,
        };
        pool.stakers.insert(&first_staker);
        pool
    }

    pub fn lock(&mut self) {
        assert!(!self.locked, "The staking pool has been already locked!");
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn is_withdrawable(&self) -> bool {
        self.unlock_epoch <= env::epoch_height()
    }

    pub fn stake(
        &mut self,
        staker: &mut Staker,
        increase_amount: Balance,
        new_total_staked_balance: Balance,
    ) -> ShareBalance {
        staker.select_staking_pool = Some(self.pool_id.clone());

        self.stakers.insert(&staker.staker_id);

        self.increase_stake(staker, increase_amount, new_total_staked_balance)
    }

    pub fn increase_stake(
        &mut self,
        staker: &mut Staker,
        increase_amount: Balance,
        new_total_staked_balance: Balance,
    ) -> ShareBalance {
        let increase_shares = self.calculate_increase_shares(increase_amount);

        self.total_share_balance += increase_shares;
        self.total_staked_balance = new_total_staked_balance;

        staker.shares += increase_shares;
        increase_shares
    }

    pub fn decrease_stake(
        &mut self,
        decrease_shares: ShareBalance,
        new_total_staked_balance: Balance,
    ) {
        self.total_share_balance -= decrease_shares;
        self.total_staked_balance = new_total_staked_balance;
        self.unlock_epoch = env::epoch_height() + NUM_EPOCHS_TO_UNLOCK;
    }

    pub fn unstake(
        &mut self,
        staker_id: &AccountId,
        decrease_shares: ShareBalance,
        new_total_staked_balance: u128,
    ) {
        self.decrease_stake(decrease_shares, new_total_staked_balance);
        self.stakers.remove(&staker_id);
        self.unlock_epoch = env::epoch_height() + NUM_EPOCHS_TO_UNLOCK;
    }

    pub fn calculate_increase_shares(&self, increase_near_amount: Balance) -> ShareBalance {
        assert!(
            increase_near_amount > 0,
            "Increase delegation amount should be positvie"
        );
        let increase_shares =
            self.share_balance_from_staked_amount_rounded_down(increase_near_amount);
        assert!(
			increase_shares>0,
            "Invariant violation. The calculated number of stake shares for unstaking should be positive"
		);

        let charge_amount = self.staked_amount_from_shares_balance_rounded_down(increase_shares);
        assert!(
            charge_amount > 0 && increase_near_amount >= charge_amount,
            "charge_amount: {}, increase_near_amount: {}",
            charge_amount,
            increase_near_amount
        );
        increase_shares
    }

    pub fn calculate_decrease_shares(&self, decrease_near_amount: Balance) -> ShareBalance {
        assert!(
            decrease_near_amount > 0,
            "Decrease near amount should be positive"
        );
        let decrease_shares = self.num_shares_from_staked_amount_rounded_up(decrease_near_amount);
        assert!(
            decrease_shares > 0,
            "Invariant violation. The calculated number of \"stake\" shares for unstaking should be positive"
        );

        decrease_shares
    }

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

        (U256::from(self.total_share_balance) * U256::from(stake_amount)
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
        ((U256::from(self.total_share_balance) * U256::from(amount)
            + U256::from(self.total_staked_balance - 1))
            / U256::from(self.total_staked_balance))
        .as_u128()
    }

    /// Returns the staked amount rounded down corresponding to the given number of "stake" shares.
    pub fn staked_amount_from_shares_balance_rounded_down(
        &self,
        share_balance: ShareBalance,
    ) -> Balance {
        if self.total_share_balance == 0 {
            return share_balance;
        }

        (U256::from(self.total_staked_balance) * U256::from(share_balance)
            / U256::from(self.total_share_balance))
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
            self.total_share_balance > 0,
            "The total number of stake shares can't be 0"
        );
        ((U256::from(self.total_staked_balance) * U256::from(share_balance)
            + U256::from(self.total_share_balance - 1))
            / U256::from(self.total_share_balance))
        .as_u128()
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_get_staking_pool_or_panic(&self, pool_id: &PoolId) -> StakingPool {
        self.staking_pools
            .get(pool_id)
            .expect(format!("Failed to get staking pool by {}", pool_id).as_str())
    }

    pub(crate) fn internal_save_staking_pool(&mut self, staking_pool: &StakingPool) {
        self.staking_pools
            .insert(&staking_pool.pool_id, &staking_pool);
    }

    pub(crate) fn internal_use_staker_staking_pool_or_panic<F, R>(
        &mut self,
        staker_id: &StakerId,
        mut f: F,
    ) -> R
    where
        F: FnMut(&mut StakingPool) -> R,
    {
        let mut staking_pool = self.internal_get_staking_pool_by_staker_or_panic(staker_id);
        let r = f(&mut staking_pool);
        self.internal_save_staking_pool(&staking_pool);
        r
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
        self.internal_save_staking_pool(&staking_pool);
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
