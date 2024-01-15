use crate::{constants::STORAGE_FEE, contract_interface::owner::OwnerAction, *};

#[near_bindgen]
impl OwnerAction for RestakingBaseContract {
    #[payable]
    fn set_new_owner(&mut self, new_owner: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner = new_owner;
    }

    #[payable]
    fn set_cc_register_fee(&mut self, new_cc_register_fee: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.cc_register_fee = new_cc_register_fee.into();
    }

    #[payable]
    fn set_slash_guarantee(&mut self, new_slash_guarantee: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.slash_guarantee = new_slash_guarantee.into();
    }

    #[payable]
    fn set_contract_running(&mut self) {
        assert_one_yocto();
        self.assert_owner();
        self.is_contract_running = true;
    }

    #[payable]
    fn set_contract_pause(&mut self) {
        assert_one_yocto();
        self.assert_owner();
        self.is_contract_running = false;
    }

    #[payable]
    fn set_withdrawal_beneficiary(
        &mut self,
        account_id: AccountId,
        withdraw_certificate: WithdrawalCertificate,
        new_beneficiary: AccountId,
    ) {
        assert_one_yocto();
        self.assert_owner();
        self.internal_use_account(&account_id, |account| {
            let mut pending_withdrawal = account
                .pending_withdrawals
                .get(&withdraw_certificate)
                .unwrap();
            pending_withdrawal.beneficiary = new_beneficiary.clone();
            account
                .pending_withdrawals
                .insert(&withdraw_certificate, &pending_withdrawal);
        })
    }

    #[payable]
    fn set_staker_unbonding_unlock_time_as_current_time(&mut self, staker_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.internal_use_staker_or_panic(&staker_id, |staker| {
            staker.unbonding_unlock_time = env::block_timestamp();
        });
    }

    #[payable]
    fn set_staking_pool_unlock(&mut self, pool_id: PoolId) {
        assert_one_yocto();
        self.assert_owner();
        self.internal_use_staking_pool_or_panic(&pool_id, |pool| {
            pool.unlock();
        });
    }
}

impl RestakingBaseContract {
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "Predecessor should be owner!"
        );
    }

    pub(crate) fn assert_contract_is_running(&self) {
        assert!(self.is_contract_running, "The contract is pause.");
    }

    pub(crate) fn assert_attached_storage_fee(&self) {
        assert_eq!(
            env::attached_deposit(),
            STORAGE_FEE,
            "Should attach {} near as storage fee.",
            STORAGE_FEE
        );
    }
}
