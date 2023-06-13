use crate::*;

pub trait OwnerAction {
    fn set_new_owner(&mut self, new_owner: AccountId);
    fn set_cc_register_fee(&mut self, new_cc_register_fee: U128);
    fn set_slash_guarantee(&mut self, new_slash_guarantee: U128);
}

pub trait ContractSettingView {
    fn get_owner(&self) -> AccountId;
    fn get_cc_register_fee(&self) -> U128;
    fn get_slash_guarantee(&self) -> U128;
}
