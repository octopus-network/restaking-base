use crate::common::*;

pub struct WhitelistContract {
    pub deploy_account: Account,
}

impl WhitelistContract {
    pub async fn deploy(
        deploy_account: Account,
        foundation_account_id: AccountId,
    ) -> WhitelistContract {
        let result = deploy_account
            .deploy(STAKING_POOL_WHITELIST_WASM_BYTES)
            .await
            .unwrap()
            .details;
        assert_result_success(&result, "Failed to deploy WhitelistContract");

        let result = deploy_account
            .call(deploy_account.id(), "new")
            .args_json(json!({ "foundation_account_id": foundation_account_id }))
            .transact()
            .await
            .unwrap();

        assert_result_success(&result, "Failed to call RestakingBaseContract new");

        WhitelistContract { deploy_account }
    }

    pub async fn add_staking_pool(
        &self,
        signer: &Account,
        staking_pool_account_id: AccountId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "add_staking_pool")
            .args_json(json!({
                "staking_pool_account_id": staking_pool_account_id
            }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn is_whitelisted(
        &self,
        signer: &Account,
        staking_pool_account_id: AccountId,
    ) -> bool {
        signer
            .view(&self.deploy_account.id(), "is_whitelisted")
            .args_json(json!({
                "staking_pool_account_id": staking_pool_account_id
            }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }
}
