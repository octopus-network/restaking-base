use crate::common::*;

pub struct StakingPoolContract {
    // pub contract_id: workspaces::types::AccountId,
    pub deploy_account: Account,
}

impl StakingPoolContract {
    pub async fn deploy(
        deploy_account: workspaces::Account,
        owner_id: AccountId,
        stake_public_key: PublicKey,
        reward_fee_fraction: RewardFeeFraction,
    ) -> StakingPoolContract {
        let result = deploy_account
            .deploy(STAKING_POOL_WASM_BYTES)
            .await
            .unwrap()
            .details;
        assert_result_success(&result, "Failed to deploy StakingPoolContract");

        let result = deploy_account
            .call(deploy_account.id(), "new")
            .args_json((owner_id, stake_public_key, reward_fee_fraction))
            .gas(parse_gas!("100 Tgas") as u64)
            .transact()
            .await
            .unwrap();
        assert_result_success(&result, "Failed to call StakingPoolContract new");

        StakingPoolContract {
            deploy_account: deploy_account,
        }
    }

    pub async fn get_account_staked_balance(
        &self,
        signer: &Account,
        account_id: AccountId,
    ) -> U128 {
        signer
            .view(&self.deploy_account.id(), "get_account_staked_balance")
            .args_json(json!({"account_id": account_id.clone()}))
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_account_unstaked_balance(
        &self,
        signer: &Account,
        account_id: AccountId,
    ) -> U128 {
        signer
            .view(&self.deploy_account.id(), "get_account_unstaked_balance")
            .args_json(
                json!({"account_id": account_id.clone()})
                    .to_string()
                    .into_bytes(),
            )
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn get_account_total_balance(&self, signer: &Account, account_id: AccountId) -> U128 {
        signer
            .view(&self.deploy_account.id(), "get_account_total_balance")
            .args_json(
                json!({"account_id": account_id.clone()})
                    .to_string()
                    .into_bytes(),
            )
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    pub async fn ping(&self, signer: &Account) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "ping")
            .transact()
            .await
            .unwrap()
    }

    pub async fn deposit(&self, signer: &Account) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "deposit")
            .transact()
            .await
            .unwrap()
    }

    pub async fn deposit_and_stake(
        &self,
        signer: &Account,
        amount: Balance,
    ) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "deposit_and_stake")
            .deposit(amount)
            .transact()
            .await
            .unwrap()
    }

    pub async fn withdraw(&self, signer: &Account, amount: U128) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "withdraw")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "amount": amount }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn stake(&self, signer: &Account, amount: U128) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "stake")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "amount": amount }))
            .transact()
            .await
            .unwrap()
    }

    pub async fn unstake(&self, signer: &Account, amount: U128) -> ExecutionFinalResult {
        signer
            .call(self.deploy_account.id(), "unstake")
            .deposit(ONE_YOCTO)
            .args_json(json!({ "amount": amount }))
            .transact()
            .await
            .unwrap()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}
