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
near view $COUNCIL_ACCOUNT_ID get_max_number_of_council_members
#
near view $COUNCIL_ACCOUNT_ID get_excluding_validator_accounts
#
near view $COUNCIL_ACCOUNT_ID get_council_members
#
near view $COUNCIL_ACCOUNT_ID get_ranked_validator_stakes '{"start_index":0,"quantity":null}'
#
near view $COUNCIL_ACCOUNT_ID get_council_change_histories '{"start_index":"0","quantity":null}'
