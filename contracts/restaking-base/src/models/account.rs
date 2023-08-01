use std::collections::HashMap;

use crate::{types::ShareBalance, *};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    // todo stakingpool 没法获取所有持续有shares的人
    pub legacy_shares: HashMap<PoolId, ShareBalance>,

    // todo need more suitable datastruct
    pub pending_withdrawals: UnorderedMap<WithdrawalCertificatetId, PendingWithdrawal>,
}

impl Account {
    pub(crate) fn new(account_id: AccountId) -> Self {
        Account {
            legacy_shares: HashMap::new(),
            pending_withdrawals: UnorderedMap::new(StorageKey::PendingWithdrawals { account_id }),
        }
    }

    pub fn save_legacy_shares(&mut self, shares: ShareBalance, pool_id: PoolId) {
        let new_shares = shares
            .checked_add(self.legacy_shares.get(&pool_id).unwrap_or(&0).to_owned())
            .unwrap();
        self.legacy_shares.insert(pool_id, new_shares);
    }
}

impl RestakingBaseContract {
    pub(crate) fn internal_get_account_or_panic(&self, account_id: &AccountId) -> Account {
        self.accounts
            .get(account_id)
            .expect(format!("Failed to get account by {}", account_id).as_str())
    }

    pub(crate) fn internal_save_account(&mut self, account_id: &AccountId, account: &Account) {
        self.accounts.insert(account_id, account);
    }

    pub(crate) fn internal_use_account<F, R>(&mut self, account_id: &AccountId, mut f: F) -> R
    where
        F: FnMut(&mut Account) -> R,
    {
        let mut account = self.internal_get_account_or_panic(account_id);
        let r = f(&mut account);
        self.internal_save_account(account_id, &account);
        r
    }
}
