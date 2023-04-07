# Restaking Base

`Restaking Base` is a near contract designed to restake NEAR tokens for `CC PoS` (Consumer Chain PoS). There are several user roles when interacting with the contract, including `Staker`, `CC PoS`, and `CC Gov` (Consumer Chain Governance). `Stakers` can stake their NEAR tokens through the contract, and gain the right to restake to `CC PoS` to provide security, receive rewards from `CC PoS`, and also bear the risk of being slashed. `CC Gov` is responsible for managing `CC PoS` and deciding whether to execute slashes submitted by `CC PoS`.

Contents:

- [Restaking Base](#restaking-base)
    - [Terminology](#terminology)
    - [Use Case](#use-case)
    - [Function specification](#function-specification)
        - [Stake](#stake)
        - [Increase Stake](#increase-stake)
        - [Decrease Stake](#decrease-stake)
        - [Register](#register)
        - [Unregister](#unregister)
        - [Update CC Info](#update-cc-info)
        - [Bond](#bond)
        - [ChangeID](#change-id)
        - [Unbond](#unbond)
        - [Blackout](#blackout)
        - [Slash](#slash)
        - [Query VS](#query-vs)
        - [Unstake](#unstake)
        - [Withdraw](#withdraw)
    - [DataStruct and Interfaces](#datastruct-and-interfaces)

## Terminology

- `Staker`: A user who stake near.
- `Validator`: A near validator.
- `staking pool contract`: A contract deployed on each near `Validator` account.
- `Staking Pool`: An account that deployed the `staking pool contract`.
- `restake`: Stake the near that has already been staked in `Validator` for `Consumer Chain`'s `PoS`.
- `unstake`: A user cancel their stake. If a restake has been performed, it will first execute the `unbond` operation with all `CC PoS`.
- `withdraw`: A user withdraws assets that can be withdrawn from the `Restaking Base`.
- `stake shares`: The total number of shares a user has in the `stake pool contract` for a particular `Staking Pool`.
- `staked balance`: The staking amount of `staker`.
- `update-shares`: Triggers an update of the user's balance in the `Restaking Base` contract. Anyone can call it, and it will also be triggered when stake/unstake/deposit/withdraw operations occur. This operation will call the `ping` function of the `stake pool contract` to obtain the balance of the `Restaking Base` contract in the `stake pool contract` contract and save it locally.
- `Consumer Chain`: A Consumer Chain establishes its PoS contract on NEAR to execute its `reward` and `slash` rules and then registers its `unbonding period`, `CC Gov`, and other parameters with the `Restaking Base` contract.
- `CC PoS`: A PoS contract established by a Consumer Chain on near.
- `CC Gov`: An account specified when registering the `CC PoS`, which has the authority to approve or reject the `slash` submitted by the `CC PoS`.
- `Bond`: After the `Staker` and `Consumer Chain` bond, the `Staker's` near provides security for the `Consumer Chain`, and the `Consumer Chain` provides `Rewards` to the `Staker`. At the same time, the `Consumer Chain` will have the right to initiate a `slash` against the `Staker`.
- `Unbond`: The `Staker` and `Consumer Chain` break the `Bond`. This process requires an `unbonding period` to complete, during which the `Consumer Chain` can apply to `Slash` the `Staker`.
- `Slash`: A Consumer Chain can apply to penalize the `Staker` during the `bonding` or `unbonding` period, and `CC Gov` decides whether to approve or reject the `Slash`.
- `identity`: Staker ID in Consumer Chain.

## Use case
![](images/use-case.jpg)

## Function specification

### Stake

The `stake` operation requires specifying an account which is sub-account of near `staking-pool-factory` account. Users will stake attached NEAR tokens to this account and record it internally in the `staking-base` contract.

Users can only choose one `staking pool` for `staking` at same time. If they want to switch to another `staking pool`, they need to complete an `unstake` operation first.

![](images/stake.png)

### Increase Stake

The `Staker` can increase their `staking` amount after the `stake` operation has been completed.
![](images/increase_stake.png)

### Decrease Stake

The `Staker` can decrease their `staking` amount after the `stake` operation has been completed. The `CC PoS` can't stop `Staker` decreasing their stake amount, but the `CC PoS` can only select `Stakers` with staking balances ranked within a certain range.

![](images/decrease_stake.png)

### Register

This `CC PoS` can submit registration information to the staking-base contract. The following information needs to be provided: `chain_id`, `unbond_period `, `website `, `governance `, and `treasury`. Additionally, a certain amount of NEAR tokens needs to be attached as the registration fee during the registration process.

![](images/register.png)

### Unregister
The `CC Gov` can unregister `CC PoS` and the `restaking-base` will transfer register fee to `treasury` account.

![](images/unregister.png)

### Update CC Info

The `CC Gov` can update `CC PoS` information and this interface must be called by the `governance` account.

![](images/update-info.png)

### Bond

After the `Staker` has completed the `stake` operation through this contract, they can execute the `bond` operation, which will restake their staked NEAR to a specific `consumer chain` to provide security for the `consumer chain` PoS operation and to receive rewards or `slash` from the `consumer chain`.
The `Staker` need to submit his staking information and identity when bonding.
And Consumer Chain PoS accepts or rejects a bond request according to its rules, such as \$NEAR > Th, NFT ownership, etc.

![](images/bond.png)

### Change ID

The `Staker` can change his `identity` after finish bonding. 

![](images/change_identity.png)

### Unbond

The `staker` can call the `unbond` operation to exit the `CC PoS`, which must go through the `unbonding period` specified by the `consumer chain` before completion.

And once unbonded, the staker can’t bond to the consumer chain until the unbonding period expired.

![](images/unbond.png)

### Blackout

The `CC PoS` can blackout Stakers, which will prevent them from bonding.

![](images/blackout.png)

### Slash

`Slash` is a penalty operation that is initiated by the `CC PoS` to punish a `staker` for misconduct. 

The process is divided into two steps: first, the `CC PoS`  submits a `slash` to the `staking-base` contract, and then `governance` decides whether to execute the `slash`.

Rules of executing slash:

1. It will slash on the asset in `Staker.pending_unstakes` with the smallest `unlock_time` first.
2. If the assets in `Staker.pending_unstakes` are not sufficient for the `slash` amount, the `unstake` operation is executed first, and `Staker.pending_unstakes` is updated before continuing with the `slash` operation.
3. If the assets are still not enough after the `unstake` operation, it is still considered a successful `slash`, and the total amount successfully `slashed` is returned.
4. After the assets in `Staker.pending_unstakes` are `slashed`, a `PendingUnstake` is created for the `CC PoS`'s `treasury` that was specified during registration. The `PendingUnstake.unlock_epoch` field inherits the `unlock_epoch` of the `PendingUnstake` that was `slashed`, and `PendingUnstake.unlock_time` is set to current_time.

![](images/submit_slash.png)
![](images/do_slash.png)

### Query VS

In order to update validator set, the `CC PoS` is motivated to query the Restaking Base contract periodically.

The `CC PoS` can specify a `limit` parameter to indicate that it only selects a certain number of Stakers ranked by staking amount. And the `CC PoS` will still perform an additional filtering based on the Stakers' staking amount to determine if they will become their validator.

![](images/queryVS.png)

### Unstake

When a `Staker` performs the `unstake` operation, they must first `Unbond` all `consumer chain` PoS they are currently bonding.

The withdrawable time after `Unstake`depends on the longest `Unbonding period` among all bonding `CC PoS`.
![](images/unstake.png)

### Withdraw

When a Staker performs the unstake or decrease stake operation, the contract will generate PendingUnstake data as a withdrawal voucher. When the Staker comes to withdraw, they need to specify the list of PendingUnstake IDs. The `restaking-base contract` will destroy the Withdrawable PendingUnstake and transfer the corresponding NEAR to the Staker.

![](images/withdraw.png)

## DataStruct and Interfaces
![](images/datastruct_and_interfaces.png)
![](images/cc-anchor.png)