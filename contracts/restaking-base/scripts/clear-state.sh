#!/bin/bash
set -e

source ./variables.sh

# reference: https://github.com/near/core-contracts/tree/master/state-cleanup
# 1. deploy state_cleanup wasm
near deploy $VESTING_CONTRACT_ACCOUNT_ID ../res/state_cleanup.wasm &&
# 2. cleanup state
python3 state-clean.py $VESTING_CONTRACT_ACCOUNT_ID $VESTING_ACCOUNT_ID