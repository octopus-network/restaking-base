use crate::{contract_interface::view::RestakingView, types::ValidatorSetInSequence, *};

#[near_bindgen]
impl ConsumerChainAction for RestakingBaseContract {
    #[payable]
    fn blackout(&mut self, consumer_chain_id: ConsumerChainId, staker_id: StakerId) {
        self.assert_contract_is_running();
        self.internal_use_consumer_chain_or_panic(&consumer_chain_id, |consumer_chain| {
            consumer_chain.assert_cc_pos_account();
            consumer_chain.blacklist.insert(&staker_id);
        });
    }

    #[payable]
    fn slash_request(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        slash_items: Vec<(AccountId, U128)>,
        evidence_sha256_hash: String,
    ) -> SlashId {
        self.assert_contract_is_running();
        assert_eq!(
            env::attached_deposit(),
            self.slash_guarantee,
            "The attached near({}) not equal slash guarantee.({})",
            env::attached_deposit(),
            self.slash_guarantee
        );
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        consumer_chain.assert_cc_pos_account();

        for (staker_id, _) in &slash_items {
            let staker = self.internal_get_staker_or_panic(staker_id);
            assert!(
                staker.allow_slash(&consumer_chain_id),
                "Failed to slash {}.",
                staker_id
            );
        }

        let slash_id = U64(self.next_uuid());

        Event::RequestSlash {
            consumer_chain_id: &consumer_chain_id,
            slash_items: &near_sdk::serde_json::to_string(&slash_items).unwrap(),
            evidence_sha256_hash: &evidence_sha256_hash,
        }
        .emit();

        // needn't check storage, the slash guarantee should able to cover storage.
        self.slashes.insert(
            &slash_id,
            &Slash {
                consumer_chain_id,
                slash_items,
                evidence_sha256_hash,
                slash_guarantee: self.slash_guarantee.into(),
            },
        );

        slash_id
    }
}

#[near_bindgen]
impl GovernanceAction for RestakingBaseContract {
    #[payable]
    fn register_consumer_chain(&mut self, register_param: ConsumerChainRegisterParam) {
        self.assert_contract_is_running();
        // check register_fee eq env::attached_deposit
        assert_eq!(
            env::attached_deposit(),
            self.cc_register_fee,
            "Attached near should be {}",
            self.cc_register_fee
        );
        // check chain id not used
        assert!(
            self.consumer_chains
                .get(&register_param.consumer_chain_id)
                .is_none(),
            "This ConsumerChainId {} has been registered.",
            register_param.consumer_chain_id
        );

        validate_chain_id(&register_param.consumer_chain_id);

        let consumer_chain = ConsumerChain::new_from_register_param(
            register_param.clone(),
            env::predecessor_account_id(),
            self.cc_register_fee,
        );

        // needn't check storage, the register fee should able to cover storage.
        self.consumer_chains
            .insert(&consumer_chain.consumer_chain_id, &consumer_chain);

        Event::RegisterConsumerChain {
            consumer_chain_info: &consumer_chain.into(),
            consumer_chain_register_param: &register_param,
        }
        .emit();
    }

    #[payable]
    fn deregister_consumer_chain(&mut self, consumer_chain_id: ConsumerChainId) {
        self.assert_contract_is_running();
        assert_one_yocto();

        let mut consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        consumer_chain.assert_cc_gov();
        consumer_chain.status = ConsumerChainStatus::Deregistered;
        self.internal_save_consumer_chain(&consumer_chain_id, &consumer_chain);
        Event::DeregisterConsumerChain {
            consumer_chain_info: &consumer_chain.into(),
        }
        .emit();
    }

    #[payable]
    fn update_consumer_chain_info(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        update_param: ConsumerChainUpdateParam,
    ) {
        self.assert_contract_is_running();
        assert_one_yocto();
        let mut consumer_chain = self.consumer_chains.get(&consumer_chain_id).expect(
            format!(
                "ConsumerChain not exist when update_consumer_chain_info by this chain_id: {}",
                consumer_chain_id
            )
            .as_str(),
        );
        // check if predecessor is consumer chain governance
        assert_eq!(
            consumer_chain.governance,
            env::predecessor_account_id(),
            "Only cc_gov({}) can update_consumer_chain_info",
            consumer_chain.governance
        );

        // Update unbonding period for every stakers.
        if let Some(new_unbonding_period) = update_param.unbonding_period {
            if new_unbonding_period != consumer_chain.unbonding_period {
                for staker_id in consumer_chain.bonding_stakers.iter() {
                    self.internal_use_staker_or_panic(&staker_id, |staker| {
                        staker.update_unbonding_period(&consumer_chain_id, new_unbonding_period)
                    });
                }
            }
        }

        consumer_chain.update(update_param.clone());

        self.consumer_chains
            .insert(&consumer_chain_id, &consumer_chain);

        Event::UpdateConsumerChain {
            consumer_chain_info: &consumer_chain.into(),
            consumer_chain_update_param: &update_param,
        }
        .emit();
    }

    #[allow(unused)]
    #[payable]
    fn slash(&mut self, consumer_chain_id: ConsumerChainId, slash_id: SlashId, is_approve: bool) {
        unreachable!();
        self.assert_contract_is_running();
        // todo if slash item too much, need finish slash by multi transaction.
        assert_one_yocto();

        // 1. check slash is belong to consumer_chain_id
        let slash = self.get_slash_or_panic(&slash_id);
        assert_eq!(
            slash.consumer_chain_id, consumer_chain_id,
            "The slash is not belong to {}.",
            consumer_chain_id
        );

        // 2. assert predecessor_account_id is cc gov
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        consumer_chain.assert_cc_gov();

        if is_approve {
            // 3. loop and slash
            let mut treasury_account = self.internal_get_account_or_new(&consumer_chain.treasury);
            for slash_item in &slash.slash_items {
                self.internal_slash(
                    &slash_item.0,
                    slash_item.1.into(),
                    &consumer_chain.treasury,
                    &mut treasury_account,
                );
            }
            self.internal_save_account(&consumer_chain.treasury, &treasury_account);
        }

        self.internal_remove_slash(&slash_id);
    }
}

#[near_bindgen]
impl StakerRestakingAction for RestakingBaseContract {
    #[payable]
    fn bond(&mut self, consumer_chain_id: ConsumerChainId, key: String) -> PromiseOrValue<bool> {
        self.assert_contract_is_running();
        self.assert_attached_storage_fee();

        let staker_id = env::predecessor_account_id();
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);

        consumer_chain.assert_chain_active();
        assert!(
            !consumer_chain.blacklist.contains(&staker_id),
            "Failed to bond, {} has been blacklisted by {}",
            staker_id,
            consumer_chain.consumer_chain_id
        );

        let staker = self.internal_get_staker_or_panic(&staker_id);

        assert!(
            staker.unbonding_unlock_time <= env::block_timestamp(),
            "Failed to bond by {}, the unbonding unlock time({}) should not greater then block time({}).",
            staker_id,
            staker.unbonding_unlock_time,
            env::block_timestamp()
        );

        self.ping(Option::None)
            .then(
                ext_consumer_chain_pos::ext(consumer_chain.pos_account_id)
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_CHANGE_KEY))
                    .bond(staker_id.clone(), key.clone()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_BOND_CALLBACK))
                    .bond_callback(consumer_chain_id, key, staker_id),
            )
            .into()
    }

    #[payable]
    fn change_key(&mut self, consumer_chain_id: ConsumerChainId, new_key: String) {
        self.assert_contract_is_running();
        assert_one_yocto();

        // 1. check if bonding
        let staker = self.internal_get_staker_or_panic(&env::predecessor_account_id());
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        assert!(
            staker
                .bonding_consumer_chains
                .get(&consumer_chain_id)
                .is_some(),
            "Failed change_key, staker({}) didn't bond in consumer_chain({})",
            staker.staker_id,
            consumer_chain_id
        );

        ext_consumer_chain_pos::ext(consumer_chain.pos_account_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_CHANGE_KEY))
            .change_key(staker.staker_id, new_key);
    }

    #[payable]
    fn unbond(&mut self, consumer_chain_id: ConsumerChainId) {
        self.assert_contract_is_running();
        assert_one_yocto();
        let staker_id = env::predecessor_account_id();
        self.internal_use_staker_or_panic(&staker_id, |staker| staker.unbond(&consumer_chain_id));
        self.internal_use_consumer_chain_or_panic(&consumer_chain_id, |consumer_chain| {
            consumer_chain.unbond(&staker_id)
        });
        Event::StakerUnbond {
            staker_id: &staker_id,
            consumer_chain_id: &consumer_chain_id,
        }
        .emit();
    }
}

#[near_bindgen]
impl RestakingCallback for RestakingBaseContract {
    #[private]
    fn bond_callback(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        key: String,
        staker_id: AccountId,
    ) -> PromiseOrValue<bool> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                emit_callback_failed_event();
                PromiseOrValue::Value(false)
            }
            PromiseResult::Successful(_) => {
                let mut staker = self.internal_get_staker_or_panic(&staker_id);
                let mut consumer_chain =
                    self.internal_get_consumer_chain_or_panic(&consumer_chain_id);

                staker.bond(&consumer_chain_id, consumer_chain.unbonding_period);
                consumer_chain.bond(&staker_id);

                self.internal_save_staker(&staker_id, &staker);
                self.internal_save_consumer_chain(&consumer_chain_id, &consumer_chain);

                Event::StakerBond {
                    staker_id: &staker_id,
                    consumer_chain_id: &consumer_chain_id,
                    key: &key,
                }
                .emit();
                PromiseOrValue::Value(true)
            }
        }
    }

    fn change_key_callback(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        new_key: String,
        staker_id: AccountId,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                emit_callback_failed_event();
            }
            PromiseResult::Successful(_) => {
                Event::StakerChangeKey {
                    staker_id: &staker_id,
                    consumer_chain_id: &consumer_chain_id,
                    new_key: &new_key,
                }
                .emit();
            }
        }
    }
}

#[near_bindgen]
impl RestakingView for RestakingBaseContract {
    fn get_consumer_chain(&self, consumer_chain_id: ConsumerChainId) -> Option<ConsumerChainInfo> {
        self.consumer_chains
            .get(&consumer_chain_id)
            .map(ConsumerChainInfo::from)
    }

    fn get_consumer_chains(&self) -> Vec<ConsumerChainInfo> {
        self.consumer_chains
            .values()
            .map(ConsumerChainInfo::from)
            .collect_vec()
    }

    fn get_validator_set(
        &self,
        consumer_chain_id: ConsumerChainId,
        limit: u32,
    ) -> ValidatorSetInSequence {
        let consumer_chains = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);

        let validator_set = consumer_chains
            .bonding_stakers
            .iter()
            .map(|staker_id| {
                (
                    staker_id.clone(),
                    U128(self.get_staker_staked_balance(&staker_id)),
                )
            })
            .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
            .take(limit as usize)
            .collect_vec();
        ValidatorSetInSequence {
            validator_set: validator_set,
            sequence: self.sequence.into(),
        }
    }

    fn get_slash_guarantee(&self) -> U128 {
        self.slash_guarantee.into()
    }

    fn get_cc_register_fee(&self) -> U128 {
        self.cc_register_fee.into()
    }

    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    fn is_contract_running(&self) -> bool {
        self.is_contract_running
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_slash(
        &mut self,
        slash_staker_id: &StakerId,
        slash_amount: Balance,
        treasury_id: &AccountId,
        treasury_account: &mut Account,
    ) -> Balance {
        let staker = self.internal_get_staker_or_panic(slash_staker_id);

        // 1. staker pending withdrawals
        let slashed_amount_from_pending_withdrawals = self.internal_slash_in_pending_withdrawals(
            slash_staker_id,
            slash_amount,
            treasury_id,
            treasury_account,
        );

        if slashed_amount_from_pending_withdrawals == slash_amount {
            return slash_amount;
        }

        let slashed_amount_from_staker_shares = if staker.shares != 0 {
            self.internal_slash_in_staker_shares(
                slash_staker_id,
                slash_amount - slashed_amount_from_pending_withdrawals,
                treasury_id,
                treasury_account,
            )
        } else {
            0
        };
        return slashed_amount_from_pending_withdrawals + slashed_amount_from_staker_shares;
    }

    pub(crate) fn internal_slash_in_pending_withdrawals(
        &mut self,
        slash_staker_id: &StakerId,
        slash_amount: Balance,
        treasury_id: &AccountId,
        treasury_account: &mut Account,
    ) -> Balance {
        let staker_account = self.internal_get_account_or_panic(slash_staker_id);
        // let mut treasury_account = self.internal_get_account_or_new(&treasury);
        let mut pending_withdrawals = staker_account
            .pending_withdrawals
            .values()
            .sorted_by(|a, b| a.unlock_time.cmp(&b.unlock_time))
            .collect_vec();

        let mut acc_slash_amount = 0;
        for pending_withdrawal in &mut pending_withdrawals {
            if acc_slash_amount == slash_amount {
                break;
            }
            let new_pending_withdrawal = pending_withdrawal.slash(
                self.next_uuid().into(),
                min(pending_withdrawal.amount, slash_amount - acc_slash_amount),
                treasury_id.clone(),
            );

            treasury_account.pending_withdrawals.insert(
                &new_pending_withdrawal.withdrawal_certificate,
                &new_pending_withdrawal,
            );
            acc_slash_amount += new_pending_withdrawal.amount;
        }
        acc_slash_amount
    }

    pub(crate) fn internal_slash_in_staker_shares(
        &mut self,
        slash_staker_id: &StakerId,
        slash_amount: Balance,
        treasury_id: &AccountId,
        treasury_account: &mut Account,
    ) -> Balance {
        let pool_id = self.internal_get_staker_selected_pool_or_panic(slash_staker_id);
        let mut staker = self.internal_get_staker_or_panic(&slash_staker_id);
        let mut staking_pool = self.internal_get_staking_pool_by_staker_or_panic(&slash_staker_id);

        // 1. Get staker staked balance
        let staker_total_staked_balance =
            staking_pool.staked_amount_from_shares_balance_rounded_down(staker.shares);

        // 2. Decrease min of staker_total_staked_balance and slash_amount
        let decrease_shares =
            staking_pool.calculate_decrease_shares(min(staker_total_staked_balance, slash_amount));

        // 3. decrease shares and actually receive amount
        let receive_amount =
            staking_pool.staked_amount_from_shares_balance_rounded_down(decrease_shares);

        staker.shares = staker
            .shares
            .checked_sub(decrease_shares)
            .expect("Failed decrease shares in staker.");

        staking_pool.decrease_stake(decrease_shares);
        let unstake_batch_id = staking_pool.batch_unstake(receive_amount);

        let pending_withdrawal = PendingWithdrawal::new(
            self.next_uuid().into(),
            pool_id,
            receive_amount,
            env::epoch_height() + NUM_EPOCHS_TO_UNLOCK,
            staker.get_unlock_time(),
            treasury_id.clone(),
            true,
            unstake_batch_id,
        );
        treasury_account.pending_withdrawals.insert(
            &pending_withdrawal.withdrawal_certificate,
            &pending_withdrawal,
        );

        self.internal_save_staking_pool(&staking_pool);
        self.internal_save_staker(&staker.staker_id, &staker);

        receive_amount
    }
}
