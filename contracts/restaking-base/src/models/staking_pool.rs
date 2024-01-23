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
    /// Last epoch for calling unstake method in staking pool.
    pub last_unstake_epoch: EpochHeight,
    /// Last unstake batch id, it'll be None if it's initial or withdrawn.
    pub last_unstake_batch_id: Option<UnstakeBatchId>,
    pub current_unstake_batch_id: UnstakeBatchId,
    pub batched_unstake_amount: u128,
    pub submitted_unstake_batches: UnorderedMap<UnstakeBatchId, SubmittedUnstakeBatch>,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmittedUnstakeBatch {
    pub unstake_batch_id: UnstakeBatchId,
    #[serde(with = "u64_dec_format")]
    pub submit_unstake_epoch: EpochHeight,
    #[serde(with = "u128_dec_format")]
    pub total_unstake_amount: u128,
    #[serde(with = "u128_dec_format")]
    pub claimed_amount: u128,
    pub is_withdrawn: bool,
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
    #[serde(with = "u64_dec_format")]
    pub last_unstake_epoch: EpochHeight,
    pub last_unstake_batch_id: Option<UnstakeBatchId>,
    pub current_unstake_batch_id: UnstakeBatchId,
    #[serde(with = "u128_dec_format")]
    pub batched_unstake_amount: u128,
    pub submitted_unstake_batches_count: u32,
}

impl From<&mut StakingPool> for StakingPoolInfo {
    fn from(value: &mut StakingPool) -> Self {
        Self {
            pool_id: value.pool_id.clone(),
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            locked: value.locked,
            unlock_epoch: value.unlock_epoch,
            last_unstake_epoch: value.last_unstake_epoch,
            last_unstake_batch_id: value.last_unstake_batch_id,
            current_unstake_batch_id: value.current_unstake_batch_id,
            batched_unstake_amount: value.batched_unstake_amount,
            submitted_unstake_batches_count: value.submitted_unstake_batches.len() as u32,
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
            last_unstake_epoch: value.last_unstake_epoch,
            last_unstake_batch_id: value.last_unstake_batch_id,
            current_unstake_batch_id: value.current_unstake_batch_id,
            batched_unstake_amount: value.batched_unstake_amount,
            submitted_unstake_batches_count: value.submitted_unstake_batches.len() as u32,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingPoolDetail {
    pub pool_id: AccountId,
    #[serde(with = "u128_dec_format")]
    pub total_share_balance: ShareBalance,
    #[serde(with = "u128_dec_format")]
    pub total_staked_balance: Balance,
    pub stakers: Vec<AccountId>,
    pub locked: bool,
    pub unlock_epoch: EpochHeight,
    #[serde(with = "u64_dec_format")]
    pub last_unstake_epoch: EpochHeight,
    pub last_unstake_batch_id: Option<UnstakeBatchId>,
    pub current_unstake_batch_id: UnstakeBatchId,
    #[serde(with = "u128_dec_format")]
    pub batched_unstake_amount: u128,
    pub submitted_unstake_batches: Vec<SubmittedUnstakeBatch>,
}

impl From<StakingPool> for StakingPoolDetail {
    fn from(value: StakingPool) -> Self {
        Self {
            pool_id: value.pool_id,
            total_share_balance: value.total_share_balance,
            total_staked_balance: value.total_staked_balance,
            stakers: value.stakers.iter().collect_vec(),
            locked: value.locked,
            unlock_epoch: value.unlock_epoch,
            last_unstake_epoch: value.last_unstake_epoch,
            last_unstake_batch_id: value.last_unstake_batch_id,
            current_unstake_batch_id: value.current_unstake_batch_id,
            batched_unstake_amount: value.batched_unstake_amount,
            submitted_unstake_batches: value.submitted_unstake_batches.values().collect_vec(),
        }
    }
}

impl StakingPool {
    pub fn new(pool_id: AccountId) -> Self {
        let pool = Self {
            pool_id: pool_id.clone(),
            total_share_balance: 0,
            total_staked_balance: 0,
            stakers: UnorderedSet::new(StorageKey::StakingPoolStakers {
                pool_id: pool_id.clone(),
            }),
            locked: false,
            unlock_epoch: 0,

            last_unstake_epoch: 0,
            last_unstake_batch_id: None,
            current_unstake_batch_id: 0.into(),
            batched_unstake_amount: 0,
            submitted_unstake_batches: UnorderedMap::new(StorageKey::SubmittedUnstakeBatches {
                pool_id: pool_id.clone(),
            }),
        };
        pool
    }

    pub fn is_able_submit_unstake_batch(&self) -> bool {
        self.batched_unstake_amount > 0 && self.last_unstake_batch_id.is_none()
    }

    pub fn is_unstake_batch_withdrawable(&self, unstake_batch_id: &UnstakeBatchId) -> bool {
        self.submitted_unstake_batches
            .get(&unstake_batch_id)
            .unwrap()
            .submit_unstake_epoch
            + NUM_EPOCHS_TO_UNLOCK
            <= env::epoch_height()
    }

    pub fn remain_staked_balance(&self) -> Balance {
        self.total_staked_balance
            .checked_sub(self.batched_unstake_amount)
            .unwrap()
    }

    pub fn batch_unstake(&mut self, amount: Balance) -> UnstakeBatchId {
        self.batched_unstake_amount += amount;
        self.current_unstake_batch_id.clone()
    }

    pub fn withdraw_from_unstake_batch(
        &mut self,
        amount: Balance,
        unstake_batch_id: UnstakeBatchId,
    ) {
        let mut submitted_unstake_batch = self
            .submitted_unstake_batches
            .get(&unstake_batch_id)
            .unwrap();
        assert!(submitted_unstake_batch.is_withdrawn);
        submitted_unstake_batch.claimed_amount += amount;
        assert!(
            submitted_unstake_batch.total_unstake_amount >= submitted_unstake_batch.claimed_amount
        );

        if submitted_unstake_batch.claimed_amount == submitted_unstake_batch.total_unstake_amount {
            self.submitted_unstake_batches.remove(&unstake_batch_id);
        } else {
            self.submitted_unstake_batches
                .insert(&unstake_batch_id, &submitted_unstake_batch);
        }
    }

    pub fn submit_unstake(&mut self) -> SubmittedUnstakeBatch {
        let submitted_unstake_batch = SubmittedUnstakeBatch {
            unstake_batch_id: self.current_unstake_batch_id,
            submit_unstake_epoch: env::epoch_height(),
            total_unstake_amount: self.batched_unstake_amount,
            claimed_amount: 0,
            is_withdrawn: false,
        };
        self.submitted_unstake_batches.insert(
            &self.current_unstake_batch_id,
            &SubmittedUnstakeBatch {
                unstake_batch_id: self.current_unstake_batch_id,
                submit_unstake_epoch: env::epoch_height(),
                total_unstake_amount: self.batched_unstake_amount,
                claimed_amount: 0,
                is_withdrawn: false,
            },
        );

        self.last_unstake_epoch = env::epoch_height();
        self.last_unstake_batch_id = Some(self.current_unstake_batch_id.clone());
        self.current_unstake_batch_id = (self.current_unstake_batch_id.0 + 1).into();
        self.batched_unstake_amount = 0;

        self.unlock_epoch = env::epoch_height() + NUM_EPOCHS_TO_UNLOCK;

        submitted_unstake_batch
    }

    pub fn withdraw_unstake_batch(&mut self, unstake_batch_id: &UnstakeBatchId) {
        let mut submitted_unstake_batch = self
            .submitted_unstake_batches
            .get(&unstake_batch_id)
            .unwrap();
        submitted_unstake_batch.is_withdrawn = true;
        self.submitted_unstake_batches
            .insert(&unstake_batch_id, &submitted_unstake_batch);
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

    pub fn is_unstake_batch_withdrawn(&self, unstake_batch_id: &UnstakeBatchId) -> bool {
        self.submitted_unstake_batches
            .get(unstake_batch_id)
            .is_some_and(|e| e.is_withdrawn)
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

    pub fn decrease_stake(&mut self, decrease_shares: ShareBalance) {
        self.total_share_balance = self
            .total_share_balance
            .checked_sub(decrease_shares)
            .expect("Failed to decrease shares");
    }

    pub fn unstake(&mut self, staker_id: &AccountId, decrease_shares: ShareBalance) {
        self.decrease_stake(decrease_shares);
        self.stakers.remove(&staker_id);
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
        let remain_staked_balance = self.remain_staked_balance();
        if remain_staked_balance == 0 {
            return stake_amount;
        }

        (U256::from(self.total_share_balance) * U256::from(stake_amount)
            / U256::from(remain_staked_balance))
        .as_u128()
    }

    /// Returns the number of "stake" shares rounded up corresponding to the given staked balance
    /// amount.
    ///
    /// Rounding up division of `a / b` is done using `(a + b - 1) / b`.
    pub fn num_shares_from_staked_amount_rounded_up(&self, amount: Balance) -> ShareBalance {
        let remain_staked_balance = self.remain_staked_balance();
        assert!(
            remain_staked_balance > 0,
            "The remain_staked_balance can't be 0"
        );
        ((U256::from(self.total_share_balance) * U256::from(amount)
            + U256::from(remain_staked_balance - 1))
            / U256::from(remain_staked_balance))
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

        let remain_staked_balance = self.remain_staked_balance();

        (U256::from(remain_staked_balance) * U256::from(share_balance)
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
