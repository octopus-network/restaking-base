mod common;
mod contracts;

use anyhow::Ok;
use common::*;

#[tokio::test]
async fn test_select_pool() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let common_test_env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&common_test_env).await?;
    Ok(())
}

#[tokio::test]
async fn test_select_pool_after_selected_pool() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let common_test_env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&common_test_env).await?;
    common_test_env
        .restaking_base_contract
        .select_pool(
            &common_test_env.staker1,
            common_test_env
                .staking_pool1_contract
                .deploy_account
                .id()
                .clone(),
        )
        .await
        .into_result()?;
    Ok(())
}

#[tokio::test]
async fn test_increase_staking() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let common_test_env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&common_test_env).await?;
    let result = common_test_env
        .restaking_base_contract
        .increase_stake(&common_test_env.staker1, parse_near!("1 near"))
        .await;
    dbg!(&result);
    // .into_result()?;
    let staked_balance = common_test_env
        .staking_pool_contract
        .get_account_staked_balance(
            &common_test_env.restaking_base_contract.deploy_account,
            common_test_env
                .restaking_base_contract
                .deploy_account
                .id()
                .clone(),
        )
        .await
        .0;
    assert_eq!(staked_balance, parse_near!("1 near"));

    Ok(())
}

#[tokio::test]
async fn test_decrease_staking() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .increase_stake(&env.staker1, parse_near!("1 near"))
        .await
        .into_result()?;
    let staked_balance = env
        .staking_pool_contract
        .get_account_staked_balance(
            &env.restaking_base_contract.deploy_account,
            env.restaking_base_contract.deploy_account.id().clone(),
        )
        .await
        .0;
    assert_eq!(staked_balance, parse_near!("1 near"));

    let result = env
        .restaking_base_contract
        .decrease_stake(&env.staker1, parse_near!("0.5 near").into())
        .await;
    dbg!(&result);
    let staked_balance = env
        .staking_pool_contract
        .get_account_staked_balance(
            &env.restaking_base_contract.deploy_account,
            env.restaking_base_contract.deploy_account.id().clone(),
        )
        .await
        .0;
    assert_eq!(staked_balance, parse_near!("0.5 near"));

    Ok(())
}

#[tokio::test]
async fn test_unstake() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .increase_stake(&env.staker1, parse_near!("1 near"))
        .await
        .into_result()?;
    let staked_balance = env
        .staking_pool_contract
        .get_account_staked_balance(
            &env.restaking_base_contract.deploy_account,
            env.restaking_base_contract.deploy_account.id().clone(),
        )
        .await
        .0;
    assert_eq!(staked_balance, parse_near!("1 near"));

    env.restaking_base_contract
        .unstake(&env.staker1)
        .await
        .into_result()?;

    let staked_balance = env
        .staking_pool_contract
        .get_account_staked_balance(
            &env.restaking_base_contract.deploy_account,
            env.restaking_base_contract.deploy_account.id().clone(),
        )
        .await
        .0;
    assert_eq!(staked_balance, parse_near!("0 near"));

    Ok(())
}
