use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use crate::{*, constants::storage_constants::PREPAY_STORAGE_USAGE};

#[near_bindgen]
impl StorageManagement for RestakingBaseContract {

    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        #[allow(unused)] registration_only: Option<bool>,
    ) -> StorageBalance {
        let attach_amount = env::attached_deposit();
        let account_id = account_id.unwrap_or(env::predecessor_account_id());

        let mut storage_manager = self.storage_managers
        .get(&account_id)
        .unwrap_or( StorageManager::new(attach_amount) );

        let usage_before_execute = env::storage_usage();

        let account = self.accounts.get(&account_id).unwrap_or(Account::new(account_id.clone()));
        self.internal_save_account(&account_id, &account);
        self.internal_save_storage_manager(&account_id, &storage_manager);
        let usage_after_execute = env::storage_usage();

        storage_manager.storage_usage += usage_after_execute - usage_before_execute;
        assert!(
            storage_manager.storage_cost() <= storage_manager.near_amount_for_storage,
            "The the storage cost({}) is great than storage deposit ({}).",
            storage_manager.storage_cost(),
            storage_manager.near_amount_for_storage
        );

        self.internal_save_storage_manager(&account_id, &storage_manager);
        self.storage_balance_of(account_id).unwrap()

    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut storage_manager = self.internal_get_storage_manager_or_panic(&account_id);
        let transfer_amount = storage_manager.withdraw_available_storage(amount);
        self.internal_save_storage_manager(&account_id, &storage_manager);
        if transfer_amount>0 {
            self.transfer_near(account_id, transfer_amount);
        }
        StorageBalance { total: storage_manager.near_amount_for_storage.into(), available: storage_manager.available_storage_deposit().into() }
    }

    #[payable]
    fn storage_unregister(&mut self, #[allow(unused)] force: Option<bool>) -> bool {
        return false
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
		return StorageBalanceBounds {
            min: U128(PREPAY_STORAGE_USAGE as u128 * env::storage_byte_cost()),
            max: Option::None,
        };
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.storage_managers.get(&account_id).map(|account| {
			StorageBalance { 
				total: account.near_amount_for_storage.into(), 
				available: account.available_storage_deposit().into() 
			}
		})
    }
}
