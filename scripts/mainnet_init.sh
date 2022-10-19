#!/bin/bash
set -e
#
export NEAR_ENV=mainnet
export REGISTRY_ACCOUNT_ID=octopus-registry.near
export COUNCIL_ACCOUNT_ID=octopus-council.$REGISTRY_ACCOUNT_ID
#
#
#
cp ~/.near-credentials/mainnet/$REGISTRY_ACCOUNT_ID.json ~/.near-credentials/mainnet/$COUNCIL_ACCOUNT_ID.json
sed -i '' "s/$REGISTRY_ACCOUNT_ID/$COUNCIL_ACCOUNT_ID/" ~/.near-credentials/mainnet/$COUNCIL_ACCOUNT_ID.json
#
near create-account $COUNCIL_ACCOUNT_ID --masterAccount $REGISTRY_ACCOUNT_ID --publicKey ed25519:4K1r59zXJ46URSCGSpaFPwupnxuMbHxkgnuAHLTvvWKK --initialBalance 3
#
near deploy --accountId $COUNCIL_ACCOUNT_ID --wasmFile res/octopus_council.wasm
near call $COUNCIL_ACCOUNT_ID new '{"dao_contract_account":"octopus-dao.sputnik-dao.near","max_number_of_council_members":21}' --accountId $COUNCIL_ACCOUNT_ID --gas 200000000000000
