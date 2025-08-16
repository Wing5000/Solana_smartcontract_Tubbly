# Tubbly Solana Program

This repository contains an [Anchor](https://github.com/coral-xyz/anchor) based
program that manages balance requests and confirmations on the Solana
blockchain.

## Program overview

The on-chain program exposes a set of instructions that let an owner manage
token-like balances for users:

- **initialize** – sets the initial owner of the program and resets the request
  counter
- **submit** – users create a pending request with an identifier and an amount
- **confirm** – the owner approves a request and credits the user's balance
  before clearing the request
- **balance_of** – returns the stored balance for a user account
- **get_request** – owner can view the details of a pending request
  (identifier, caller, amount, and active flag)
- **change_ownership** – allows the current owner to transfer control to a new
  public key

Key account types include:

- `State` – stores the owner public key and a request counter
- `Request` – tracks submitted requests
- `UserAccount` – holds per-user balances

The program also emits events such as `OwnershipChanged`, `Submission`, and
`Confirmation` to make state transitions observable.

## Development

1. Install dependencies:

   ```bash
   npm install
   ```

2. Build the program:

   ```bash
   anchor build
   ```

3. Run tests:

   ```bash
   anchor test
   ```

The tests in `tests/` use Anchor's Mocha wrapper and expect a local Solana
cluster to be running.

