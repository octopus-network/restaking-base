use crate::*;

pub trait OwnerAction {
    fn set_new_owner(&mut self, new_owner: AccountId);
    fn set_cc_register_fee(&mut self, new_cc_register_fee: U128);
    fn set_slash_guarantee(&mut self, new_slash_guarantee: U128);
    fn set_contract_running(&mut self);
    fn set_contract_pause(&mut self);
}
