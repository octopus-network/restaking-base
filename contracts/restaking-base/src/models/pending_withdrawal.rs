use near_sdk::{EpochHeight, Timestamp};

use crate::{types::WithdrawalCertificatetId, *};

#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PendingWithdrawal {
    pub withdrawal_certificate: WithdrawalCertificatetId,
    pub pool_id: PoolId,
    pub amount: Balance,
    pub unlock_epoch: EpochHeight,
    pub unlock_time: Timestamp,
    pub beneficiary: AccountId,
}

impl PendingWithdrawal {
    pub fn new(
        withdrawal_certificate: WithdrawalCertificatetId,
        pool_id: PoolId,
        amount: Balance,
        unlock_epoch: EpochHeight,
        unlock_time: Timestamp,
        beneficiary: AccountId,
    ) -> PendingWithdrawal {
        Self {
            withdrawal_certificate,
            pool_id,
            amount,
            unlock_epoch,
            unlock_time,
            beneficiary,
        }
    }

    pub fn is_withdrawable(&self) -> bool {
        return env::block_timestamp() >= self.unlock_time
            && env::epoch_height() >= self.unlock_epoch;
    }

    pub fn slash(
        &mut self,
        withdrawal_certificate: WithdrawalCertificatetId,
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
        }
    }
}
