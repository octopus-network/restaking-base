use crate::*;

pub trait StakerAction {
    fn ping(&mut self, pool_id: Option<PoolId>) -> Promise;
    fn stake(&mut self, pool_id: PoolId) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn increase_stake(&mut self) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn decrease_stake(
        &mut self,
        decrease_amount: U128,
        beneficiary: Option<AccountId>,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn unstake(
        &mut self,
        beneficiary: Option<AccountId>,
        withdraw_by_anyone: Option<bool>,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;
    fn withdraw_unstake_batch(&mut self, pool_id: PoolId, unstake_batch_id: UnstakeBatchId);
    fn submit_unstake_batch(&mut self, pool_id: PoolId);
    fn withdraw(&mut self, staker: AccountId, id: WithdrawalCertificate) -> PromiseOrValue<U128>;
}

pub trait StakingCallback {
    fn stake_after_check_whitelisted(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn stake_after_ping(
        &mut self,
        staker_id: AccountId,
        pool_id: PoolId,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn increase_stake_after_ping(
        &mut self,
        staker_id: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn stake_callback(
        &mut self,
        staker_id: AccountId,
        stake_amount: U128,
        pool_id: PoolId,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn increase_stake_callback(
        &mut self,
        staker_id: AccountId,
        increase_amount: U128,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn decrease_stake_after_ping(
        &mut self,
        staker_id: AccountId,
        decrease_amount: U128,
        beneficiary: AccountId,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn unstake_after_ping(
        &mut self,
        staker_id: AccountId,
        beneficiary: AccountId,
        withdraw_by_anyone: bool,
    ) -> PromiseOrValue<Option<StakingChangeResult>>;

    fn withdraw_callback(
        &mut self,
        account_id: AccountId,
        pending_withdrawal: PendingWithdrawal,
    ) -> PromiseOrValue<U128>;

    fn submit_unstake_batch_callback(&mut self, pool_id: PoolId);

    fn withdraw_unstake_batch_callback(
        &mut self,
        pool_id: PoolId,
        unstake_batch_id: UnstakeBatchId,
    );

    fn ping_callback(&mut self, pool_id: PoolId);
}
