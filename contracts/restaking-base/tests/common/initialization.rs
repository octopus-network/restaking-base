use anyhow::Ok;
use near_sdk::log;
use restaking_base_contract::external::consumer_chain_pos::ConsumerChainPos;

use crate::{
    common::*,
    contracts::{
        mock_consumer_chain_pos::MockConsumerChainPosContract, whitelist::WhitelistContract,
    },
};

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
    pub staking_pool1_contract: StakingPoolContract,
    pub foundation_account: Account,
    pub whitelist_contract: WhitelistContract,
    pub restaking_base_contract: RestakingBaseContract,
    pub restaking_base_owner: Account,
    pub staker1: Account,
    pub cc_gov: Account,
    pub cc_treasury: Account,
    pub cc_pos_contract: MockConsumerChainPosContract,
    pub test_chain_id: String,
}

pub async fn setup_staker_select_pool(common_test_env: &CommonTestEnv) -> anyhow::Result<()> {
    let result = common_test_env
        .restaking_base_contract
        .storage_deposit(
            &common_test_env.staker1,
            None,
            None,
            parse_near!("0.1 near"),
        )
        .await;
    assert_result_success(&result, "Failed to storage_deposit.");
    let result = common_test_env
        .restaking_base_contract
        .select_pool(
            &common_test_env.staker1,
            common_test_env
                .staking_pool_contract
                .deploy_account
                .id()
                .clone(),
        )
        .await;
    assert_result_success(&result, "Failed to select pool.");
    let is_select_success: bool = result.json().unwrap();
    assert!(is_select_success, "Failed to select pool.");
    log!("Staker1 select pool success.");
    Ok(())
}

pub async fn setup_common_test_env(worker: &Worker<Sandbox>) -> anyhow::Result<CommonTestEnv> {
    let staking_pool_owner = register_account(&worker, "staking_pool_owner").await;
    let staking_pool_contract = init_staking_pool(
        register_account(&worker, "staking_pool").await,
        staking_pool_owner.id().clone(),
    )
    .await;
    let staking_pool1_contract = init_staking_pool(
        register_account(&worker, "staking_pool1").await,
        staking_pool_owner.id().clone(),
    )
    .await;
    let foundation_account = register_account(&worker, "foundation").await;
    let whitelist_contract = WhitelistContract::deploy(
        register_account(&worker, "whitelist").await,
        foundation_account.id().clone(),
    )
    .await;
    whitelist_contract
        .add_staking_pool(
            &foundation_account,
            staking_pool_contract.deploy_account.id().clone(),
        )
        .await
        .into_result()?;
    whitelist_contract
        .add_staking_pool(
            &foundation_account,
            staking_pool1_contract.deploy_account.id().clone(),
        )
        .await
        .into_result()?;
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

    let cc_gov = register_account(&worker, "cc_gov").await;
    restaking_base_contract
        .storage_deposit(
            &cc_gov,
            Some(cc_gov.id().clone()),
            None,
            parse_near!("0.1 near"),
        )
        .await
        .into_result()?;

    let cc_treasury = register_account(&worker, "cc_treasury").await;
    let cc_pos_contract =
        MockConsumerChainPosContract::deploy(register_account(&worker, "cc_pos").await).await;

    let cc_id = "test:test".to_string();
    let cc_unbond_period = (86400 * 7) as u64;
    restaking_base_contract
        .register_consumer_chain(
            &cc_gov,
            ConsumerChainRegisterParam {
                consumer_chain_id: cc_id.clone(),
                cc_pos_account: near_sdk::AccountId::new_unchecked(
                    cc_pos_contract.deploy_account.id().to_string(),
                ),
                unbond_period: cc_unbond_period,
                website: "website".to_string(),
                treasury: near_sdk::AccountId::new_unchecked(cc_treasury.id().to_string()),
            },
            CC_REGISTER_FEE,
        )
        .await
        .into_result()?;

    anyhow::Ok(CommonTestEnv {
        staking_pool_owner,
        staking_pool_contract,
        staking_pool1_contract,
        foundation_account,
        whitelist_contract,
        restaking_base_contract,
        restaking_base_owner,
        staker1,
        cc_gov,
        cc_treasury,
        cc_pos_contract,
        test_chain_id: cc_id,
    })
}
