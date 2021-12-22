# govern

[![License](https://img.shields.io/badge/license-AGPL%203.0-blue)](https://github.com/TribecaHQ/tribeca/blob/master/LICENSE)
[![crates](https://img.shields.io/crates/v/govern)](https://crates.io/crates/govern)

Handles proposals, voting, and queueing of transactions into a Smart Wallet.

## Life of a Tribeca Proposal

### Draft

Anyone can create a proposal on Tribeca. When a governance proposal is created, it is considered a "draft" and enters a review period, after which voting weights are recorded and voting begins.

### Active

Each DAO has requirements for who can activate proposals; a common way is to require the user to have a minimum amount of tokens. An _activated_ proposal is one that is surfaced to the community to put up for voting.

### Outcome: Succeeded, Defeated

Voting lasts for a configurable period. If a majority, and at least `quorum` votes are cast for the proposal, it is queued in a Smart Wallet for execution by the other signers of the multisig.

### Executed

Once the transaction is queued in a Smart Wallet, the other signers on the Smart Wallet may execute the transaction. We suggest the following structure for the Smart Wallet:

- 2-of-3
- Signers:
  - Tribeca Governor
  - 1-of-n Smart Wallet of trusted parties that will execute a proposal without frontrunning. This may be composed of the core protocol team.
  - 4-of-7 Emergency DAO Smart Wallet that can quickly execute a proposal in case of emergency. This should be composed of trusted community members that will not collude.

## License

AGPL-3.0.
