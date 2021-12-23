# ♜ Tribeca

[![License](https://img.shields.io/badge/license-AGPL%203.0-blue)](https://github.com/TribecaHQ/tribeca/blob/master/LICENSE)
[![Build Status](https://img.shields.io/github/workflow/status/TribecaHQ/tribeca/E2E/master)](https://github.com/TribecaHQ/tribeca/actions/workflows/programs-e2e.yml?query=branch%3Amaster)
[![Contributors](https://img.shields.io/github/contributors/TribecaHQ/tribeca)](https://github.com/TribecaHQ/tribeca/graphs/contributors)

<p align="center">
    <img src="/images/banner.png" />
</p>

<p align="center">
    An open standard and toolkit for launching DAOs on Solana.
</p>

- [Website](https://tribeca.so)
- [Twitter](https://twitter.com/TribecaDAO)
- [GitHub](https://github.com/TribecaHQ)
- [Official documentation](https://docs.tribeca.so/)

## About

Tribeca is an open source protocol for launching decentralized autonomous organizations on Solana. It is heavily inspired by the designs of [Compound](https://compound.finance/docs/governance) and [Curve](https://curve.readthedocs.io/dao-vecrv.html) governance.

- **Built with Anchor.** Tribeca is pluggable with other Anchor-based projects and tools, allowing for greater composability.
- **Modularity.** Tribeca is built to be flexible and extensible, splitting out proposals/voting, transaction execution, and staking into three separate programs.
- **Hosted.** The Tribeca DAO program is meant to be deployed as one instance of a program, similar to the [SPL Token Program](https://spl.solana.com/token). It is an open standard that any other project can build upon, without requiring teams to deploy their own DAO or giving custody of user tokens to Tribeca.
- **Simplicity.** Tribeca does as little as possible, giving DAOs the freedom to choose how their governance system operates.

For the most up-to-date documentation, please visit the [official documentation site.](https://docs.tribeca.so/)

## Note

- **Tribeca is in active development, so all APIs are subject to change.**
- **This code is unaudited. Use at your own risk.**

## Packages

| Package                  | Description                                                                                                               | Version                                                                                                                 | Docs                                                                                  |
| :----------------------- | :------------------------------------------------------------------------------------------------------------------------ | :---------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ |
| `govern`                 | Handles proposals, voting, and queueing of transactions into a [Smart Wallet](https://docs.tribeca.so/goki/smart-wallet). | [![Crates.io](https://img.shields.io/crates/v/govern)](https://crates.io/crates/govern)                                 | [![Docs.rs](https://docs.rs/govern/badge.svg)](https://docs.rs/govern)                |
| `locked-voter`           | Voter which locks up governance tokens for a user-provided duration in exchange for increased voting power.               | [![crates](https://img.shields.io/crates/v/locked-voter)](https://crates.io/crates/locked-voter)                        | [![Docs.rs](https://docs.rs/locked-voter/badge.svg)](https://docs.rs/locked-voter)    |
| `simple-voter`           | A simple Tribeca voter program where 1 token = 1 vote.                                                                    | [![crates](https://img.shields.io/crates/v/simple-voter)](https://crates.io/crates/simple-voter)                        | [![Docs.rs](https://docs.rs/simple-voter/badge.svg)](https://docs.rs/simple-voter)    |
| `@tribecahq/tribeca-sdk` | TypeScript SDK for Tribeca                                                                                                | [![npm](https://img.shields.io/npm/v/@tribecahq/tribeca-sdk.svg)](https://www.npmjs.com/package/@tribecahq/tribeca-sdk) | [![Docs](https://img.shields.io/badge/docs-typedoc-blue)](https://docs.quarry.so/ts/) |

## Addresses

Program addresses are the same on devnet and mainnet-beta.

## Documentation

The official documentation is hosted at [docs.tribeca.so.](https://docs.tribeca.so/).

## License

Tribeca Protocol is licensed under the GNU Affero General Public License v3.0.

In short, this means that any changes to this code must be made open source and available under the AGPL-v3.0 license, even if only used privately. If you have a need to use this program and cannot respect the terms of the license, please message us our team directly at [team@tribeca.so](mailto:team@tribeca.so).
