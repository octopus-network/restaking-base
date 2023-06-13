mod common;
mod contracts;

use anyhow::Ok;
use common::*;

#[tokio::test]
async fn test_register_consumer_chain() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;

    dbg!(
        &env.restaking_base_contract
            .get_consumer_chain(&env.staker1, env.test_chain_id)
            .await
    );

    Ok(())
}

#[tokio::test]
async fn test_update_consumer_chain_info() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    env.restaking_base_contract
        .update_consumer_chain_info(
            &env.cc_gov,
            env.test_chain_id.clone(),
            ConsumerChainUpdateParam {
                unbond_period: Some(86400 * 14),
                website: Some("new website".to_string()),
                treasury: None,
                governance: None,
            },
        )
        .await
        .into_result()?;

    Ok(())
}

#[tokio::test]
async fn test_deregister_consumer_chain() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    env.restaking_base_contract
        .deregister_consumer_chain(&env.cc_gov, env.test_chain_id.clone())
        .await
        .into_result()?;

    Ok(())
}

#[tokio::test]
async fn test_bond() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result()?;
    Ok(())
}

#[tokio::test]
async fn test_change_key() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result()?;
    env.restaking_base_contract
        .change_key(
            &env.staker1,
            env.test_chain_id.clone(),
            "new_key".to_string(),
        )
        .await
        .into_result()?;
    Ok(())
}

#[tokio::test]
async fn test_unbond() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result()?;
    env.restaking_base_contract
        .unbond(&env.staker1, env.test_chain_id.clone())
        .await
        .into_result()?;
    Ok(())
}

#[tokio::test]
async fn test_blackout() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .blackout(
            &env.cc_pos_contract.deploy_account,
            env.test_chain_id.clone(),
            env.staker1.id().clone(),
        )
        .await
        .into_result()?;

    let result = env
        .restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result();
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_slash_request() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;
    env.restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result()?;

    env.restaking_base_contract
        .slash_request(
            &env.cc_pos_contract.deploy_account,
            env.test_chain_id,
            vec![(env.staker1.id().clone(), parse_near!("10 near").into())],
            "evidence_sha256_hash".to_string(),
        )
        .await
        .into_result()?;

    Ok(())
}

#[tokio::test]
async fn test_slash() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;
    setup_staker_select_pool(&env).await?;

    env.restaking_base_contract
        .increase_stake(&env.staker1, parse_near!("10 near"))
        .await
        .into_result()?;

    env.restaking_base_contract
        .bond(&env.staker1, env.test_chain_id.clone(), "key".to_string())
        .await
        .into_result()?;

    let slash_id = env
        .restaking_base_contract
        .slash_request(
            &env.cc_pos_contract.deploy_account,
            env.test_chain_id.clone(),
            vec![(env.staker1.id().clone(), parse_near!("10 near").into())],
            "evidence_sha256_hash".to_string(),
        )
        .await
        .into_result()?
        .json()
        .unwrap();
    env.restaking_base_contract
        .slash(&env.cc_gov, env.test_chain_id.clone(), slash_id, true)
        .await
        .into_result()?;

    Ok(())
}
