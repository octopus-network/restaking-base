use near_contract_standards::storage_management::StorageManagement;

use super::owner::{ContractSettingView, OwnerAction};
use crate::*;

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
    fn mock_staker_bond(
        &mut self,
        staker_sum: u32,
        selected_pool_id: PoolId,
        bond_cc_id: ConsumerChainId,
    ) {
        let mut cc = self.internal_get_consumer_chain_or_panic(&bond_cc_id);

        for i in 0..staker_sum {
            let id = self.next_uuid();
            let staker_id =
                AccountId::new_unchecked(format!("mock-staker-{}.testnet", id).to_string());
            self.storage_deposit(Some(staker_id.clone()), None);
            let mut staker = Staker::new(staker_id.clone());
            staker.select_staking_pool = Some(selected_pool_id.clone());
            staker.shares = i as u128;
            staker.bond(&bond_cc_id, cc.unbond_period);
            cc.bond(&staker_id);

            // self.intern

            self.internal_save_staker(&staker_id, &staker)
        }

        self.internal_save_consumer_chain(&cc.consumer_chain_id, &cc);
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
}
