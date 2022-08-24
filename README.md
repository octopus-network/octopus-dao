# Octopus DAO

The DAO implementation of Octopus Network.

## Octopus Smart Contracts Structure

On NEAR protocol, the Octopus Network infrastructure is consisted of a set of smart contracts. The structure is shown below:

![Smart Contracts Structure](/images/near-contract-accounts.png)

* `octopus-registry.near` - A top-level account on NEAR protocol. The [Octopus Appchain Registry](https://github.com/octopus-network/octopus-appchain-registry) contract lived in this account. This is also the root account of Octopus Network infrastructure.
* `<appchain id>.octopus-registry.near` - A sub-account of octopus appchain registry account. The [Octopus Appchain Anchor](https://github.com/octopus-network/octopus-appchain-anchor) contract lived in this account. Each Octopus Appchain has a corresponding anchor contract, named by its `appchain id`.
* `<appchain id>-token.near` - A top-level account on NEAR protocol. The [Wrapped Appchain Token](https://github.com/octopus-network/wrapped-appchain-token) contract lived in this account. Each Octopus Appchain has a corresponding token contract, named by its `appchain id` plus `-token`.
* `<NFT class name>.<appchain id>.octopus-registry.near` - A sub-account of octopus appchain anchor account. The [Wrapped Appchain NFT](https://github.com/octopus-network/wrapped-appchain-nft) contract lived in this account. This contract is deployed automatically when register a NFT class in `Octopus Appchain Anchor`. Each class of wrapped appchain NFT has a corresponding contract of this template, named by its `class name`.
* `near-receiver.<appchain id>.octopus-registry.near` - A sub-account of octopus appchain anchor account. A contract for receiving native near token for cross-chain transfer lived in this contract. _**(under development)**_
* `wat-faucet.<appchain id>.octopus-registry.near` - A sub-account of octopus appchain anchor account. A contract for automatically issuing wrapped appchain token to new validator of an appchain lived in this account. The source code is [inside of octopus appchain anchor](https://github.com/octopus-network/octopus-appchain-anchor/tree/main/wat-faucet).
* `octopus-dao.sputnik-dao.near` - A sub-account of [sputnik-dao](https://github.com/near-daos/sputnik-dao-contract) account lived in NEAR protocol. This contract includes all basic rules and operations for the operation of Octopus DAO.
* `octopus-council.octopus-registry.near` - A sub-account of octopus registry account. A contract for automatically managing the members of `Octopus Council` will live in this account.

## Octopus Council Contract

The Octopus Council consists of a part of all validators of all appchains lived in Octopus Network.

The top `X` of validators with the most stake in all appchains will automatically become the members of Octopus Council. The number `X` is a setting in Octopus Council contract which can be changed in future by a proposal in Octopus DAO. And the stake of a validator is the total stake of his/her stake (including all of the delegation) in all appchains lived in Octopus Network.

The octopus council contract will sort all validators in descending order based on their total stake in all appchains. The sequence is:

![Sync Validators Stake](/images/sync-validators-stake.png)

This contract has a set of view functions to show the status of octopus council.

## Octopus DAO Contract

The DAO contract implemented using [sputnik-dao](https://github.com/near-daos/sputnik-dao-contract).

The proposal kinds defined in sputnik DAO are as the following:

Proposal Kind | Description
---- | ----
ChangeConfig | Change the DAO config.
ChangePolicy | Change the full policy.
AddMemberToRole | Add member to given role in the policy. This is short cut to updating the whole policy.
RemoveMemberFromRole | Remove member to given role in the policy. This is short cut to updating the whole policy.
FunctionCall | Calls `receiver_id` with list of method names in a single promise. Allows this contract to execute any arbitrary set of actions in other contracts.
UpgradeSelf | Upgrade this contract with given hash from blob store.
UpgradeRemote | Upgrade another contract, by calling method with the code from given hash from blob store.
Transfer | Transfers given amount of `token_id` from this DAO to `receiver_id`. If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token. For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
SetStakingContract | Sets staking contract. Can only be proposed if staking contract is not set yet.
AddBounty | Add new bounty.
BountyDone | Indicates that given bounty is done by given user.
Vote | Just a signaling vote, with no execution.
FactoryInfoUpdate | Change information about factory and auto update.
ChangePolicyAddOrUpdateRole | Add new role to the policy. If the role already exists, update it. This is short cut to updating the whole policy.
ChangePolicyRemoveRole | Remove role from the policy. This is short cut to updating the whole policy.
ChangePolicyUpdateDefaultVotePolicy | Update the default vote policy from the policy. This is short cut to updating the whole policy.
ChangePolicyUpdateParameters | Update the parameters from the policy. This is short cut to updating the whole policy.

A initial policy is needed when create the sputnik DAO contract. It is defined as:

Field | Description | Value | Notes
---- | ---- | ---- | ----
roles | List of roles and permissions for them in the current policy. | [`council_permissions`, `council_manager_permissions`] |
default_vote_policy | Default vote policy. Used when given proposal kind doesn't have special policy. | {"weight_kind":"WeightKind::RoleWeight", "quorum":"minimum number of votes", "threshold":"WeightOrRatio::Ratio(1, 2)"} |
proposal_bond | Bond for a proposal. | 0 | 0 NEAR
proposal_period | Expiration period for proposals. | 604800000000000 | 7 days
bounty_bond | Bond for claiming a bounty. | 1000000000000000000000000 | 1 NEAR
bounty_forgiveness_period | Period in which giving up on bounty is not punished. | 86400000000000 | 1 day

The council manager permissions are defined as:

Field | Description | Value | Notes
---- | ---- | ---- | ----
name | Name of the role to display to the user. | council_manager |
kind | Kind of the role: defines which users this permissions apply. | RoleKind::Group({"`account id of Octopus Council contract`"}) |
permissions | Set of actions on which proposals that this role is allowed to execute. | {"ChangePolicyAddOrUpdateRole:\*"} |
vote_policy | For each proposal kind, defines voting policy. | {} |

The council permissions are defined as:

Field | Description | Value | Notes
---- | ---- | ---- | ----
name | Name of the role to display to the user. | council |
kind | Kind of the role: defines which users this permissions apply. | RoleKind::Group({}) | This field is managed by council manager automatically.
permissions | Set of actions on which proposals that this role is allowed to execute. | {"\*:\*"} | Maybe should specify a list of proposal kind and actions.
vote_policy | For each proposal kind, defines voting policy. | {} | Using default vote policy is OK.
