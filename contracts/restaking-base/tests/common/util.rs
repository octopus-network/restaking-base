use crate::common::*;

pub async fn register_account(worker: &Worker<Sandbox>, name: &str) -> Account {
    worker
        .root_account()
        .unwrap()
        .create_subaccount(name)
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await
        .unwrap()
        .into_result()
        .unwrap()
}

pub fn assert_result_success(result: &ExecutionFinalResult, msg: &str) {
    assert!(
        result.is_success(),
        "{}. Result Detail is: {:?}",
        msg,
        result
    )
}
