use near_sdk::AccountId;
use crate::types::{ConsumerChainId, DurationInSeconds};
use crate::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ConsumerChainUpdateParam {
    pub unbond_period: Option<DurationInSeconds>,
    pub website: Option<String>,
    pub treasury: Option<AccountId>,
    pub governance: Option<AccountId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ConsumerChainRegisterParam {
    pub consumer_chain_id: ConsumerChainId,
    pub cc_pos_account: AccountId,
    pub unbond_period: DurationInSeconds,
    pub website: String,
    pub treasury: AccountId,
    // pub goverannce: AccountId
}

// todo
#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ConsumerChainStatus {
    Running,
    Unregistered
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConsumerChain {
    pub consumer_chain_id: ConsumerChainId,
    pub unbond_period: DurationInSeconds,
    pub website: String,
    pub governance: AccountId,
    // todo need a more suitable datastruct
    pub bonding_stakers: UnorderedSet<StakerId>,
    pub treasury: AccountId,
    pub status: ConsumerChainStatus,
    pub pos_account_id: AccountId,
    pub blacklist: UnorderedSet<AccountId>,
}

impl ConsumerChain {

    pub fn assert_cc_gov(&self) {
        let predecessor_account_id = env::predecessor_account_id();

        assert_eq!(
            predecessor_account_id, self.governance,
            "The predecessor_account_id({}) is not consumer chain governance({})",
            predecessor_account_id, self.governance
        );
    }

    pub fn assert_cc_pos_account(&self) {
        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(
            predecessor_account_id, self.pos_account_id,
            "The predecessor_account_id({}) is not pos_account_id({})",
            predecessor_account_id, self.pos_account_id
        );
    }

    pub fn new_from_register_param(register_param: ConsumerChainRegisterParam, governance: AccountId) -> Self {
        Self {
            consumer_chain_id: register_param.consumer_chain_id.clone(),
            unbond_period: register_param.unbond_period,
            website: register_param.website,
            governance: governance,
            bonding_stakers: UnorderedSet::new(StorageKey::ConsumerChainBondingStakers { consumer_chain_id: register_param.consumer_chain_id.clone()}),
            treasury: register_param.treasury,
            // todo
            status: ConsumerChainStatus::Running,
            pos_account_id: register_param.cc_pos_account,
            blacklist: UnorderedSet::new(StorageKey::ConsumerChainBlackList { consumer_chain_id: register_param.consumer_chain_id.clone() })
        }
    }

    pub fn update(&mut self, update_param: ConsumerChainUpdateParam) {

        if update_param.unbond_period.is_some() {
            self.unbond_period = update_param.unbond_period.unwrap();
        }

        if update_param.website.is_some() {
            self.website = update_param.website.unwrap();
        }

        if update_param.treasury.is_some() {
            self.treasury = update_param.treasury.unwrap();
        }

        if update_param.governance.is_some() {
            self.governance = update_param.governance.unwrap();
        }
    }

    pub fn assert_running(&self) {
        assert!(matches!(self.status, ConsumerChainStatus::Running), "The consumer chain({}) is not running.", self.consumer_chain_id);
    }

    pub fn bond(&mut self, staker_id: &StakerId) {
        self.assert_running();
        self.bonding_stakers.insert(staker_id);
    }

    pub fn unbond(&mut self, staker_id: &StakerId) {
        self.bonding_stakers.remove(staker_id);
    }

}

impl RestakingBaseContract {

    pub fn internal_get_consumer_chain_or_panic(&self, consumer_chain_id: &ConsumerChainId) -> ConsumerChain {
        self.consumer_chains.get(consumer_chain_id).expect(format!("Failed to get consumer chain by {}", consumer_chain_id).as_str())
    }

    pub(crate) fn internal_save_consumer_chain(&mut self, consumer_chain_id: &ConsumerChainId, consumer_chain: &ConsumerChain) {
        self.consumer_chains.insert(consumer_chain_id, &consumer_chain);
    }

    pub(crate) fn internal_use_consumer_chain_or_panic<F, R>(&mut self, consumer_chain_id: &ConsumerChainId, mut f: F) -> R 
    where
        F: FnMut(&mut ConsumerChain) -> R,
    {
        let mut consumer_chain = self.internal_get_consumer_chain_or_panic(consumer_chain_id);
        let r = f(&mut consumer_chain);
        self.internal_save_consumer_chain(consumer_chain_id, &consumer_chain);
        r
    }

    pub fn get_top_stakers_by_shares() {

    }

}