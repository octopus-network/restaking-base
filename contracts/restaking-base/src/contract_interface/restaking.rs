use crate::types::SlashId;

use super::*;

pub trait GovernanceAction {
    fn register_consumer_chain(&mut self, register_param: ConsumerChainRegisterParam);

    fn deregister_consumer_chain(&mut self, consumer_chain_account_id: ConsumerChainId);

    fn update_consumer_chain_info(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        update_param: ConsumerChainUpdateParam,
    );

    fn slash(&mut self, consumer_chain_id: ConsumerChainId, slash_id: SlashId, is_approve: bool);
}

pub trait ConsumerChainAction {
    fn blackout(&mut self, consumer_chain_id: ConsumerChainId, staker_id: StakerId);
    fn slash_request(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        slash_items: Vec<(AccountId, U128)>,
        evidence_sha256_hash: String,
    ) -> SlashId;
}

pub trait StakerRestakingAction {
    fn change_key(&mut self, consumer_chain_id: ConsumerChainId, new_key: String);
    fn bond(&mut self, consumer_chain_id: ConsumerChainId, key: String) -> PromiseOrValue<bool>;
    fn unbond(&mut self, consumer_chain_id: ConsumerChainId);
}

pub trait RestakingCallback {
    fn bond_callback(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        key: String,
        staker_id: AccountId,
    ) -> PromiseOrValue<bool>;

    fn change_key_callback(
        &mut self,
        consumer_chain_id: ConsumerChainId,
        new_key: String,
        staker_id: AccountId,
    );
}
