use crate::{
    models::{consumer_chain::ConsumerChainView, staker::{StakerView}},
    types::{WithdrawalCertificatetId, Sequence},
};

use super::*;

pub trait StakerAction {
    fn ping(&mut self, pool_id: Option<PoolId>) -> Promise;
    fn stake(&mut self, pool_id: PoolId) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn increase_stake(&mut self) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn decrease_stake(
        &mut self,
        decrease_amount: U128,
        beneficiary: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn unstake(&mut self) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn withdraw(&mut self, staker: AccountId, id: WithdrawalCertificatetId)->PromiseOrValue<U128>;
}

pub trait StakeView {
    fn get_staker(&self, staker_id: StakerId) -> Option<StakerView>;

    fn get_staker_bonding_consumer_chains(
        &self,
        staker_id: StakerId,
        skip: u32,
        limit: u32,
    ) -> Vec<ConsumerChainView>;

    fn get_staking_pool(&self, pool_id: PoolId) -> StakingPool;

    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;
}

pub trait StakingCallBack {
    // fn select_pool_callback(&mut self, staker_id: AccountId, pool_id: PoolId, whitelisted: bool)->PromiseOrValue<bool>;
    fn select_pool_after_check_whitelisted(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
        whitelisted: bool,
    ) -> PromiseOrValue<bool>;

    fn increase_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        increase_amount: U128,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn increase_stake_callback(
        &mut self,
        staker_id: AccountId,
        increase_amount: U128,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn decrease_stake_callback(
        &mut self,
        staker_id: AccountId,
        share_balance: U128,
        decrease_amount: U128,
        beneficiary: AccountId,
        slash_governance: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn decrease_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        decrease_amount: Option<U128>,
        beneficiary: AccountId
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        withdrawal_certificates: Vec<WithdrawalCertificatetId>,
    );

    fn ping_callback(&mut self, pool_id: PoolId, staked_balance: U128);
}
