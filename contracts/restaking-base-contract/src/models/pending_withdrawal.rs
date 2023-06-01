use near_sdk::{EpochHeight, Timestamp};

use crate::{types::WithdrawalReceiptId, *};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PendingWithdrawal {
    pub withdrawal_certificate: WithdrawalReceiptId,
    pub beneficiary: AccountId,
    pub pool_id: PoolId,
    pub amount: Balance,
    pub unlock_epoch: EpochHeight,
    pub unlock_time: Timestamp,
}

impl PendingWithdrawal {
    pub fn new(
        withdrawal_certificate: WithdrawalReceiptId,
        beneficiary: AccountId,
        pool_id: PoolId,
        amount: Balance,
        unlock_epoch: EpochHeight,
        unlock_time: Timestamp,
    ) -> PendingWithdrawal {
        Self {
            withdrawal_certificate,
            beneficiary,
            pool_id,
            amount,
            unlock_epoch,
            unlock_time,
        }
    }

    pub fn is_withdrawable(&self) -> bool {
        return env::block_timestamp() >= self.unlock_time
            && env::epoch_height() >= self.unlock_epoch;
    }

    pub fn slash(
        &mut self,
        withdrawal_certificate: WithdrawalReceiptId,
        slash_beneficiary: AccountId,
        amount: Balance,
    ) -> Self {
        self.amount = self.amount
        .checked_sub(amount)
        .expect(format!("Failed to slash, the slash amount({}) is greater than PendingWithdrawal amount({})", amount, self.amount)
        .as_str());

        Self {
            withdrawal_certificate,
            beneficiary: slash_beneficiary,
            pool_id: self.pool_id.clone(),
            amount: amount,
            unlock_epoch: self.unlock_epoch,
            unlock_time: env::block_timestamp(),
        }
    }
}
