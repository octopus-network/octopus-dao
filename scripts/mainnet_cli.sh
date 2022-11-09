#!/bin/bash
set -e
#
export NEAR_ENV=mainnet
export REGISTRY_ACCOUNT_ID=octopus-registry.near
export COUNCIL_ACCOUNT_ID=octopus-council.$REGISTRY_ACCOUNT_ID
#
#
#
# near deploy --accountId $COUNCIL_ACCOUNT_ID --wasmFile res/octopus_council.wasm
#
# near call $COUNCIL_ACCOUNT_ID migrate_state '' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# WASM_BYTES='cat res/octopus_council.wasm | base64'
# near call $COUNCIL_ACCOUNT_ID store_wasm_of_self $(eval "$WASM_BYTES") --base64 --accountId $COUNCIL_ACCOUNT_ID --deposit 3 --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID update_self '' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID set_dao_contract_account '{"account_id":"octopus-dao.sputnikv2.testnet"}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID update_council_change_histories --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID apply_change_histories_to_dao_contract '{"start_index":"0"}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID set_max_number_of_council_members '{"max_number_of_council_members":10}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID clear_council_members_and_regenerate_change_histories '' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
#
# near call $COUNCIL_ACCOUNT_ID set_excluding_validator_accounts '{"accounts":["alice-octopus.testnet","bob-octopus.testnet","charlie-octopus.testnet","dave-octopus.testnet"]}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
