use crate::common::*;

pub struct MockConsumerChainPosContract {
    pub deploy_account: Account,
}

impl MockConsumerChainPosContract {
    pub async fn deploy(deploy_account: Account) -> MockConsumerChainPosContract {
        let result = deploy_account
            .deploy(MOCK_CONSUMER_CHAIN_POS_WASM_BYTES)
            .await
            .unwrap()
            .details;

        assert_result_success(&result, "Failed to deploy MockConsumerChainPosContract");
        let result = deploy_account
            .call(deploy_account.id(), "new")
            .args_json(json!({}))
            .transact()
            .await
            .unwrap();

        assert_result_success(&result, "Failed to call MockConsumerChainPosContract new");
        MockConsumerChainPosContract { deploy_account }
    }

    pub async fn set_should_bond_success(
        &self,
        signer: &Account,
        should_bond_success: bool,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "set_should_bond_success")
            .args_json(json!({ "should_bond_success": should_bond_success }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn set_should_change_key_success(
        &self,
        signer: &Account,
        should_change_key_success: bool,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "set_should_change_key_success")
            .args_json(json!({
                "should_change_key_success": should_change_key_success
            }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn bond(
        &self,
        signer: &Account,
        staker_id: AccountId,
        key: Key,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "bond")
            .args_json(json!({"staker_id": staker_id, "key": key}))
            .transact()
            .await
            .unwrap()
    }

    pub async fn change_key(
        &self,
        signer: &Account,
        staker_id: AccountId,
        key: Key,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "change_key")
            .args_json(json!({"staker_id": staker_id, "key": key}))
            .transact()
            .await
            .unwrap()
    }
}
