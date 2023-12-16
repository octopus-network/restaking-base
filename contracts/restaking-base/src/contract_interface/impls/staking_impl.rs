use crate::{types::Sequence, *};

#[near_bindgen]
impl StakerAction for RestakingBaseContract {
    #[payable]
    fn stake(&mut self, pool_id: PoolId) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        assert_attached_near();

        let staker_id = env::predecessor_account_id();

        let staker = self
            .stakers
            .get(&staker_id)
            .unwrap_or(Staker::new(staker_id.clone()));

        assert_eq!(staker.shares, 0, "Can't stake, shares is not zero");
        assert!(
            staker.select_staking_pool.is_none()
                || staker.select_staking_pool.clone().unwrap().ne(&pool_id),
            "Staker({}) have selected pool({})",
            staker_id,
            pool_id
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
        withdraw_by_anyone: Option<bool>,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        self.assert_contract_is_running();
        self.assert_attached_storage_fee();
        let staker_id = env::predecessor_account_id();
        let mut staker = self.internal_get_staker_or_panic(&staker_id);
        let bonding_consumer_chain_ids = staker
            .bonding_consumer_chains
            .iter()
            .map(|e| e.0)
            .collect_vec();
        for bonding_consumer_chain_id in bonding_consumer_chain_ids {
            let mut consumer_chain =
                self.internal_get_consumer_chain_or_panic(&bonding_consumer_chain_id);
            consumer_chain.unbond(&staker.staker_id);
            staker.unbond(&consumer_chain.consumer_chain_id);
            self.internal_save_consumer_chain(&bonding_consumer_chain_id, &consumer_chain);
        }
        self.internal_save_staker(&staker_id, &staker);

        self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
            staking_pool.lock()
        });

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_AFTER_PING))
                    .unstake_after_ping(
                        staker_id.clone(),
                        staker_id.clone(),
                        withdraw_by_anyone.unwrap_or(true),
                    ),
            )
            .into();
    }

    fn withdraw(&mut self, staker: AccountId, id: WithdrawalCertificate) -> PromiseOrValue<U128> {
        self.assert_contract_is_running();
        let pending_withdrawal = self.internal_use_account(&staker, |account| {
            account.pending_withdrawals.remove(&id).unwrap()
        });
        assert!(
            pending_withdrawal.is_withdrawable(),
            "unlock timestamp:{}, unlock epoch:{}, current timestamp:{}, current epoch: {} ",
            pending_withdrawal.unlock_time,
            pending_withdrawal.unlock_epoch,
            env::block_timestamp(),
            env::epoch_height()
        );
        assert!(
            pending_withdrawal.allow_other_withdraw
                || env::predecessor_account_id().eq(&pending_withdrawal.beneficiary)
        );

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

    fn get_staking_pool(&self, pool_id: PoolId) -> StakingPoolInfo {
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

    fn is_withdrawable(&self, staker_id: StakerId, certificate: WithdrawalCertificate) -> bool {
        self.internal_get_account_or_panic(&staker_id)
            .pending_withdrawals
            .get(&certificate)
            .unwrap()
            .is_withdrawable()
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
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
            PromiseResult::Successful(_) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let staking_pool = self.internal_get_staking_pool_by_staker_or_panic(&staker_id);

                let decrease_shares = staker.shares;
                let receive_amount =
                    staking_pool.staked_amount_from_shares_balance_rounded_up(decrease_shares);
                staker.shares = 0;
                self.internal_save_staker(&staker_id, &staker);

                ext_staking_pool::ext(staking_pool.pool_id.clone())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
                    .unstake(receive_amount.into())
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
                            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_CALL_BACK))
                            .unstake_callback(
                                staker_id,
                                decrease_shares.into(),
                                receive_amount.into(),
                                beneficiary,
                                withdraw_by_anyone,
                            ),
                    )
                    .into()
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
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
            PromiseResult::Successful(_) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let staking_pool = self.internal_get_staking_pool_by_staker_or_panic(&staker_id);

                let decrease_shares = staking_pool.calculate_decrease_shares(decrease_amount.0);
                let receive_amount =
                    staking_pool.staked_amount_from_shares_balance_rounded_up(decrease_shares);
                staker.shares = staker
                    .shares
                    .checked_sub(decrease_shares)
                    .expect("Failed decrease shares in staker.");

                self.internal_save_staker(&staker_id, &staker);

                ext_staking_pool::ext(staking_pool.pool_id.clone())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
                    .unstake(receive_amount.into())
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
                            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_CALL_BACK))
                            .decrease_stake_callback(
                                staker_id,
                                decrease_shares.into(),
                                receive_amount.into(),
                                beneficiary,
                                None,
                            ),
                    )
                    .into()
            }
        }
    }

    #[private]
    fn unstake_callback(
        &mut self,
        staker_id: AccountId,
        share_balance: U128,
        receive_amount: U128,
        beneficiary: AccountId,
        withdraw_by_anyone: bool,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in decrease_stake_callback by value.")
                    .0;

                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&selected_pool_id);
                staking_pool.total_staked_balance = new_total_staked_balance;
                staking_pool.unstake(&staker_id, share_balance.0, new_total_staked_balance);

                staking_pool.unlock();

                let pending_withdrawal = self.internal_create_pending_withdrawal_in_staker(
                    &mut staker,
                    beneficiary,
                    receive_amount.0,
                    staking_pool.pool_id.clone(),
                    withdraw_by_anyone,
                );
                self.internal_save_staking_pool(&staking_pool);
                // self.internal_save_staker(&staker_id, &staker);
                self.stakers.remove(&staker_id);

                let sequence = U64(self.next_sequence());

                Event::StakerUnstake {
                    staking_pool_info: &(&mut staking_pool).into(),
                    staker_info: &(&staker).into(),
                    decrease_stake_amount: &receive_amount,
                    decrease_shares: &share_balance,
                    pending_withdrawal: &pending_withdrawal,
                    sequence: &sequence,
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
            PromiseResult::Failed => {
                self.internal_decrease_stake_rollback(&staker_id, share_balance.0);

                self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                    staking_pool.unlock()
                });
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
        }
    }

    #[private]
    fn decrease_stake_callback(
        &mut self,
        staker_id: AccountId,
        decrease_shares: U128,
        decrease_amount: U128,
        beneficiary: AccountId,
        slash_treasury: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in decrease_stake_callback by value.")
                    .0;
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&selected_pool_id);

                staking_pool.total_staked_balance = new_total_staked_balance;
                staking_pool.decrease_stake(decrease_shares.0, new_total_staked_balance);
                staking_pool.unlock();

                let pending_withdrawal = self.internal_create_pending_withdrawal_in_staker(
                    &mut staker,
                    beneficiary,
                    decrease_amount.0,
                    staking_pool.pool_id.clone(),
                    true,
                );
                self.internal_save_staking_pool(&staking_pool);
                self.internal_save_staker(&staker_id, &staker);

                let sequence = U64::from(self.next_sequence());
                Event::StakerDecreaseStake {
                    staking_pool_info: &(&mut staking_pool).into(),
                    staker_info: &(&staker).into(),
                    decrease_stake_amount: &decrease_amount,
                    decrease_shares: &decrease_shares,
                    pending_withdrawal: &pending_withdrawal,
                    sequence: &sequence,
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
            PromiseResult::Failed => {
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);

                match slash_treasury {
                    Some(treasury_account_id) => {
                        let mut treasury_account =
                            self.internal_get_account_or_new(&treasury_account_id);
                        treasury_account
                            .save_legacy_shares(decrease_shares.0, selected_pool_id.clone());
                        self.internal_save_account(&treasury_account_id, &treasury_account);
                    }
                    None => {
                        self.internal_decrease_stake_rollback(&staker_id, decrease_shares.0);
                    }
                }
                self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                    staking_pool.unlock();
                });
                emit_callback_failed_event();
                PromiseOrValue::Value(None)
            }
        }
    }

    #[payable]
    #[private]
    fn stake_after_ping(
        &mut self,
        staker_id: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let pool_id =
                    self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
                        staking_pool.lock();
                        staking_pool.pool_id.clone()
                    });

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
                            .stake_callback(staker_id, env::attached_deposit().into()),
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
                    self.internal_use_staker_staking_pool_or_panic(&staker_id, |staking_pool| {
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
            self.internal_save_staking_pool(&StakingPool::new(pool_id.clone(), staker_id.clone()));
            Event::SaveStakingPool { pool_id: &pool_id }.emit();
        }

        self.internal_use_staker_or_panic(&staker_id, |staker| {
            staker.select_staking_pool = Some(pool_id.clone());
        });

        self.ping(Some(pool_id))
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_AFTER_PING))
                    .stake_after_ping(staker_id),
            )
            .into()

        // return PromiseOrValue::Value(true);
    }

    #[private]
    fn ping_callback(&mut self, pool_id: PoolId, #[callback] staked_balance: U128) {
        self.internal_use_staking_pool_or_panic(&pool_id, |staking_pool| {
            staking_pool.total_staked_balance = staked_balance.0;
        });

        Event::Ping {
            pool_id: &pool_id,
            new_total_staked_balance: &staked_balance,
        }
        .emit();
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
    ) -> PendingWithdrawal {
        let pending_withdrawal = PendingWithdrawal::new(
            self.next_uuid().into(),
            pool_id,
            amount,
            env::epoch_height() + NUM_EPOCHS_TO_UNLOCK,
            staker.get_unlock_time(),
            beneficiary,
            allow_other_withdraw,
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

    pub(crate) fn internal_decrease_stake(
        &mut self,
        staker_id: &StakerId,
        decrease_amount: Balance,
    ) -> (ShareBalance, Balance) {
        assert!(
            decrease_amount > 0,
            "Decrease stake amount should be positive"
        );
        let mut staker = self.internal_get_staker_or_panic(staker_id);
        let pool_id = &self.internal_get_staker_selected_pool_or_panic(staker_id);
        let staking_pool = self.internal_get_staking_pool_or_panic(pool_id);

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

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let receive_amount = staking_pool.staked_amount_from_shares_balance_rounded_up(num_shares);
        assert!(
            receive_amount > 0,
            "Invariant violation. Calculated staked amount must be positive, because stake share price should be at least 1"
        );

        staker.shares -= num_shares;

        self.internal_save_staker(staker_id, &staker);

        (num_shares, receive_amount)
    }

    pub(crate) fn internal_decrease_stake_rollback(
        &mut self,
        staker_id: &StakerId,
        decrease_share: ShareBalance,
    ) {
        self.internal_use_staker_or_panic(staker_id, |staker| {
            staker.shares += decrease_share;
        });
    }
}
