use crate::{common::*, contracts::whitelist::WhitelistContract};

pub async fn init_staking_pool(
    deploy_account: Account,
    owner_id: AccountId,
) -> StakingPoolContract {
    let public_key = PublicKey::from_str("JD5ZEQjqEkoUQkgrG4AvrFoc4TvpQqAkz3nKniBtGGCK").unwrap();
    let fraction = RewardFeeFraction {
        numerator: 1,
        denominator: 100,
    };
    StakingPoolContract::deploy(deploy_account, owner_id, public_key, fraction).await
}

pub struct CommonTestEnv {
    pub staking_pool_owner: Account,
    pub staking_pool_contract: StakingPoolContract,
    pub foundation_account: Account,
    pub whitelist_contract: WhitelistContract,
    pub restaking_base_contract: RestakingBaseContract,
    pub staker1: Account,
    // storage_management
}

pub async fn setup_common_test_env(worker: &Worker<Sandbox>) -> CommonTestEnv {
    let worker = workspaces::sandbox().await.unwrap();
    let staking_pool_owner = register_account(&worker, "staking_pool_owner").await;
    let staking_pool_contract = init_staking_pool(
        register_account(&worker, "staking_pool").await,
        staking_pool_owner.id().clone(),
    )
    .await;
    let foundation_account = register_account(&worker, "foundation").await;
    let whitelist_contract = WhitelistContract::deploy(
        register_account(&worker, "whitelist").await,
        foundation_account.id().clone(),
    )
    .await;
    let restaking_base_owner = register_account(&worker, "restaking_base_owner").await;
    let restaking_base_contract = RestakingBaseContract::deploy(
        register_account(&worker, "restaking_base").await,
        restaking_base_owner.id().clone(),
        CC_REGISTER_FEE.into(),
        whitelist_contract.deploy_account.id().clone(),
        SLASH_GUARANTEE.into(),
    )
    .await;

    let staker1 = register_account(&worker, "staker1").await;

    CommonTestEnv {
        staking_pool_owner,
        staking_pool_contract,
        foundation_account,
        whitelist_contract,
        restaking_base_contract,
        staker1,
    }
}
