#!/bin/bash
set -e

DEPLOY_ACCOUNT=lotkey.testnet
OWNER_ACCOUNT_ID=ownervesting.testnet
# owner: AccountId,
cc_register_fee: U128,
        staking_pool_whitelist_account: AccountId,
        slash_guarantee


VESTING_ACCOUNT_ID=octvesting.testnet
VESTING_CONTRACT_ACCOUNT_ID=contract.$VESTING_ACCOUNT_ID
ONE_YOCTO=0.000000000000000000000001
OWNER_ACCOUNT_ID=ownervesting.testnet
TOKEN_ID=token.$VESTING_ACCOUNT_ID

VESTING_WASM_NAME=nep141_token_vesting_contract.wasm
TEST_TOKEN_WASM_NAME=test_token.wasm
CREATE_POOL_DEPOSIT_NEAR_AMOUNT=1000000000000000000000000