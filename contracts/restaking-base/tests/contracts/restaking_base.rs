use near_sdk::ONE_YOCTO;

use crate::common::*;
use crate::contracts::storgae_management::*;

pub struct RestakingBaseContract {
    pub deploy_account: Account,
}

impl NearContract for RestakingBaseContract {
    fn get_deploy_account(&self) -> &Account {
        &self.deploy_account
    }
}

// owner: AccountId, cc_register_fee: Balance, staking_pool_whitelist_account: AccountId, slash_guarantee: Balance

#[allow(unused)]
impl RestakingBaseContract {
    pub async fn deploy(
        deploy_account: Account,
        owner: AccountId,
        cc_register_fee: U128,
        staking_pool_whitelist_account: AccountId,
        slash_guarantee: U128,
    ) -> RestakingBaseContract {
        let result = deploy_account
            .deploy(RESTAKING_BASE_WASM_BYTES)
            .await
            .unwrap()
            .details;
        assert_result_success(&result, "Failed to deploy RestakingBaseContract");

        let result = deploy_account
            .call(deploy_account.id(), "new")
            .args_json(json!({
                "owner": owner, 
                "cc_register_fee": cc_register_fee, 
                "staking_pool_whitelist_account": staking_pool_whitelist_account, 
                "slash_guarantee": slash_guarantee}))
            .gas(parse_gas!("100 Tgas") as u64)
            .transact()
            .await
            .unwrap();
        assert_result_success(&result, "Failed to call RestakingBaseContract new");

        RestakingBaseContract { deploy_account }
    }

    // #region GovernanceAction
    pub async fn register_consumer_chain(
        &self,
        signer: &Account,
        register_param: ConsumerChainRegisterParam,
        register_fee: Balance,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "register_consumer_chain")
            .args_json(json!({ "register_param": register_param }))
            .deposit(register_fee)
            .transact()
            .await
            .unwrap()
    }

    pub async fn deregister_consumer_chain(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "deregister_consumer_chain")
            .args_json(json!({ "consumer_chain_id": consumer_chain_id }))
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    pub async fn update_consumer_chain_info(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        update_param: ConsumerChainUpdateParam,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "update_consumer_chain_info")
            .args_json(
                json!({"consumer_chain_id": consumer_chain_id, "update_param": update_param}),
            )
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    pub async fn slash(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        slash_id: SlashId,
        is_approve: bool,
    ) -> ExecutionFinalResult {
        signer.call(&self.deploy_account.id(), "slash")
        .args_json(json!({"consumer_chain_id": consumer_chain_id, "slash_id": slash_id, "is_approve": is_approve}))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await
        .unwrap()
    }
    // #endregion

    // #region ConsumerChainAction
    pub async fn blackout(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        staker_id: AccountId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "blackout")
            .args_json(json!({"consumer_chain_id": consumer_chain_id, "staker_id": staker_id}))
            .transact()
            .await
            .unwrap()
    }
    pub async fn slash_request(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        slash_items: Vec<(AccountId, U128)>,
        evidence_sha256_hash: String,
    ) -> ExecutionFinalResult {
        signer.call(&self.deploy_account.id(), "slash_request")
        .args_json(json!({"consumer_chain_id": consumer_chain_id, "slash_items": slash_items, "evidence_sha256_hash": evidence_sha256_hash }))
        .deposit(SLASH_GUARANTEE)
        .transact()
        .await
        .unwrap()
    }
    // #endregion

    // #region StakerRestakingAction
    pub async fn bond(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        key: String,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "bond")
            .args_json(json!({"consumer_chain_id": consumer_chain_id, "key": key}))
            .gas(parse_gas!("150 Tgas") as u64)
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    pub async fn change_key(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        new_key: String,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "change_key")
            .args_json(json!({"consumer_chain_id": consumer_chain_id, "new_key": new_key}))
            .gas(parse_gas!("150 Tgas") as u64)
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    pub async fn unbond(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "unbond")
            .args_json(json!({ "consumer_chain_id": consumer_chain_id }))
            .gas(parse_gas!("150 Tgas") as u64)
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    // #endregion

    // #region ReStakingView

    pub async fn call_get_validator_set(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        limit: u32,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "get_validator_set")
            .args_json(json!({ "consumer_chain_id": consumer_chain_id, "limit": limit }))
            .max_gas()
            .transact()
            .await
            .unwrap()
    }

    pub async fn get_validator_set(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
        limit: u32,
    ) -> ValidatorSet {
        signer
            .view(&self.deploy_account.id(), "get_validator_set")
            .args_json(json!({ "consumer_chain_id": consumer_chain_id, "limit": limit }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_consumer_chain(
        &self,
        signer: &Account,
        consumer_chain_id: ConsumerChainId,
    ) -> Option<ConsumerChainInfo> {
        signer
            .view(&self.deploy_account.id(), "get_consumer_chain")
            .args_json(json!({ "consumer_chain_id": consumer_chain_id }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_slash_guarantee(&self, signer: &Account) -> U128 {
        signer
            .view(&self.deploy_account.id(), "get_slash_guarantee")
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_cc_register_fee(&self, signer: &Account) -> U128 {
        signer
            .view(&self.deploy_account.id(), "get_cc_register_fee")
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_owner(&self, signer: &Account) -> AccountId {
        signer
            .view(&self.deploy_account.id(), "get_owner")
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    // #endregion

    // #region StakerAction
    pub async fn select_pool(&self, signer: &Account, pool_id: AccountId) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "select_pool")
            .args_json(json!({ "pool_id": pool_id }))
            .deposit(ONE_YOCTO)
            .gas(parse_gas!("100 Tgas") as u64)
            .transact()
            .await
            .unwrap()
    }
    pub async fn ping(&self, signer: &Account, pool_id: Option<PoolId>) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "ping")
            .args_json(json!({ "pool_id": pool_id }))
            .transact()
            .await
            .unwrap()
    }
    pub async fn increase_stake(
        &self,
        signer: &Account,
        increase_amount: u128,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "increase_stake")
            .deposit(increase_amount)
            .gas(parse_gas!("200 Tgas") as u64)
            .transact()
            .await
            .unwrap()
    }
    pub async fn decrease_stake(
        &self,
        signer: &Account,
        decrease_amount: U128,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "decrease_stake")
            .args_json(json!({ "decrease_amount": decrease_amount }))
            .deposit(ONE_YOCTO)
            .gas(parse_gas!("200 Tgas") as u64)
            .transact()
            .await
            .unwrap()
    }
    pub async fn unstake(&self, signer: &Account) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "unstake")
            .deposit(ONE_YOCTO)
            .gas(parse_gas!("200 Tgas") as u64)
            .transact()
            .await
            .unwrap()
    }
    pub async fn withdraw_all(
        &self,
        signer: &Account,
        account_id: AccountId,
        pool_id: PoolId,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "withdraw_all")
            .deposit(ONE_YOCTO)
            .transact()
            .await
            .unwrap()
    }

    // #endregion

    // #region Storage Management
    pub async fn storage_deposit(
        &self,
        signer: &Account,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
        deposit_amount: Balance,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "storage_deposit")
            .deposit(deposit_amount)
            .args_json(json!({
                "account_id": account_id,
                "registration_only": registration_only
            }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn storage_withdraw(
        &self,
        signer: &Account,
        amount: Option<U128>,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "storage_withdraw")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "amount": amount }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn storage_unregister(
        &self,
        signer: &Account,
        force: Option<bool>,
    ) -> ExecutionFinalResult {
        signer
            .call(&self.deploy_account.id(), "storage_unregister")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "force": force }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn storage_balance_bounds(&self, signer: &Account) -> StorageBalanceBounds {
        signer
            .view(self.get_deploy_account().id(), "storage_balance_bounds")
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn storage_balance_of(
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
    // #endregion

    // #region OwnerAction
    pub async fn set_new_owner(
        &self,
        signer: &Account,
        new_owner: AccountId,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "set_new_owner")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "new_owner": new_owner }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn set_cc_register_fee(
        &self,
        signer: &Account,
        new_cc_register_fee: U128,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "set_cc_register_fee")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "new_cc_register_fee": new_cc_register_fee }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn set_slash_guarantee(
        &self,
        signer: &Account,
        new_slash_guarantee: U128,
    ) -> ExecutionFinalResult {
        signer
            .call(self.get_deploy_account().id(), "set_slash_guarantee")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "new_slash_guarantee": new_slash_guarantee }))
            .transact()
            .await
            .unwrap()
    }

    // #endregion
}
