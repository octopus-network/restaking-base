use crate::common::*;
pub struct Nep141 {
    pub deploy_account: Account,
}

impl Nep141 {
    pub async fn deploy(deploy_account: Account, owner_id: AccountId, wasm: &[u8]) -> Nep141 {
        let result = deploy_account.deploy(wasm).await.unwrap().details;

        assert_result_success(&result, "Failed to deploy Nep141");

        let result = deploy_account
            .call(deploy_account.id(), "new")
            .transact()
            .await
            .unwrap();

        assert_result_success(&result, "Failed to call Nep141 new");

        Nep141 { deploy_account }
    }

    pub async fn ft_transfer_call(
        &self,
        signer: &Account,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "ft_transfer_call")
            .deposit(1)
            .max_gas()
            .args_json(json!((receiver_id, amount, memo, msg)))
            .transact()
            .await
            .unwrap()
    }

    pub async fn storage_deposit(
        &self,
        signer: &Account,
        account_id: AccountId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "storage_deposit")
            .deposit(parse_near!("0.00125 N"))
            .args_json(json!({ "account_id": account_id }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn mint(
        &self,
        signer: &Account,
        account_id: AccountId,
        amount: U128,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "mint")
            .args_json(json!((account_id, amount)))
            .transact()
            .await
            .unwrap()
    }
}
