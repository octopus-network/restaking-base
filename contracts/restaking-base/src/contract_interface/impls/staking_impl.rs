use crate::*;

#[near_bindgen]
impl StakerAction for RestakingBaseContract {
    #[payable]
    fn stake(&mut self, pool_id: PoolId) -> PromiseOrValue<Option<StakingChangeResult>> {
        assert_attached_near();

        let staker_id = env::predecessor_account_id();

        let staker = self
            .stakers
            .get(&staker_id)
            .unwrap_or(Staker::new(staker_id.clone()));

        log!(
            "{:?}, new pool id: {:?}",
            staker.select_staking_pool,
            pool_id
        );

        assert_eq!(staker.shares, 0, "Can't stake, shares is not zero");
        assert!(
            staker.select_staking_pool.is_none()
                || staker.select_staking_pool.clone().unwrap().ne(&pool_id),
            "Staker({}) have selected pool({})",
            staker_id,
            pool_id
        );

        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker_id);
        storage_manager
            .execute_in_storage_monitoring(|| self.internal_save_staker(&staker_id, &staker));
        self.internal_save_storage_manager(&staker_id, &storage_manager);

        return ext_whitelist::ext(self.staking_pool_whitelist_account.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_IS_WHITELISTED))
            .is_whitelisted(pool_id.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_SELECT_POOL_AFTER_CHECK_WHITELIST))
                    .stake_after_check_whitelisted(staker_id.clone(), pool_id.clone()),
            )
            // .then(
            //     // Self::ext(env::current_account_id())
            //     self.ping(Some(pool_id)).then(
            //         Self::ext(current_account_id())
            //             .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_INCREASE_STAKE_AFTER_PING))
            //             .increase_stake_after_ping(staker_id, env::attached_deposit().into()),
            //     ),
            // )
            .into();
    }

    fn ping(&mut self, pool_id: Option<PoolId>) -> Promise {
        log!("ping, pool_id: {:?}", pool_id);
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
        assert_attached_near();

        let staker_id = env::predecessor_account_id();

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
        log!("decrease_stake, gas: {:?}", env::prepaid_gas());
        assert_one_yocto();
        assert!(decrease_amount.0 > 0, "The decrease amount should gt 0");

        let staker_id = env::predecessor_account_id();

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_AFTER_PING))
                    .decrease_stake_after_ping(
                        staker_id,
                        Some(decrease_amount),
                        beneficiary.unwrap_or(env::predecessor_account_id()),
                    ),
            )
            .into();
    }

    #[payable]
    fn unstake(&mut self) -> PromiseOrValue<Option<StakingChangeResult>> {
        log!("unstake");
        assert_one_yocto();
        let staker_id = env::predecessor_account_id();
        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker_id);
        storage_manager.execute_in_storage_monitoring(|| {
            let mut staker = self.internal_get_staker_or_panic(&staker_id);
            let bonding_consumer_chain_ids = staker
                .bonding_consumer_chains
                .iter()
                .map(|e| e.0)
                .collect_vec();
            for bonding_consumer_chain_id in bonding_consumer_chain_ids {
                // consumer_chain.unbond(&staker.staker_id);
                // staker.unbond(&consumer_chain.consumer_chain_id);
                let mut consumer_chain =
                    self.internal_get_consumer_chain_or_panic(&bonding_consumer_chain_id);
                consumer_chain.unbond(&staker.staker_id);
                staker.unbond(&consumer_chain.consumer_chain_id);
                // self.internal_unbond(&mut staker, &mut consumer_chain);
                self.internal_save_consumer_chain(&bonding_consumer_chain_id, &consumer_chain);
            }
            self.internal_save_staker(&staker_id, &staker);
        });
        self.internal_save_storage_manager(&staker_id, &storage_manager);

        return self
            .ping(Option::None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_DECREASE_STAKE_AFTER_PING))
                    .decrease_stake_after_ping(staker_id.clone(), None, staker_id.clone()),
            )
            .into();
    }

    fn withdraw(
        &mut self,
        staker: AccountId,
        id: WithdrawalCertificatetId,
    ) -> PromiseOrValue<U128> {
        log!("withdraw");
        let account = self.internal_get_account_or_panic(&staker);
        let pending_withdrawal = account.pending_withdrawals.get(&id).unwrap();

        ext_staking_pool::ext(pending_withdrawal.pool_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_WITHDRAW))
            .withdraw(pending_withdrawal.amount.into())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_SINGLE_WITHDRAW_CALLBACK))
                    .withdraw_callback(staker, id),
            )
            .into()
    }
}

#[near_bindgen]
impl StakeView for RestakingBaseContract {
    fn get_staker(&self, staker_id: StakerId) -> Option<StakerView> {
        self.stakers.get(&staker_id).map(|e| e.into())
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
                        .map(|chain_id| self.consumer_chains.get(&chain_id.0).unwrap())
                        .map(ConsumerChainView::from)
                        .collect(),
                )
            })
            .unwrap_or(vec![])
    }

    fn get_staking_pool(&self, pool_id: PoolId) -> StakingPool {
        self.internal_get_staking_pool_or_panic(&pool_id)
    }

    fn get_staking_pools(&self) -> Vec<StakingPool> {
        self.staking_pools.values().collect_vec()
    }

    fn get_account_staked_balance(&self, account_id: AccountId) -> U128 {
        self.get_staker_staked_balance(&account_id).into()
    }
}

#[near_bindgen]
impl StakingCallBack for RestakingBaseContract {
    #[private]
    fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        withdrawal_certificate: WithdrawalCertificatetId,
    ) {
        log!("withdraw_callback");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut storage_manager = self.internal_get_storage_manager_or_panic(&account_id);
                let pending_withdrawal = storage_manager
                    .execute_in_storage_monitoring(|| {
                        self.internal_use_account(&account_id, |account| {
                            account.pending_withdrawals.remove(&withdrawal_certificate)
                        })
                    })
                    .unwrap();
                self.internal_save_storage_manager(&account_id, &storage_manager);
                self.transfer_near(pending_withdrawal.beneficiary, pending_withdrawal.amount);
            }
            PromiseResult::Failed => {
                log!("{:?}", env::promise_result(0));
            }
        }
    }

    #[private]
    fn decrease_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        decrease_amount: Option<U128>,
        beneficiary: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                panic!("Failed to decrease_stake because ping failed.")
            }
            PromiseResult::Successful(_) => {
                //todo refactor
                let pool_id: AccountId =
                    self.internal_get_staker_selected_pool_or_panic(&staker_id);

                // todo add decrease check, should not lt min stake request
                let unwraped_decrease_amount = decrease_amount.unwrap_or_else(|| {
                    let staker = self.internal_get_staker_or_panic(&staker_id);
                    self.internal_use_staking_pool_or_panic(&pool_id, |pool| {
                        pool.staked_amount_from_shares_balance_rounded_down(staker.shares)
                            .into()
                    })
                });

                let (decrease_shares, receive_amount) =
                    self.internal_decrease_stake(&staker_id, unwraped_decrease_amount.0);

                ext_staking_pool::ext(pool_id)
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
    fn decrease_stake_callback(
        &mut self,
        staker_id: AccountId,
        decrease_shares: U128,
        decrease_amount: U128,
        beneficiary: AccountId,
        slash_governance: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in decrease_stake_callback by value.")
                    .0;
                let selected_pool_id = self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&selected_pool_id);
                staking_pool.total_staked_shares -= decrease_shares.0;
                staking_pool.total_staked_balance = total_staked_balance;
                self.internal_save_staking_pool(&staking_pool);

                let mut staker = self.internal_get_staker_or_panic(&staker_id);

                let withdraw_certificate = self.internal_create_pending_withdrawal_in_staker(
                    &mut staker,
                    beneficiary,
                    decrease_amount.0,
                    staking_pool.pool_id.clone(),
                );
                PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence: self.next_sequence().into(),
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                }))
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

    // #[private]
    // fn stake_after_selected_pool(
    //     &mut self,
    //     pool_id: PoolId,
    //     staker_id: AccountId,
    //     stake_amount: U128,
    // )-> PromiseOrValue<Option<StakingChangeResult>>{
    //     match env::promise_result(0) {
    //         PromiseResult::NotReady => unreachable!(),
    //         PromiseResult::Successful(_) => {
    //             self.ping(pool_id)
    //         }
    //         PromiseResult::Failed => {
    //             log!("Failed to stake by select pool error.");
    //             self.transfer_near(staker_id, increase_amount.0);
    //             return PromiseOrValue::Value(None);
    //         }
    //     }
    // }

    #[payable]
    #[private]
    fn stake_after_ping(
        &mut self,
        staker_id: AccountId,
    )-> PromiseOrValue<Option<StakingChangeResult>>{
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
                return PromiseOrValue::Value(None);
            }
        }
    }

    #[private]
    fn stake_callback(
        &mut self,
        staker_id: AccountId,
        stake_amount: U128
    )-> PromiseOrValue<Option<StakingChangeResult>> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);

                let pool_id = &self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);

                let increase_shares = staking_pool.calculate_increase_shares(stake_amount.0);

                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in increase_stake_callback by value.")
                    .0;

                staking_pool.increase_stake(&mut staker, increase_shares, new_total_staked_balance);

                log!(
                    " total balance: {}, total_shares: {}, staker.shares: {}, staker balance: {}",
                    new_total_staked_balance,
                    staking_pool.total_staked_shares,
                    staker.shares,
                    staking_pool.staked_amount_from_shares_balance_rounded_down(staker.shares)
                );

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_staking_pool(&staking_pool);

                return PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence: self.next_sequence().into(),
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                }));
            }
            PromiseResult::Failed => {
                self.transfer_near(staker_id, stake_amount.0);
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
                let mut staker = self.internal_get_staker_or_panic(&staker_id);

                let pool_id = &self.internal_get_staker_selected_pool_or_panic(&staker_id);
                let mut staking_pool = self.internal_get_staking_pool_or_panic(&pool_id);

                let increase_shares = staking_pool.calculate_increase_shares(increase_amount.0);

                let new_total_staked_balance = near_sdk::serde_json::from_slice::<U128>(&value)
                    .expect("Failed to deserialize in increase_stake_callback by value.")
                    .0;

                staking_pool.increase_stake(&mut staker, increase_shares, new_total_staked_balance);

                log!(
                    " total balance: {}, total_shares: {}, staker.shares: {}, staker balance: {}",
                    new_total_staked_balance,
                    staking_pool.total_staked_shares,
                    staker.shares,
                    staking_pool.staked_amount_from_shares_balance_rounded_down(staker.shares)
                );

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_staking_pool(&staking_pool);

                return PromiseOrValue::Value(Some(StakingChangeResult {
                    sequence: self.next_sequence().into(),
                    new_total_staked_balance: staking_pool
                        .staked_amount_from_shares_balance_rounded_down(staker.shares)
                        .into(),
                }));
            }
            PromiseResult::Failed => {
                self.transfer_near(staker_id, increase_amount.0);
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
            self.internal_save_staking_pool(
                &StakingPool::new(pool_id.clone(), staker_id.clone()),
            );
        }

        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker_id);
        storage_manager.execute_in_storage_monitoring(|| {
            self.internal_use_staker_or_panic(&staker_id, |staker| {
                staker.select_staking_pool = Some(pool_id.clone());
                log!("{} select pool{}.", staker_id, pool_id);
            });
        });
        self.internal_save_storage_manager(&staker_id, &storage_manager);

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
        log!("ping call back");
        self.internal_use_staking_pool_or_panic(&pool_id, |pool| {
            pool.total_staked_balance = staked_balance.0;
        });
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_create_pending_withdrawal_in_staker(
        &mut self,
        staker: &Staker,
        beneficiary: AccountId,
        amount: Balance,
        pool_id: PoolId,
    ) -> WithdrawalCertificatetId {
        let pending_withdrawal = PendingWithdrawal::new(
            self.next_uuid().into(),
            pool_id,
            amount,
            env::epoch_height() + NUM_EPOCHS_TO_UNLOCK,
            env::block_timestamp() + self.get_staker_unlock_time(&staker.staker_id.clone()),
            beneficiary,
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
        pool_id: &PoolId,
        decrease_share: ShareBalance,
    ) {
        self.internal_use_staker_or_panic(staker_id, |staker| {
            staker.shares += decrease_share;
        });
    }

    // todo delete
    pub(crate) fn internal_increase_stake(
        &mut self,
        staking_pool: &StakingPool,
        increase_stake_amount: Balance,
    ) -> ShareBalance {
        assert!(
            increase_stake_amount > 0,
            "Staking amount should be positive"
        );

        let increase_shares =
            staking_pool.share_balance_from_staked_amount_rounded_down(increase_stake_amount);
        assert!(
            increase_shares > 0,
            "The calculated number of increase stake shares should be positive, increase amount: {}, total_staked_shares: {}, total_staked_balance: {}",
            increase_stake_amount,
            staking_pool.total_staked_shares,staking_pool.total_staked_balance
        );

        // The amount of tokens the account will be charged from the unstaked balance.
        // Rounded down to avoid overcharging the account to guarantee that the account can always
        // unstake at least the same amount as staked.
        let charge_amount =
            staking_pool.staked_amount_from_shares_balance_rounded_down(increase_shares);
        assert!(
            charge_amount > 0 && increase_stake_amount >= charge_amount,
            "Invariant violation. Calculated staked amount must be positive, because \"stake\" share price should be at least 1.Increase_stake_amount is {},charge_amount is {})",
            increase_stake_amount,
            charge_amount
        );

        log!(
            "stake {} increase shares {}.",
            increase_stake_amount,
            increase_shares
        );

        increase_shares
    }
}
