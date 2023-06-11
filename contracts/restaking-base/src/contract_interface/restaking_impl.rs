use std::cmp::max;
use std::cmp::min;
use std::ops::Mul;

use super::restaking::*;
use super::staking::*;
use super::*;
use crate::constants::gas_constants::TGAS_FOR_CHANGE_KEY;
use crate::constants::gas_constants::TGAS_FOR_GET_ACCOUNT_STAKED_BALANCE;
use crate::constants::gas_constants::TGAS_FOR_UNSTAKE;
use crate::models::consumer_chain::ConsumerChainStatus;
use crate::types::ValidaotrSet;
use crate::{
    external::consumer_chain_pos::ext_consumer_chain_pos,
    external::staking_pool::*,
    models::consumer_chain::{ConsumerChainRegisterParam, ConsumerChainUpdateParam},
};
use itertools::Itertools;
use near_sdk::serde_json::json;
use near_sdk::Gas;
use near_sdk::ONE_YOCTO;
use types::SlashId;

impl RestakingBaseContract {}

#[near_bindgen]
impl ConsumerChainAction for RestakingBaseContract {
    #[payable]
    fn blackout(&mut self, consumer_chain_id: ConsumerChainId, staker_id: StakerId) {
        self.internal_use_consumer_chain_or_panic(&consumer_chain_id, |consumer_chain| {
            consumer_chain.assert_cc_pos_account();
            consumer_chain.blacklist.insert(&staker_id);
        });
    }

    #[payable]
    fn slash_request(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        slash_items: Vec<(AccountId, Balance)>,
        evidence_sha256_hash: String,
    ) -> SlashId {
        assert_eq!(
            env::attached_deposit(),
            self.slash_guarantee,
            "The attached near({}) not equal slash guarantee.({})",
            env::attached_deposit(),
            self.slash_guarantee
        );
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        consumer_chain.assert_cc_pos_account();

        let slash_id = U64(self.next_uuid());

        // needn't check storage, the slash guarantee should able to cover storage.
        self.slashes.insert(
            &slash_id,
            &Slash {
                consumer_chain_id,
                slash_items,
                evidence_sha256_hash,
            },
        );
        slash_id
    }
}

#[near_bindgen]
impl GovernanceAction for RestakingBaseContract {
    // todo CC怎么指定自己Governance
    #[payable]
    fn register_consumer_chain(&mut self, register_param: ConsumerChainRegisterParam) {
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

        // todo how to check chain id is legal
        // 按照ccv的规则来

        let consumer_chain =
            ConsumerChain::new_from_register_param(register_param, env::current_account_id());

        // needn't check storage, the register fee should able to cover storage.

        self.consumer_chains
            .insert(&consumer_chain.consumer_chain_id, &consumer_chain);
    }

    #[payable]
    fn deregister_consumer_chain(&mut self, consumer_chain_id: ConsumerChainId) {
        // todo how to clean consumer chain state?

        assert_one_yocto();
        self.internal_use_consumer_chain_or_panic(&consumer_chain_id, |consumer_chain| {
            consumer_chain.assert_cc_gov();
            consumer_chain.status = ConsumerChainStatus::Unregistered;
        });
    }

    #[payable]
    fn update_consumer_chain_info(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        update_param: ConsumerChainUpdateParam,
    ) {
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

        consumer_chain.update(update_param);
        self.consumer_chains
            .insert(&consumer_chain_id, &consumer_chain);
    }

    #[payable]
    fn slash(&mut self, consumer_chain_id: ConsumerChainId, slash_id: SlashId, is_approve: bool) {
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
            for slash_item in &slash.slash_items {
                let actual_execution_receive_amount =
                    self.internal_slash(&slash_item.0, slash_item.1, &consumer_chain.governance);
                // todo log
            }
        }

        self.internal_remove_slash(&slash_id);
    }
}

#[near_bindgen]
impl StakerRestakingAction for RestakingBaseContract {
    #[payable]
    fn bond(&mut self, consumer_chain_id: ConsumerChainId, key: String) -> PromiseOrValue<bool> {
        assert_one_yocto();

        let staker_id = env::predecessor_account_id();
        let mut staker = self.internal_get_staker_or_panic(&staker_id);
        let mut consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);

        self.internal_bond(&mut staker, &mut consumer_chain);

        self.ping(Option::None)
            .then(
                ext_consumer_chain_pos::ext(consumer_chain.pos_account_id)
                    .bond(staker_id.clone(), key),
            )
            .then(Self::ext(env::current_account_id()).bond_callback(consumer_chain_id, staker_id))
            .into()
    }

    #[payable]
    fn change_key(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        new_key: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();

        // 1. check if bonding
        let staker = self.internal_get_staker_or_panic(&env::predecessor_account_id());
        let consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        assert!(
            staker.bonding_consumer_chains.contains(&consumer_chain_id),
            "Failed change_key, staker({}) didn't bond in consumer_chain({})",
            staker.staker_id,
            consumer_chain_id
        );

        ext_consumer_chain_pos::ext(consumer_chain.pos_account_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_CHANGE_KEY))
            .change_key(staker.staker_id, new_key)
            .into()
    }

    #[payable]
    fn unbond(&mut self, consumer_chain_id: ConsumerChainId) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let staker_id = env::predecessor_account_id();
        self.internal_unbond(&staker_id, &consumer_chain_id);
        PromiseOrValue::Value(true)
    }
}

#[near_bindgen]
impl ReStakingCallBack for RestakingBaseContract {
    #[private]
    fn bond_callback(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        staker_id: AccountId,
        #[callback] success: bool,
    ) -> PromiseOrValue<bool> {
        let mut consumer_chain = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);
        let mut staker = self.internal_get_staker_or_panic(&staker_id);
        if !success {
            self.internal_rollback_bond(&mut staker, &mut consumer_chain);
            return PromiseOrValue::Value(false);
        }

        PromiseOrValue::Value(true)
    }
}

#[near_bindgen]
impl ReStakingView for RestakingBaseContract {
    fn get_validator_set(&self, consumer_chain_id: ConsumerChainId, limit: u32) -> ValidaotrSet {
        let consumer_chains = self.internal_get_consumer_chain_or_panic(&consumer_chain_id);

        consumer_chains
            .bonding_stakers
            .iter()
            .map(|staker_id| {
                (
                    staker_id.clone(),
                    U128(self.get_staker_staked_balance(&staker_id)),
                )
            })
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .take(limit as usize)
            .collect_vec()
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_slash(
        &mut self,
        slash_staker_id: &StakerId,
        slash_amount: Balance,
        governance: &AccountId,
    ) -> Balance {
        let staker = self.internal_get_staker_or_panic(slash_staker_id);

        // 1. staker pending withdrawals
        let slashed_amount_from_pending_withdrawals =
            self.internal_slash_in_pending_withdrawals(slash_staker_id, slash_amount, governance);

        if slashed_amount_from_pending_withdrawals == slash_amount {
            return slash_amount;
        }

        let slashed_amount_from_staker_shares = if staker.shares != 0 {
            self.internal_slash_in_staker_shares(slash_staker_id, slash_amount, governance)
        } else {
            0
        };
        return slashed_amount_from_pending_withdrawals + slashed_amount_from_staker_shares;
    }

    pub(crate) fn internal_slash_in_pending_withdrawals(
        &mut self,
        slash_staker_id: &StakerId,
        slah_amount: Balance,
        governance: &AccountId,
    ) -> Balance {
        let staker_account = self.internal_get_account_or_panic(slash_staker_id);
        let mut governance_account = self.internal_get_account_or_panic(&governance);
        let mut pending_withdrawals = staker_account
            .pending_withdrawals
            .values()
            .sorted_by(|a, b| a.unlock_time.cmp(&b.unlock_time))
            .collect_vec();

        let mut acc_slash_amount = 0;
        for pending_withdrawal in &mut pending_withdrawals {
            if acc_slash_amount == slah_amount {
                break;
            }
            let new_pending_withdrawal = pending_withdrawal.slash(
                self.next_uuid().into(),
                governance.clone(),
                max(pending_withdrawal.amount, slah_amount - acc_slash_amount),
            );

            governance_account.pending_withdrawals.insert(
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
        slah_amount: Balance,
        governance: &AccountId,
    ) -> Balance {
        let pool_id = self.internal_get_staker_selected_pool_or_panic(slash_staker_id);
        let staker = self.internal_get_staker_or_panic(&slash_staker_id);
        let staking_pool = self.internal_get_staking_pool_by_staker_or_panic(&slash_staker_id);

        let slah_staker_total_balance =
            staking_pool.staked_amount_from_shares_balance_rounded_up(staker.shares);
        let actul_slash_amount = min(slah_staker_total_balance, slah_amount);

        let (decrease_shares, receive_amount) =
            self.internal_decrease_stake(&slash_staker_id, actul_slash_amount);

        // todo check storage for call successful

        ext_staking_pool::ext(pool_id)
            .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
            .with_attached_deposit(ONE_YOCTO)
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
                    .with_static_gas(Gas::ONE_TERA.mul(TGAS_FOR_UNSTAKE))
                    .decrease_stake_callback(
                        slash_staker_id.clone(),
                        decrease_shares.into(),
                        receive_amount.into(),
                        Some(governance.clone()),
                    ),
            );
        actul_slash_amount
    }

    pub(crate) fn internal_bond(
        &mut self,
        staker: &mut Staker,
        consumer_chain: &mut ConsumerChain,
    ) {
        // 1. check black list

        assert!(
            !consumer_chain.blacklist.contains(&staker.staker_id),
            "Failed to bond, {} has been blacklisted by {}",
            staker.staker_id,
            consumer_chain.consumer_chain_id
        );
        let mut staker = self.internal_get_staker_or_panic(&staker.staker_id);

        // let mut account = self.internal_get_account_or_panic(&staker.staker_id);

        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker.staker_id);

        storage_manager.execute_in_storage_monitoring(|| {
            self.internal_use_staker_or_panic(&staker.staker_id, |staker| {
                staker.bond(&consumer_chain.consumer_chain_id)
            });
            consumer_chain.bond(&staker.staker_id);
            staker.bond(&consumer_chain.consumer_chain_id);
        });

        self.internal_save_storage_manager(&staker.staker_id, &storage_manager);
    }

    pub(crate) fn internal_rollback_bond(
        &mut self,
        staker: &mut Staker,
        consumer_chain: &mut ConsumerChain,
    ) {
        let mut storage_manager = self.internal_get_storage_manager_or_panic(&staker.staker_id);
        storage_manager.execute_in_storage_monitoring(|| {
            consumer_chain.unbond(&staker.staker_id);
            staker.unbond(&consumer_chain.consumer_chain_id);
        });
        self.internal_save_storage_manager(&staker.staker_id, &storage_manager);
    }

    pub(crate) fn internal_unbond(
        &mut self,
        staker_id: &StakerId,
        consumer_chain_id: &ConsumerChainId,
    ) {
        let mut consumer_chain = self.internal_get_consumer_chain_or_panic(consumer_chain_id);
        let mut staker = self.internal_get_staker_or_panic(staker_id);

        let mut storage_manager = self.internal_get_storage_manager_or_panic(staker_id);
        storage_manager.execute_in_storage_monitoring(|| {
            consumer_chain.unbond(staker_id);
            staker.unbond(consumer_chain_id);
            staker.unbonding_unlock_time = max(
                staker.unbonding_unlock_time,
                env::block_timestamp() + consumer_chain.unbond_period.0,
            )
        });
        self.internal_save_storage_manager(staker_id, &storage_manager);
    }
}
