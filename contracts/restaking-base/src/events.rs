use crate::*;

pub const EVENT_STANDARD: &str = "restaking-base";
pub const EVENT_STANDARD_VERSION: &str = "1.1.0";

#[derive(Serialize)]
#[serde(
    crate = "near_sdk::serde",
    rename_all = "snake_case",
    tag = "event",
    content = "data"
)]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event<'a> {
    Ping {
        pool_id: &'a PoolId,
        new_total_staked_balance: &'a U128,
    },

    SaveStakingPool {
        pool_id: &'a AccountId,
    },

    StakerStake {
        staking_pool_info: &'a StakingPoolInfo,
        staker_info: &'a StakerInfo,
        select_pool: &'a PoolId,
        stake_amount: &'a U128,
        increase_shares: &'a U128,
        sequence: &'a U64,
    },

    StakerIncreaseStake {
        staking_pool_info: &'a StakingPoolInfo,
        staker_info: &'a StakerInfo,
        increase_stake_amount: &'a U128,
        increase_shares: &'a U128,
        sequence: &'a U64,
    },

    StakerDecreaseStake {
        staking_pool_info: &'a StakingPoolInfo,
        staker_info: &'a StakerInfo,
        decrease_stake_amount: &'a U128,
        decrease_shares: &'a U128,
        pending_withdrawal: &'a PendingWithdrawal,
        sequence: &'a U64,
        unstake_batch_id: &'a UnstakeBatchId,
    },

    StakerUnstake {
        staking_pool_info: &'a StakingPoolInfo,
        staker_info: &'a StakerInfo,
        decrease_stake_amount: &'a U128,
        decrease_shares: &'a U128,
        pending_withdrawal: &'a PendingWithdrawal,
        sequence: &'a U64,
        unstake_batch_id: &'a UnstakeBatchId,
    },

    StakerBond {
        staker_id: &'a StakerId,
        consumer_chain_id: &'a ConsumerChainId,
        key: &'a String,
    },

    StakerChangeKey {
        staker_id: &'a StakerId,
        consumer_chain_id: &'a ConsumerChainId,
        new_key: &'a String,
    },

    StakerUnbond {
        staker_id: &'a StakerId,
        consumer_chain_id: &'a ConsumerChainId,
    },

    RegisterConsumerChain {
        consumer_chain_info: &'a ConsumerChainInfo,
        consumer_chain_register_param: &'a ConsumerChainRegisterParam,
    },
    UpdateConsumerChain {
        consumer_chain_info: &'a ConsumerChainInfo,
        consumer_chain_update_param: &'a ConsumerChainUpdateParam,
    },
    DeregisterConsumerChain {
        consumer_chain_info: &'a ConsumerChainInfo,
    },
    RequestSlash {
        consumer_chain_id: &'a ConsumerChainId,
        slash_items: &'a String,
        evidence_sha256_hash: &'a String,
    },
    Withdraw {
        withdrawal_certificate: &'a WithdrawalCertificate,
    },
    CallbackWithFailed {
        current_account_id: &'a AccountId,
        predecessor_account_id: &'a AccountId,
    },
    WithdrawUnstakeBatch {
        unstake_batch_id: &'a UnstakeBatchId,
    },
    SubmitUnstakeBatch {
        unstake_batch_id: &'a UnstakeBatchId,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        let json = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": json["event"],
            "data": [json["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}
