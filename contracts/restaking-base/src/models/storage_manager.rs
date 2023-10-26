use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StorageManager {
    /// The near amount for storage
    pub near_amount_for_storage: Balance,
    /// The used storage
    pub storage_usage: StorageUsage,
}

impl StorageManager {
    pub(crate) fn new(init_storage_balance: Balance) -> Self {
        StorageManager {
            near_amount_for_storage: init_storage_balance,
            storage_usage: 0,
        }
    }

    pub fn storage_usage(&self) -> u64 {
        self.storage_usage
    }

    pub fn storage_cost(&self) -> Balance {
        self.storage_usage() as u128 * env::storage_byte_cost()
    }

    pub fn available_storage_deposit(&self) -> Balance {
        if self.near_amount_for_storage > self.storage_cost() {
            self.near_amount_for_storage - self.storage_cost()
        } else {
            0
        }
    }

    pub fn withdraw_available_storage(&mut self, amount: Option<U128>) -> Balance {
        match amount {
            Some(withdraw_amount) => {
                let available_storage_deposit = self.available_storage_deposit();
                assert!(
                    available_storage_deposit>=withdraw_amount.0,
					"Failed to withdraw storage deposit, available_storage_deposit({}) is less than withdraw amount({}).",
					available_storage_deposit,
					withdraw_amount.0,
				);
                available_storage_deposit
            }
            None => {
                let available_storage_deposit = self.available_storage_deposit();
                self.near_amount_for_storage -= available_storage_deposit;
                available_storage_deposit
            }
        }
    }
}
