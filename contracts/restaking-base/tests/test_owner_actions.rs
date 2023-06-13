mod common;
mod contracts;

use anyhow::Ok;
use common::*;

#[tokio::test]
async fn test_owner_actions() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let env = setup_common_test_env(&worker).await?;

    let signer = &env.restaking_base_owner;
    env.restaking_base_contract
        .set_cc_register_fee(signer, parse_near!("15 near").into())
        .await
        .into_result()?;
    assert_eq!(
        parse_near!("15 near"),
        env.restaking_base_contract
            .get_cc_register_fee(signer)
            .await
            .0
    );
    env.restaking_base_contract
        .set_slash_guarantee(signer, parse_near!("15 near").into())
        .await
        .into_result()?;
    assert_eq!(
        parse_near!("15 near"),
        env.restaking_base_contract
            .get_slash_guarantee(signer)
            .await
            .0
    );

    let new_owner = AccountId::from_str("new_owner").unwrap();
    env.restaking_base_contract
        .set_new_owner(signer, new_owner.clone())
        .await
        .into_result()?;
    assert_eq!(
        new_owner.to_string(),
        env.restaking_base_contract
            .get_owner(signer)
            .await
            .to_string()
    );

    Ok(())
}
