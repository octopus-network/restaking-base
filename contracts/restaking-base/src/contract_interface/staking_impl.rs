use std::{cmp::max, ops::Mul};

use itertools::Itertools;
use near_sdk::{
    env::current_account_id, serde_json::json, Gas, PromiseOrValue, PromiseResult, Timestamp,
    ONE_YOCTO,
};

use crate::{
    constants::{
        gas_constants::{
            NO_DEPOSIT, TGAS_FOR_DEPOSIT_AND_STAKE, TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE,
            TGAS_FOR_IS_WHITELISTED, TGAS_FOR_PING, TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST,
            TGAS_FOR_UNSTAKE, TGAS_FOR_WITHDRAW,
        },
        NUM_EPOCHS_TO_UNLOCK,
    },
    external::{staking_pool::ext_staking_pool, staking_pool_whitelist::ext_whitelist},
    models::{consumer_chain::ConsumerChainView, staker::StakerView},
    types::{ShareBalance, WithdrawalReceiptId},
    *,
};

use super::staking::*;

#[near_bindgen]
impl StakerAction for RestakingBaseContract {
    fn select_pool(&mut self, pool_id: PoolId) -> Promise {
        assert_one_yocto();

        // todo first selector pay pool storage fee.

        let staker_id = env::predecessor_account_id();

        let staker_shares = self
            .stakers
            .get(&staker_id)
            .map(|staker| staker.shares)
            .unwrap_or(0);

        assert_eq!(staker_shares, 0, "Can't select pool, shares is not zero");

        return ext_whitelist::ext(self.staking_pool_whitelist_account.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_IS_WHITELISTED))
            .is_whitelisted(pool_id.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST))
                    .select_pool_after_check_whitelisted(staker_id, pool_id),
            );
    }

    fn ping(&mut self, pool_id: Option<PoolId>) -> Promise {
        let ping_pool_id = pool_id.unwrap_or_else(|| {
            self.stakers
                .get(&env::predecessor_account_id())
                .and_then(|pool| pool.select_staking_pool.clone())
                .expect("Can't choose a pool to ping!")
        });

        ext_staking_pool::ext(ping_pool_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_PING))
            .ping()
            .function_call(
                "get_account_staked_balance".to_string(),
                json!({ "account_id": env::current_account_id() })
                    .to_string()
                    .into_bytes(),
                NO_DEPOSIT,
                Gas::ONE_TERA.mul(TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE),
            )
            .then(Self::ext(current_account_id()).ping_callback())
    }

    #[payable]
    fn increase_stake(&mut self) -> PromiseOrValue<U128> {
        assert_attached_near();

        let increase_amount = env::attached_deposit();
        let staker_id = env::predecessor_account_id();

        return self
            .ping(Option::None)
            .then(
                Self::ext(current_account_id())
                    .increase_stake_after_ping(staker_id, increase_amount.into()),
            )
            .into();
    }

    #[payable]
    fn decrease_stake(
        &mut self,
        decrease_amount: U128,
    ) -> PromiseOrValue<Option<WithdrawalReceiptId>> {
        assert!(decrease_amount.0 > 0, "The decrease amount should gt 0");

        let staker_id = env::predecessor_account_id();

        return self
            .ping(Option::None)
            .then(
                Self::ext(current_account_id())
                    .decrease_stake_after_ping(staker_id, Some(decrease_amount)),
            )
            .into();
    }

    #[payable]
    fn unstake(&mut self) -> PromiseOrValue<Option<WithdrawalReceiptId>> {
        let staker_id = env::predecessor_account_id();
        let staker = self.internal_get_staker_or_panic(&staker_id);
        for bonding_consumer_chain in staker.bonding_consumer_chains.iter() {
            self.internal_unbond(&staker_id, &bonding_consumer_chain);
        }
        return self
            .ping(Option::None)
            .then(Self::ext(current_account_id()).decrease_stake_after_ping(staker_id, None))
            .into();
    }

    fn withdraw_all(&mut self, account_id: AccountId, pool_id: PoolId) -> PromiseOrValue<U128> {
        // todo use order map to withdraw
        let account = self.internal_get_account_or_panic(&account_id);

        let withdrawable_pending_withdrawals = account
            .pending_withdrawals
            .values()
            .filter(|e| e.pool_id.eq(&pool_id) && e.is_withdrawable())
            .collect_vec();

        let withdraw_amount: u128 = withdrawable_pending_withdrawals
            .iter()
            .map(|e| e.amount.to_owned())
            .sum();

        ext_staking_pool::ext(account_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_WITHDRAW))
            .withdraw(withdraw_amount.into())
            .then(
                Self::ext(env::current_account_id()).withdraw_callback(
                    account_id,
                    withdrawable_pending_withdrawals
                        .iter()
                        .map(|e| e.withdrawal_certificate)
                        .collect_vec(),
                ),
            )
            .into()
    }
}

#[near_bindgen]
impl StakeView for RestakingBaseContract {
    fn get_staker(&self, staker_id: StakerId) -> Option<StakerView> {
        self.stakers.get(&staker_id).map(|e| e.into())
    }

    fn get_staker_bonding_consumer_chains(
        &self,
        staker_id: StakerId,
        skip: u32,
        limit: u32,
    ) -> Vec<ConsumerChainView> {
        self.stakers
            .get(&staker_id)
            .and_then(|staker| {
                Some(
                    staker
                        .bonding_consumer_chains
                        .iter()
                        .skip(skip as usize)
                        .take(limit as usize)
                        .map(|chain_id| self.consumer_chains.get(&chain_id).unwrap())
                        .map(ConsumerChainView::from)
                        .collect(),
                )
            })
            .unwrap_or(vec![])
    }
}

#[near_bindgen]
impl StakingCallBack for RestakingBaseContract {
    #[private]
    fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        withdrawal_certificates: Vec<WithdrawalReceiptId>,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut storage_manager = self.internal_get_storage_manager_or_panic(&account_id);
                storage_manager.execute_in_storage_monitoring(|| {
                    self.internal_use_account(&account_id, |account| {
                        for withdrawal_certificate in &withdrawal_certificates {
                            account.pending_withdrawals.remove(withdrawal_certificate);
                        }
                    });
                });
                self.internal_save_storage_manager(&account_id, &storage_manager);
            }
            PromiseResult::Failed => {}
        }
    }

    #[private]
    fn decrease_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        decrease_amount: Option<U128>,
        #[callback] staked_balance: U128,
    ) -> PromiseOrValue<Option<WithdrawalReceiptId>> {
        let pool_id: AccountId = self.internal_get_staker_selected_pool_or_panic(&staker_id);

        let unwraped_decrease_amount = decrease_amount.unwrap_or_else(|| {
            let staker = self.internal_get_staker_or_panic(&staker_id);
            self.internal_use_staking_pool_or_panic(&pool_id, |pool| {
                pool.total_staked_balance = staked_balance.0;
                pool.staked_amount_from_shares_balance_rounded_down(staker.shares)
                    .into()
            })
        });

        let (decrease_shares, receive_amount) =
            self.internal_decrease_stake(&staker_id, unwraped_decrease_amount.0);

        // todo check storage for call successful

        ext_staking_pool::ext(pool_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
            .with_attached_deposit(ONE_YOCTO)
            .unstake(unwraped_decrease_amount)
            .function_call(
                "get_account_staked_balance".to_string(),
                json!({ "account_id": env::current_account_id() })
                    .to_string()
                    .into_bytes(),
                0,
                Gas::ONE_TERA.mul(TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
                    .decrease_stake_callback(
                        staker_id,
                        decrease_shares.into(),
                        receive_amount.into(),
                        None,
                    ),
            )
            .into()
    }

    #[private]
    fn decrease_stake_callback(
        &mut self,
        staker_id: AccountId,
        decrease_shares: U128,
        decrease_amount: U128,
        slash_governance: Option<AccountId>,
    ) -> PromiseOrValue<Option<WithdrawalReceiptId>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in decrease_stake_callback by value.")
                    .0;
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);

                self.internal_use_staking_pool_or_panic(&selected_pool_id, |staking_pool| {
                    staking_pool.total_staked_balance = total_staked_balance;
                });

                let mut staker = self.internal_get_staker_or_panic(&staker_id);

                let withdraw_certificate = self.internal_create_pending_withdrawal_for_staker(
                    &mut staker,
                    decrease_amount.0,
                    env::predecessor_account_id(),
                );
                PromiseOrValue::Value(Some(withdraw_certificate))
            }
            PromiseResult::Failed => {
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);

                match slash_governance {
                    Some(governance_account_id) => {
                        let mut slash_governance_account =
                            self.internal_get_account_or_panic(&governance_account_id);
                        slash_governance_account
                            .save_legacy_shares(decrease_shares.0, selected_pool_id.clone())
                    }
                    None => {
                        self.internal_decrease_stake_rollback(
                            &staker_id,
                            &selected_pool_id,
                            decrease_shares.0,
                        );
                    }
                }
                PromiseOrValue::Value(None)
            }
        }
    }

    #[private]
    fn increase_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        increase_amount: U128,
    ) -> PromiseOrValue<U128> {
        let pool_id: AccountId = self.internal_get_staker_selected_pool_or_panic(&staker_id);
        let increase_share_balance = self.internal_increase_stake(increase_amount.0);

        ext_staking_pool::ext(pool_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DEPOSIT_AND_STAKE))
            .with_attached_deposit(increase_amount.0)
            .deposit_and_stake()
            .function_call(
                "get_account_staked_balance".to_string(),
                json!({ "account_id": env::current_account_id() })
                    .to_string()
                    .into_bytes(),
                0,
                Gas::ONE_TERA.mul(TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
                    .increase_stake_callback(
                        staker_id,
                        increase_share_balance.into(),
                        increase_amount,
                    ),
            )
            .into()
    }

    #[private]
    fn increase_stake_callback(
        &mut self,
        staker_id: AccountId,
        increase_shares: U128,
        increase_amount: U128,
    ) -> PromiseOrValue<U128> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                //todo udpate staking pool total_staked_balance
                let total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in increase_stake_callback by value.")
                    .0;
                let mut pool = self.internal_get_staking_pool_by_staker_or_panic(&staker_id);
                pool.total_staked_balance = total_staked_balance;
                return PromiseOrValue::Value(increase_shares);
            }
            PromiseResult::Failed => {
                let pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);
                self.internal_increase_stake_rollback(&staker_id, &pool_id, increase_shares.0);
                self.transfer_near(staker_id, increase_amount.0);
                return PromiseOrValue::Value(0.into());
            }
        };
    }

    #[private]
    fn select_pool_after_check_whitelisted(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
        #[callback] whitelisted: bool,
    ) -> PromiseOrValue<bool> {
        if !whitelisted {
            return PromiseOrValue::Value(false);
        }

        self.internal_use_staker_or_panic(&staker_id, |staker| {
            staker.select_staking_pool = Some(pool_id.clone());
        });

        return PromiseOrValue::Value(true);
    }

    #[private]
    fn ping_callback(&mut self, #[callback] staked_balance: U128) {
        let pool_id = env::predecessor_account_id();
        let mut pool = self.staking_pools.get(&pool_id).expect(
            format!(
                "Failed to get staking pool by {} when ping callback.",
                pool_id
            )
            .as_str(),
        );
        pool.total_staked_balance = staked_balance.0;
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_create_pending_withdrawal_for_staker(
        &mut self,
        staker: &Staker,
        amount: Balance,
        pool_id: PoolId,
    ) -> WithdrawalReceiptId {
        let pending_withdrawal = PendingWithdrawal::new(
            self.next_uuid().into(),
            staker.staker_id.clone(),
            pool_id,
            amount,
            env::epoch_height() + NUM_EPOCHS_TO_UNLOCK,
            env::block_timestamp() + self.get_staker_unlock_time(&&staker.staker_id.clone()),
        );

        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker.staker_id);

        storage_manager.execute_in_storage_monitoring(|| {
            self.internal_use_account(&staker.staker_id, |account| {
                account.pending_withdrawals.insert(
                    &pending_withdrawal.withdrawal_certificate,
                    &pending_withdrawal,
                );
            });
        });

        pending_withdrawal.withdrawal_certificate
    }

    pub(crate) fn get_staker_unlock_time(&self, staker_id: &StakerId) -> Timestamp {
        let staker = self.internal_get_staker_or_panic(staker_id);
        max(
            staker.max_bonding_unlock_period,
            staker.unbonding_unlock_time,
        )
    }

    pub(crate) fn internal_get_staker_selected_pool_or_panic(
        &self,
        account_id: &AccountId,
    ) -> PoolId {
        self.stakers
            .get(account_id)
            .and_then(|staker| staker.select_staking_pool.clone())
            .expect("The staker haven't select pool!")
    }

    pub(crate) fn internal_decrease_stake(
        &mut self,
        staker_id: &StakerId,
        decrease_amount: Balance,
    ) -> (ShareBalance, Balance) {
        assert!(decrease_amount > 0, "Staking amount should be positive");
        let mut staker = self.internal_get_staker_or_panic(staker_id);
        let pool_id = &self.internal_get_staker_selected_pool_or_panic(staker_id);
        let mut staking_pool = self.internal_get_staking_pool_or_panic(pool_id);

        // Calculate the number of shares required to unstake the given amount.
        // NOTE: The number of shares the account will pay is rounded up.
        let num_shares = staking_pool.num_shares_from_staked_amount_rounded_up(decrease_amount);
        assert!(
            num_shares > 0,
            "Invariant violation. The calculated number of stake shares for unstaking should be positive"
        );
        assert!(
            staker.shares >= num_shares,
            "Not enough staked balance to unstake"
        );

        staking_pool.total_staked_shares -= num_shares;
        staker.shares -= num_shares;

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let receive_amount = staking_pool.staked_amount_from_shares_balance_rounded_up(num_shares);
        assert!(
            receive_amount > 0,
            "Invariant violation. Calculated staked amount must be positive, because stake share price should be at least 1"
        );

        self.internal_save_staking_pool(pool_id, &staking_pool);
        self.internal_save_staker(pool_id, &staker);

        (num_shares, receive_amount)
    }

    pub(crate) fn internal_decrease_stake_rollback(
        &mut self,
        staker_id: &StakerId,
        pool_id: &PoolId,
        decrease_share: ShareBalance,
    ) {
        let mut staker = self.internal_get_staker_or_panic(staker_id);
        // let staking_pool = self.get_mut_staking_pool_or_panic(pool_id);
        let mut staking_pool = self.internal_get_staking_pool_or_panic(pool_id);

        staker.shares += decrease_share;
        staking_pool.total_staked_shares += decrease_share;
    }

    pub(crate) fn internal_increase_stake_rollback(
        &mut self,
        staker_id: &StakerId,
        pool_id: &PoolId,
        increase_shares: ShareBalance,
    ) {
        let mut staker = self.internal_get_staker_or_panic(staker_id);
        let mut staking_pool = self.internal_get_staking_pool_or_panic(pool_id);

        staker.shares -= increase_shares;
        staking_pool.total_staked_shares -= increase_shares;
    }

    pub(crate) fn internal_increase_stake(
        &mut self,
        increase_stake_amount: Balance,
    ) -> ShareBalance {
        assert!(
            increase_stake_amount > 0,
            "Staking amount should be positive"
        );
        let staker_id = env::predecessor_account_id();
        let mut staker = self.internal_get_staker_or_panic(&staker_id);
        let pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);
        let staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);

        let increase_shares =
            staking_pool.share_balance_from_staked_amount_rounded_down(increase_stake_amount);
        assert!(
            increase_shares > 0,
            "The calculated number of increase stake shares should be positive"
        );

        // The amount of tokens the account will be charged from the unstaked balance.
        // Rounded down to avoid overcharging the account to guarantee that the account can always
        // unstake at least the same amount as staked.
        let charge_amount =
            staking_pool.staked_amount_from_shares_balance_rounded_down(increase_shares);
        assert!(
            charge_amount > 0 && increase_stake_amount >= charge_amount,
            "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1"
        );

        staker.shares += increase_shares;

        increase_shares
    }
}
