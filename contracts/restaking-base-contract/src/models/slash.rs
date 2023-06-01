
use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Slash {
	pub consumer_chain_id: ConsumerChainId,
	pub slash_items: Vec<(AccountId, Balance)>,
	pub evidence_sha256_hash: String 
}

impl RestakingBaseContract {

	pub(crate) fn get_slash_or_panic(&self, slash_id: &SlashId) -> Slash {
        self.slashes
            .get(slash_id)
            .expect(format!("Failed to get slash.").as_str())
    }

	pub(crate) fn internal_remove_slash(&mut self, slash_id: &SlashId) {
		let slash = self.get_slash_or_panic(slash_id);
		self.slashes.remove(slash_id);

		let submitter = self.internal_get_consumer_chain_or_panic(&slash.consumer_chain_id).pos_account_id;
		self.transfer_near(submitter, self.slash_guarantee);
	}
	
}