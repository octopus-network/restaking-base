use crate::*;

pub trait OwnerAction {
    fn set_new_owner(&mut self, new_owner: AccountId);
    fn set_cc_register_fee(&mut self, new_cc_register_fee: U128);
    fn set_slash_guarantee(&mut self, new_slash_guarantee: U128);
    fn set_contract_running(&mut self);
    fn set_contract_pause(&mut self);
    fn set_withdrawal_beneficiary(
        &mut self,
        account_id: AccountId,
        withdraw_certificate: WithdrawalCertificate,
        new_beneficiary: AccountId,
    );
    fn set_staker_unbonding_unlock_time_as_current_time(&mut self, staker_id: AccountId);
}
