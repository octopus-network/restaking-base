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
