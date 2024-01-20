use near_sdk::{EpochHeight, Timestamp};

use crate::{types::WithdrawalCertificate, *};

#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PendingWithdrawal {
    pub withdrawal_certificate: WithdrawalCertificate,
    pub pool_id: PoolId,
    #[serde(with = "u128_dec_format")]
    pub amount: Balance,
    #[serde(with = "u64_dec_format")]
    pub unlock_epoch: EpochHeight,
    #[serde(with = "u64_dec_format")]
    pub unlock_time: Timestamp,
    pub beneficiary: AccountId,
    pub allow_other_withdraw: bool,
    pub unstake_batch_id: Option<UnstakeBatchId>,
}

impl PendingWithdrawal {
    pub fn new(
        withdrawal_certificate: WithdrawalCertificate,
        pool_id: PoolId,
        amount: Balance,
        unlock_epoch: EpochHeight,
        unlock_time: Timestamp,
        beneficiary: AccountId,
        allow_other_withdraw: bool,
        unstake_batch_id: UnstakeBatchId,
    ) -> PendingWithdrawal {
        Self {
            withdrawal_certificate,
            pool_id,
            amount,
            unlock_epoch,
            unlock_time,
            beneficiary,
            allow_other_withdraw,
            unstake_batch_id: Some(unstake_batch_id),
        }
    }

    pub fn is_withdrawable(&self) -> bool {
        return env::block_timestamp() >= self.unlock_time;
    }

    pub fn slash(
        &mut self,
        withdrawal_certificate: WithdrawalCertificate,
        amount: Balance,
        beneficiary: AccountId,
    ) -> Self {
        self.amount = self.amount
        .checked_sub(amount)
        .expect(format!("Failed to slash, the slash amount({}) is greater than PendingWithdrawal amount({})", amount, self.amount)
        .as_str());

        Self {
            withdrawal_certificate,
            pool_id: self.pool_id.clone(),
            amount: amount,
            unlock_epoch: self.unlock_epoch,
            unlock_time: env::block_timestamp(),
            beneficiary,
            allow_other_withdraw: true,
            unstake_batch_id: self.unstake_batch_id.clone(),
        }
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_is_withdrawable(
        &self,
        staking_pool: &StakingPool,
        pending_withdrawal: &PendingWithdrawal,
    ) -> bool {
        assert_eq!(pending_withdrawal.pool_id, staking_pool.pool_id);
        if let Some(unstake_batch_id) = pending_withdrawal.unstake_batch_id {
            pending_withdrawal.is_withdrawable()
                && staking_pool.is_unstake_batch_withdrawn(&unstake_batch_id)
        } else {
            pending_withdrawal.is_withdrawable() && staking_pool.is_withdrawable()
        }
    }
}
