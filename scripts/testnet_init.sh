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
cp ~/.near-credentials/testnet/$REGISTRY_ACCOUNT_ID.json ~/.near-credentials/testnet/$COUNCIL_ACCOUNT_ID.json
sed -i '' "s/$REGISTRY_ACCOUNT_ID/$COUNCIL_ACCOUNT_ID/" ~/.near-credentials/testnet/$COUNCIL_ACCOUNT_ID.json
#
near create-account $COUNCIL_ACCOUNT_ID --masterAccount $REGISTRY_ACCOUNT_ID --publicKey ed25519:5xprFQ2PvGLs5TAFieQzdgLHMHcsHSJnQj8FD6KuXq13 --initialBalance 3
near deploy --accountId $COUNCIL_ACCOUNT_ID --wasmFile res/octopus_council.wasm
near call $COUNCIL_ACCOUNT_ID new '{{"dao_contract_account":"octopus-dao.sputnikv2.testnet","max_number_of_council_members":5}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
