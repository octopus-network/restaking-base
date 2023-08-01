use crate::*;
use uint::construct_uint;

pub type ShareBalance = u128;
pub type PoolId = AccountId;
pub type StakerId = AccountId;
pub type DurationOfSeconds = u64;
pub type ConsumerChainId = String;
pub type Key = String;
pub type SlashId = U64;
pub type WithdrawalCertificatetId = U64;
pub type ValidaotrSet = Vec<(AccountId, U128)>;
pub type Sequence = U64;

construct_uint! {
    /// 256-bit unsigned integer.
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U256(4);
}
