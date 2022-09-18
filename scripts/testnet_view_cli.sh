#!/bin/bash
set -e
#
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://near-testnet.infura.io/v3/4f80a04e6eb2437a9ed20cb874e10d55
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://public-rpc.blockpi.io/http/near-testnet
export NEAR_ENV=testnet
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export COUNCIL_ACCOUNT_ID=octopus-council.$REGISTRY_ACCOUNT_ID
#
#
#
near view $COUNCIL_ACCOUNT_ID version
#
near view $COUNCIL_ACCOUNT_ID get_living_appchain_ids
#
near view $COUNCIL_ACCOUNT_ID get_council_members
#
near view $COUNCIL_ACCOUNT_ID get_validator_stake_of '{"account_id":"alice-octopus.testnet"}'
near view $COUNCIL_ACCOUNT_ID get_validator_stake_of '{"account_id":"bob-octopus.testnet"}'
near view $COUNCIL_ACCOUNT_ID get_validator_stake_of '{"account_id":"charlie-octopus.testnet"}'
near view $COUNCIL_ACCOUNT_ID get_validator_stake_of '{"account_id":"dave-octopus.testnet"}'
