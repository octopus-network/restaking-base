use crate::common::*;
use async_trait::async_trait;

#[async_trait]
pub trait StorageManagement {
    // pub deploy_account: Account,
    async fn storage_deposit(
        &self,
        signer: &Account,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> ExecutionFinalResult;

    async fn storage_withdraw(
        &self,
        signer: &Account,
        amount: Option<U128>,
    ) -> ExecutionFinalResult;

    async fn storage_unregister(
        &self,
        signer: &Account,
        force: Option<bool>,
    ) -> ExecutionFinalResult;

    async fn storage_balance_bounds(&self, signer: &Account) -> StorageBalanceBounds;

    async fn storage_balance_of(
        &self,
        signer: &Account,
        account_id: AccountId,
    ) -> Option<StorageBalance>;
}

#[async_trait]
impl<T: NearContract + std::marker::Sync> StorageManagement for T {
    async fn storage_deposit(
        &self,
        signer: &Account,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "storage_deposit")
            .args_json(json!({
                "account_id": account_id,
                "registration_only": registration_only
            }))
            .transact()
            .await
            .unwrap()
    }

    async fn storage_withdraw(
        &self,
        signer: &Account,
        amount: Option<U128>,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "storage_withdraw")
            .args_json(json!({ "amount": amount }))
            .transact()
            .await
            .unwrap()
    }

    async fn storage_unregister(
        &self,
        signer: &Account,
        force: Option<bool>,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "storage_unregister")
            .args_json(json!({ "force": force }))
            .transact()
            .await
            .unwrap()
    }
    async fn storage_balance_bounds(&self, signer: &Account) -> StorageBalanceBounds {
        signer
            .view(self.get_deploy_account().id(), "storage_balance_bounds")
            .await
            .unwrap()
            .json()
            .unwrap()
    }
    async fn storage_balance_of(
        &self,
        signer: &Account,
        account_id: AccountId,
    ) -> Option<StorageBalance> {
        signer
            .view(self.get_deploy_account().id(), "storage_balance_of")
            .await
            .unwrap()
            .json()
            .unwrap()
    }
}
