#!/bin/bash
set -e

source ./variables.sh

cd ..
bash build.sh &&
cd scripts

if [ "$1" == "deploy" ]; then
  near deploy $DEPLOY_ACCOUNT ../res/$RESTAKING_BASE_WASM_NAME new '{"owner": "'$OWNER_ACCOUNT_ID'", "cc_register_fee": "'$CC_REGISTER_FEE'", "staking_pool_whitelist_account": "'$STAKING_POOL_WHITELIST_ACCOUNT'", "slash_guarantee": "'$SLASH_GUARANTEE'"}'
elif [ "$1" == "redeploy" ]; then
  near deploy $DEPLOY_ACCOUNT ../res/$RESTAKING_BASE_WASM_NAME
elif [ "$1" == "clean" ]; then
  bash clear-state.sh && near deploy $DEPLOY_ACCOUNT ../res/$RESTAKING_BASE_WASM_NAME new '{"owner": "'$OWNER_ACCOUNT_ID'", "cc_register_fee": "'$CC_REGISTER_FEE'", "staking_pool_whitelist_account": "'$STAKING_POOL_WHITELIST_ACCOUNT'", "slash_guarantee": "'$SLASH_GUARANTEE'"}'
fi
