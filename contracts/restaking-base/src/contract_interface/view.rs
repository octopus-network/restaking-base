use crate::*;
pub trait StakeView {
    fn get_staker(&self, staker_id: StakerId) -> Option<StakerInfo>;

    fn get_pending_withdrawals(&self, account_id: AccountId) -> Vec<PendingWithdrawal>;

    fn get_staker_bonding_consumer_chains(
        &self,
        staker_id: StakerId,
        skip: u32,
        limit: u32,
    ) -> Vec<ConsumerChainInfo>;

    fn get_staking_pool(&self, pool_id: PoolId) -> StakingPoolInfo;

    fn get_staking_pools(&self) -> Vec<StakingPoolInfo>;

    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;

    fn get_current_sequence(&self) -> Sequence;

    fn is_withdrawable(&self, staker_id: StakerId, certificate: WithdrawalCertificate) -> bool;
}

pub trait RestakingView {
    fn get_consumer_chain(&self, consumer_chain_id: ConsumerChainId) -> Option<ConsumerChainInfo>;

    fn get_consumer_chains(&self) -> Vec<ConsumerChainInfo>;

    fn get_validator_set(
        &self,
        consumer_chain_id: ConsumerChainId,
        limit: u32,
    ) -> ValidatorSetInSequence;

    fn get_slash_guarantee(&self) -> U128;

    fn get_cc_register_fee(&self) -> U128;

    fn get_owner(&self) -> AccountId;
}
