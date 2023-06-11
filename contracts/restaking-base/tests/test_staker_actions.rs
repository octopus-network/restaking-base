mod common;
mod contracts;

use anyhow::Ok;
use common::*;

#[tokio::test]
async fn test_select_pool() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let common_test_env = setup_common_test_env(&worker).await;
    common_test_env
        .restaking_base_contract
        .select_pool(
            &common_test_env.staker1,
            common_test_env
                .staking_pool_contract
                .deploy_account
                .id()
                .clone(),
        )
        .await
        .into_result()?;

    Ok(())
}

#[tokio::test]
async fn test_register_consumer_chain() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let common_test_env = setup_common_test_env(&worker).await;

    // let common_test_env.restaking_base_contract.register_consumer_chain(signer, register_param);

    Ok(())
}
