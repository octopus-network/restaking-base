use near_sdk::env::current_account_id;

use crate::{types::Sequence, *};

#[near_bindgen]
impl StakerAction for RestakingBaseContract {
    #[payable]
    fn stake(&mut self, pool_id: PoolId) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        assert_attached_near();

        let staker_id = env::predecessor_account_id();

        assert!(
            self.accounts.get(&staker_id).is_some(),
            "Should register by storage_deposit first."
        );

        let staker = self
            .stakers
            .get(&staker_id)
            .unwrap_or(Staker::new(staker_id.clone()));

        assert_eq!(staker.shares, 0, "Can't stake, shares is not zero");
        assert!(
            staker.select_staking_pool.is_none(),
            "Staker({}) have selected pool({:?}). Need unstake first.",
            staker_id,
            staker.select_staking_pool
        );

        self.internal_save_staker(&staker_id, &staker);

        return ext_whitelist::ext(self.staking_pool_whitelist_account.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_IS_WHITELISTED))
            .is_whitelisted(pool_id.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST))
                    .stake_after_check_whitelisted(staker_id.clone(), pool_id.clone()),
            )
            .into();
    }

    fn ping(&mut self, pool_id: Option<PoolId>) -> Promise {
        self.assert_contract_is_running();
        let ping_pool_id = pool_id.unwrap_or_else(|| {
            self.stakers
                .get(&env::predecessor_account_id())
                .and_then(|pool| pool.select_staking_pool.clone())
                .expect("Can't choose a pool to ping!")
        });

        ext_staking_pool::ext(ping_pool_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_PING))
            .with_unused_gas_weight(0)
            .ping()
            .function_call(
                "get_account_staked_balance".to_string(),
                json!({ "account_id": env::current_account_id() })
                    .to_string()
                    .into_bytes(),
                NO_DEPOSIT,
                Gas::ONE_TERA.mul(TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_PING_CALLBACK))
                    .with_unused_gas_weight(0)
                    .ping_callback(ping_pool_id),
            )
    }

    #[payable]
    fn increase_stake(&mut self) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        assert_attached_near();

        let staker_id = env::predecessor_account_id();
        self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
            staking_pool.lock()
        });

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_AFTER_PING))
                    .increase_stake_after_ping(staker_id),
            )
            .into();
    }

    #[payable]
    fn decrease_stake(
        &mut self,
        decrease_amount: U128,
        beneficiary: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        self.assert_attached_storage_fee();
        assert!(decrease_amount.0 > 0, "The decrease amount should gt 0");

        let staker_id = env::predecessor_account_id();

        self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
            staking_pool.lock()
        });

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_AFTER_PING))
                    .decrease_stake_after_ping(
                        staker_id,
                        decrease_amount,
                        beneficiary.unwrap_or(env::predecessor_account_id()),
                    ),
            )
            .into();
    }

    #[payable]
    fn unstake(
        &mut self,
        beneficiary: Option<AccountId>,
        withdraw_by_anyone: Option<bool>,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        self.assert_attached_storage_fee();
        log!("Prepaid gas: {:?}", env::prepaid_gas());
        let staker_id = env::predecessor_account_id();

        self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
            staking_pool.lock()
        });

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE_AFTER_PING))
                    .unstake_after_ping(
                        staker_id.clone(),
                        beneficiary.unwrap_or(staker_id.clone()),
                        withdraw_by_anyone.unwrap_or(true),
                    ),
            )
            .into();
    }

    fn withdraw_unstake_batch(&mut self, pool_id: PoolId, unstake_batch_id: UnstakeBatchId) {
        self.assert_contract_is_running();
        let submitted_unstake_batch =
            self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                assert!(staking_pool.is_unstake_batch_withdrawable(&unstake_batch_id));

                staking_pool.lock();
                staking_pool
                    .submitted_unstake_batches
                    .get(&unstake_batch_id)
                    .unwrap()
            });

        ext_staking_pool::ext(pool_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_WITHDRAW))
            .with_unused_gas_weight(0)
            .withdraw(submitted_unstake_batch.total_unstake_amount.into())
            .then(
                Self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_WITHDRAW_UNSTAKE_BATCH_CALLBACK))
                    .with_unused_gas_weight(0)
                    .withdraw_unstake_batch_callback(pool_id.clone(), unstake_batch_id),
            );
    }

    fn submit_unstake_batch(&mut self, pool_id: PoolId) {
        self.assert_contract_is_running();
        let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);
        assert!(staking_pool.is_able_submit_unstake_batch());

        staking_pool.lock();

        self.internal_save_staking_pool(&staking_pool);

        ext_staking_pool::ext(pool_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
            .with_unused_gas_weight(0)
            .unstake(staking_pool.batched_unstake_amount.into())
            .then(
                Self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE_BATCH_CALLBACK))
                    .with_unused_gas_weight(0)
                    .submit_unstake_batch_callback(pool_id),
            );
    }

    fn withdraw(&mut self, staker: AccountId, id: WithdrawalCertificate) -> PromiseOrValue<U128> {
        self.assert_contract_is_running();
        let pending_withdrawal = self.internal_use_account(&staker, |account| {
            account.pending_withdrawals.remove(&id).unwrap()
        });
        let mut staking_pool = self.internal_get_staking_pool_or_panic(&pending_withdrawal.pool_id);
        assert!(
            self.internal_is_withdrawable(&staking_pool, &pending_withdrawal),
            "unlock timestamp:{}, current timestamp:{}, current epoch: {}",
            pending_withdrawal.unlock_time,
            env::block_timestamp(),
            env::epoch_height(),
        );
        assert!(
            pending_withdrawal.allow_other_withdraw
                || env::predecessor_account_id().eq(&pending_withdrawal.beneficiary)
        );

        if let Some(unstake_batch_id) = pending_withdrawal.unstake_batch_id {
            let submitted_unstake_batch = staking_pool
                .submitted_unstake_batches
                .get(&unstake_batch_id)
                .unwrap();
            assert!(submitted_unstake_batch.is_withdrawn);
            staking_pool.withdraw_from_unstake_batch(pending_withdrawal.amount, unstake_batch_id);
            self.internal_save_staking_pool(&staking_pool);

            self.transfer_near(pending_withdrawal.beneficiary, pending_withdrawal.amount);
            Event::Withdraw {
                withdrawal_certificate: &pending_withdrawal.withdrawal_certificate,
            }
            .emit();

            PromiseOrValue::Value(pending_withdrawal.amount.into())
        } else {
            ext_staking_pool::ext(pending_withdrawal.pool_id.clone())
                .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_WITHDRAW))
                .withdraw(pending_withdrawal.amount.into())
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_SINGLE_WITHDRAW_CALLBACK))
                        .withdraw_callback(staker, pending_withdrawal),
                )
                .into()
        }
    }
}

#[near_bindgen]
impl StakeView for RestakingBaseContract {
    fn get_staker(&self, staker_id: StakerId) -> Option<StakerInfo> {
        self.stakers.get(&staker_id).map(|e| (&e).into())
    }

    fn get_pending_withdrawals(&self, account_id: AccountId) -> Vec<PendingWithdrawal> {
        let account = self.internal_get_account_or_panic(&account_id);
        account.pending_withdrawals.values().collect_vec()
    }

    fn get_staker_bonding_consumer_chains(
        &self,
        staker_id: StakerId,
        skip: u32,
        limit: u32,
    ) -> Vec<ConsumerChainInfo> {
        self.stakers
            .get(&staker_id)
            .and_then(|staker| {
                Some(
                    staker
                        .bonding_consumer_chains
                        .iter()
                        .skip(skip as usize)
                        .take(limit as usize)
                        .map(|chain_id| self.consumer_chains.get(&chain_id.0).unwrap())
                        .map(ConsumerChainInfo::from)
                        .collect(),
                )
            })
            .unwrap_or(vec![])
    }

    fn get_staking_pool(&self, pool_id: PoolId) -> StakingPoolDetail {
        self.internal_get_staking_pool_or_panic(&pool_id).into()
    }

    fn get_staking_pools(&self) -> Vec<StakingPoolInfo> {
        self.staking_pools.values().map_into().collect_vec()
    }

    fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        self.get_staker_staked_balance(&account_id).into()
    }

    fn get_current_sequence(&self) -> Sequence {
        self.sequence.into()
    }

    fn get_current_epoch_height(&self) -> U64 {
        env::epoch_height().into()
    }

    fn is_withdrawable(&self, staker_id: StakerId, certificate: WithdrawalCertificate) -> bool {
        let pending_withdrawal = self
            .internal_get_account_or_panic(&staker_id)
            .pending_withdrawals
            .get(&certificate)
            .unwrap();

        let staking_pool = self.internal_get_staking_pool_or_panic(&pending_withdrawal.pool_id);

        self.internal_is_withdrawable(&staking_pool, &pending_withdrawal)
    }
}

#[near_bindgen]
impl StakingCallback for RestakingBaseContract {
    #[private]
    fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        pending_withdrawal: PendingWithdrawal,
    ) -> PromiseOrValue<U128> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.transfer_near(pending_withdrawal.beneficiary, pending_withdrawal.amount);
                Event::Withdraw {
                    withdrawal_certificate: &pending_withdrawal.withdrawal_certificate,
                }
                .emit();
                PromiseOrValue::Value(pending_withdrawal.amount.into())
            }
            PromiseResult::Failed => {
                self.internal_use_account(&account_id, |account| {
                    account.rollback_pending_withdrawals(&pending_withdrawal)
                });
                emit_callback_failed_event();
                PromiseOrValue::Value(0.into())
            }
        }
    }

    #[private]
    fn unstake_after_ping(
        &mut self,
        staker_id: AccountId,
        beneficiary: AccountId,
        withdraw_by_anyone: bool,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
            PromiseResult::Successful(_) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let mut staking_pool =
                    self.internal_get_staking_pool_by_staker_or_panic(&staker_id);

                staking_pool.unlock();

                let decrease_shares = staker.shares;
                let receive_amount =
                    staking_pool.staked_amount_from_shares_balance_rounded_up(decrease_shares);
                staker.shares = 0;

                let unstake_batch_id = staking_pool.batch_unstake(receive_amount);

                let pending_withdrawal = self.internal_create_pending_withdrawal_in_staker(
                    &staker,
                    beneficiary,
                    receive_amount,
                    staking_pool.pool_id.clone(),
                    withdraw_by_anyone,
                    unstake_batch_id.clone(),
                );

                let staker_bonding_consumer_chains =
                    staker.bonding_consumer_chains.keys().collect_vec();
                for consumer_chain_id in &staker_bonding_consumer_chains {
                    self.internal_use_consumer_chain_or_panic(
                        &consumer_chain_id,
                        |consumer_chain| consumer_chain.unbond(&staker_id),
                    );
                }

                staker.unstake();

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_staking_pool(&staking_pool);

                let sequence = U64(self.next_sequence());

                Event::StakerUnstake {
                    staking_pool_info: &(&mut staking_pool).into(),
                    staker_info: &(&staker).into(),
                    decrease_stake_amount: &receive_amount.into(),
                    decrease_shares: &decrease_shares.into(),
                    pending_withdrawal: &pending_withdrawal,
                    sequence: &sequence,
                    unstake_batch_id: &unstake_batch_id,
                }
                .emit();

                PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence: sequence,
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                    withdrawal_certificate: Some(pending_withdrawal.withdrawal_certificate),
                }))
            }
        }
    }

    #[private]
    fn decrease_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        decrease_amount: U128,
        beneficiary: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
            PromiseResult::Successful(_) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let mut staking_pool =
                    self.internal_get_staking_pool_by_staker_or_panic(&staker_id);

                staking_pool.unlock();

                let decrease_shares = staking_pool.calculate_decrease_shares(decrease_amount.0);
                let receive_amount =
                    staking_pool.staked_amount_from_shares_balance_rounded_up(decrease_shares);
                staker.shares = staker
                    .shares
                    .checked_sub(decrease_shares)
                    .expect("Failed decrease shares in staker.");

                staking_pool.decrease_stake(decrease_shares);

                let unstake_batch_id = staking_pool.batch_unstake(receive_amount);

                let pending_withdrawal = self.internal_create_pending_withdrawal_in_staker(
                    &mut staker,
                    beneficiary,
                    receive_amount,
                    staking_pool.pool_id.clone(),
                    true,
                    unstake_batch_id.clone(),
                );

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_staking_pool(&staking_pool);

                let sequence = U64::from(self.next_sequence());
                Event::StakerDecreaseStake {
                    staking_pool_info: &(&mut staking_pool).into(),
                    staker_info: &(&staker).into(),
                    decrease_stake_amount: &decrease_amount,
                    decrease_shares: &decrease_shares.into(),
                    pending_withdrawal: &pending_withdrawal,
                    sequence: &sequence,
                    unstake_batch_id: &unstake_batch_id,
                }
                .emit();

                PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence: sequence,
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                    withdrawal_certificate: None,
                }))
            }
        }
    }

    #[payable]
    #[private]
    fn stake_after_ping(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => ext_staking_pool::ext(pool_id.clone())
                .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DEPOSIT_AND_STAKE))
                .with_attached_deposit(env::attached_deposit())
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
                        .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_CALL_BACK))
                        .stake_callback(staker_id, env::attached_deposit().into(), pool_id.clone()),
                )
                .into(),
            PromiseResult::Failed => {
                log!("Failed to increase stake by ping error.");
                self.transfer_near(staker_id, env::attached_deposit());
                emit_callback_failed_event();
                return PromiseOrValue::Value(None);
            }
        }
    }

    #[payable]
    #[private]
    fn increase_stake_after_ping(
        &mut self,
        staker_id: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        log!("increase_stake_after_ping, gas: {:?}", env::prepaid_gas());
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let pool_id: AccountId =
                    self.internal_get_staker_selected_pool_or_panic(&staker_id);

                ext_staking_pool::ext(pool_id)
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DEPOSIT_AND_STAKE))
                    .with_attached_deposit(env::attached_deposit())
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
                            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_CALL_BACK))
                            .increase_stake_callback(staker_id, env::attached_deposit().into()),
                    )
                    .into()
            }
            PromiseResult::Failed => {
                log!("Failed to increase stake by ping error.");
                self.transfer_near(staker_id, env::attached_deposit());
                emit_callback_failed_event();
                return PromiseOrValue::Value(None);
            }
        }
    }

    #[private]
    fn stake_callback(
        &mut self,
        staker_id: AccountId,
        stake_amount: U128,
        pool_id: PoolId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in increase_stake_callback by value.")
                    .0;

                let mut staker = self.internal_get_staker_or_panic(&staker_id);

                let sequence = U64(self.next_sequence());

                let staker_new_balance =
                    self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                        let increase_shares = staking_pool.stake(
                            &mut staker,
                            stake_amount.0,
                            new_total_staked_balance,
                        );
                        staking_pool.unlock();

                        Event::StakerStake {
                            staking_pool_info: &staking_pool.into(),
                            staker_info: &(&staker).into(),
                            select_pool: &staking_pool.pool_id,
                            stake_amount: &stake_amount,
                            increase_shares: &increase_shares.into(),
                            sequence: &sequence,
                        }
                        .emit();
                        staking_pool.staked_amount_from_shares_balance_rounded_down(staker.shares)
                    });
                self.internal_save_staker(&staker_id, &staker);

                return PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence,
                    new_total_staked_balance: staker_new_balance.into(),
                    withdrawal_certificate: None,
                }));
            }
            PromiseResult::Failed => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                staker.select_staking_pool = None;
                self.internal_use_staker_or_panic(&staker_id, |staker| {
                    staker.select_staking_pool = None
                });
                self.internal_use_staker_staking_pool_or_panic(&staker_id, |pool| pool.unlock());
                self.transfer_near(staker_id, stake_amount.0);
                emit_callback_failed_event();
                return PromiseOrValue::Value(None);
            }
        };
    }

    #[private]
    fn increase_stake_callback(
        &mut self,
        staker_id: AccountId,
        increase_amount: U128,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        log!("increase_stake_callback, gas: {:?}", env::prepaid_gas());
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in increase_stake_callback by value.")
                    .0;

                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let pool_id = staker.select_staking_pool.clone().unwrap();

                // let pool_id = &self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);

                let increase_shares = staking_pool.increase_stake(
                    &mut staker,
                    increase_amount.0,
                    new_total_staked_balance,
                );
                staking_pool.unlock();

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_staking_pool(&staking_pool);

                let sequence = U64(self.next_sequence());

                Event::StakerIncreaseStake {
                    staking_pool_info: &(&mut staking_pool).into(),
                    staker_info: &(&staker).into(),
                    increase_stake_amount: &increase_amount,
                    increase_shares: &increase_shares.into(),
                    sequence: &sequence,
                }
                .emit();

                return PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence,
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                    withdrawal_certificate: None,
                }));
            }
            PromiseResult::Failed => {
                self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                    staking_pool.unlock()
                });
                self.transfer_near(staker_id, increase_amount.0);
                emit_callback_failed_event();
                return PromiseOrValue::Value(None);
            }
        };
    }

    #[payable]
    #[private]
    fn stake_after_check_whitelisted(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
        #[callback] whitelisted: bool,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        if !whitelisted {
            log!("Failed to select pool, {} is not whitelisted.", pool_id);
            return PromiseOrValue::Value(None);
        }

        if !self.staking_pools.get(&pool_id).is_some() {
            self.internal_save_staking_pool(&StakingPool::new(pool_id.clone()));
            Event::SaveStakingPool { pool_id: &pool_id }.emit();
        }

        self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| staking_pool.lock());

        self.ping(Some(pool_id.clone()))
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_AFTER_PING))
                    .stake_after_ping(staker_id, pool_id.clone()),
            )
            .into()
    }

    #[private]
    fn submit_unstake_batch_callback(&mut self, pool_id: PoolId) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);
                Event::SubmitUnstakeBatch {
                    unstake_batch_id: &staking_pool.current_unstake_batch_id,
                }
                .emit();
                staking_pool.submit_unstake();
                staking_pool.unlock();
                self.internal_save_staking_pool(&staking_pool);
            }
            PromiseResult::Failed => {
                self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
            }
        }
    }

    #[private]
    fn withdraw_unstake_batch_callback(
        &mut self,
        pool_id: PoolId,
        unstake_batch_id: UnstakeBatchId,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);
                staking_pool.last_unstake_batch_id = None;
                staking_pool.withdraw_unstake_batch(&unstake_batch_id);
                staking_pool.unlock();
                self.internal_save_staking_pool(&staking_pool);

                Event::WithdrawUnstakeBatch {
                    unstake_batch_id: &unstake_batch_id,
                }
                .emit();
            }
            PromiseResult::Failed => {
                self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
            }
        }
    }

    #[private]
    fn ping_callback(&mut self, pool_id: PoolId) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let staked_balance: U128 = near_sdk::serde_json::from_slice(&value).unwrap();

                self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                    staking_pool.total_staked_balance = staked_balance.0;
                    staking_pool.unlock();
                });

                Event::Ping {
                    pool_id: &pool_id,
                    new_total_staked_balance: &staked_balance,
                }
                .emit();
            }
            PromiseResult::Failed => {
                self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
            }
        }
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_create_pending_withdrawal_in_staker(
        &mut self,
        staker: &Staker,
        beneficiary: AccountId,
        amount: Balance,
        pool_id: PoolId,
        allow_other_withdraw: bool,
        unstake_batch_id: UnstakeBatchId,
    ) -> PendingWithdrawal {
        let pending_withdrawal = PendingWithdrawal::new(
            self.next_uuid().into(),
            pool_id,
            amount,
            env::epoch_height() + NUM_EPOCHS_TO_UNLOCK,
            staker.get_unlock_time(),
            beneficiary,
            allow_other_withdraw,
            unstake_batch_id,
        );

        self.internal_use_account(&staker.staker_id, |account| {
            account.pending_withdrawals.insert(
                &pending_withdrawal.withdrawal_certificate,
                &pending_withdrawal,
            );
        });

        pending_withdrawal
    }

    pub(crate) fn internal_get_staker_selected_pool_or_panic(
        &self,
        account_id: &AccountId,
    ) -> PoolId {
        self.stakers
            .get(account_id)
            .and_then(|staker| staker.select_staking_pool.clone())
            .expect(format!("The staker({}) haven't select pool!", account_id).as_str())
    }
}
