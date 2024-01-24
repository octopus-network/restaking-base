use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use crate::{constants::REGISTER_STORAGE_FEE, *};

#[near_bindgen]
impl StorageManagement for RestakingBaseContract {
    /// Only support register account, if account exist, the deposit near will be refund
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        #[allow(unused)] registration_only: Option<bool>,
    ) -> StorageBalance {
        let account_id = account_id.unwrap_or(env::predecessor_account_id());
        let exist = self.accounts.contains_key(&account_id);
        if exist {
            self.transfer_near(env::predecessor_account_id(), env::attached_deposit())
        } else {
            assert!(env::attached_deposit() >= REGISTER_STORAGE_FEE);
            self.internal_save_account(&account_id, &Account::new(account_id.clone()));
            if env::attached_deposit() > REGISTER_STORAGE_FEE {
                self.transfer_near(
                    env::predecessor_account_id(),
                    env::attached_deposit() - REGISTER_STORAGE_FEE,
                )
            }
        }

        self.storage_balance_of(account_id).unwrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, #[allow(unused)] amount: Option<U128>) -> StorageBalance {
        unreachable!()
    }

    #[payable]
    fn storage_unregister(&mut self, #[allow(unused)] force: Option<bool>) -> bool {
        return false;
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        return StorageBalanceBounds {
            min: REGISTER_STORAGE_FEE.into(),
            max: Option::None,
        };
    }

    fn storage_balance_of(&self, #[allow(unused)] account_id: AccountId) -> Option<StorageBalance> {
        Some(StorageBalance {
            total: REGISTER_STORAGE_FEE.into(),
            available: 0.into(),
        })
    }
}
