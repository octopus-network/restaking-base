#!/bin/bash
set -e

source ./variables.sh

# reference: https://github.com/near/core-contracts/tree/master/state-cleanup
# 1. deploy state_cleanup wasm
near deploy $DEPLOY_ACCOUNT ../res/state_cleanup.wasm &&
# 2. cleanup state
python3 state-clean.py $DEPLOY_ACCOUNT $DEPLOY_ACCOUNT