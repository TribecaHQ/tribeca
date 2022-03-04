//! Voter which locks up governance tokens for a user-provided duration in exchange for increased voting power.
//!
//! Detailed documentation is available on the [Tribeca documentation site.](https://docs.tribeca.so/voting-escrow)
//!
//! # License
//!
//! Tribeca Protocol is licensed under the GNU Affero General Public License v3.0.
//!
//! In short, this means that any changes to this code must be made open source and
//! available under the AGPL-v3.0 license, even if only used privately. If you have
//! a need to use this program and cannot respect the terms of the license, please
//! message us our team directly at [team@tribeca.so](mailto:team@tribeca.so).
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

pub mod macros;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use govern::{Governor, Proposal, Vote};
use vipers::prelude::*;

mod instructions;
pub mod locker;
mod state;

pub use instructions::*;
pub use state::*;

declare_id!("LocktDzaV1W2Bm9DeZeiyz4J9zs4fRqNiYqQyracRXw");

/// Locked voter program.
#[deny(missing_docs)]
#[program]
pub mod locked_voter {
    use super::*;

    /// Creates a new [Locker].
    #[access_control(ctx.accounts.validate())]
    pub fn new_locker(ctx: Context<NewLocker>, _bump: u8, params: LockerParams) -> Result<()> {
        ctx.accounts
            .new_locker(*unwrap_int!(ctx.bumps.get("locker")), params)
    }

    /// Creates a new [Escrow] for an account.
    ///
    /// A Vote Escrow, or [Escrow] for short, is an agreement between an account (known as the `authority`) and the DAO to
    /// lock up tokens for a specific period of time, in exchange for voting rights
    /// linearly proportional to the amount of votes given.
    #[access_control(ctx.accounts.validate())]
    pub fn new_escrow(ctx: Context<NewEscrow>, _bump: u8) -> Result<()> {
        ctx.accounts
            .new_escrow(*unwrap_int!(ctx.bumps.get("escrow")))
    }

    /// Stakes `amount` tokens into the [Escrow].
    #[access_control(ctx.accounts.validate())]
    pub fn lock<'info>(
        ctx: Context<'_, '_, '_, 'info, Lock<'info>>,
        amount: u64,
        duration: i64,
    ) -> Result<()> {
        if ctx.accounts.locker.params.whitelist_enabled {
            ctx.accounts.check_whitelisted(ctx.remaining_accounts)?;
        }

        ctx.accounts.lock(amount, duration)
    }

    /// Exits the DAO; i.e., withdraws all staked tokens in an [Escrow] if the [Escrow] is unlocked.
    #[access_control(ctx.accounts.validate())]
    pub fn exit(ctx: Context<Exit>) -> Result<()> {
        ctx.accounts.exit()
    }

    /// Activates a proposal.
    #[access_control(ctx.accounts.validate())]
    pub fn activate_proposal(ctx: Context<ActivateProposal>) -> Result<()> {
        ctx.accounts.activate_proposal()
    }

    /// Casts a vote.
    #[access_control(ctx.accounts.validate())]
    pub fn cast_vote(ctx: Context<CastVote>, side: u8) -> Result<()> {
        ctx.accounts.cast_vote(side)
    }

    /// Delegate escrow vote.
    #[access_control(ctx.accounts.validate())]
    pub fn set_vote_delegate(ctx: Context<SetVoteDelegate>, new_delegate: Pubkey) -> Result<()> {
        ctx.accounts.set_vote_delegate(new_delegate)
    }

    /// Set locker params.
    #[access_control(ctx.accounts.validate())]
    pub fn set_locker_params(ctx: Context<SetLockerParams>, params: LockerParams) -> Result<()> {
        ctx.accounts.set_locker_params(params)
    }

    /// Creates a new [LockerWhitelistEntry] to whitelist program from CPI.
    #[access_control(ctx.accounts.validate())]
    pub fn approve_program_lock_privilege(
        ctx: Context<ApproveProgramLockPrivilege>,
        _bump: u8,
    ) -> Result<()> {
        ctx.accounts
            .approve_program_lock_privilege(*unwrap_int!(ctx.bumps.get("whitelist_entry")))
    }

    /// Close a [LockerWhitelistEntry] revoking program's CPI privilege.
    #[access_control(ctx.accounts.validate())]
    pub fn revoke_program_lock_privilege(ctx: Context<RevokeProgramLockPrivilege>) -> Result<()> {
        ctx.accounts.revoke_program_lock_privilege()
    }
}

/// [locked_voter] errors.
#[error_code]
pub enum ErrorCode {
    #[msg("CPI caller not whitelisted to invoke lock instruction.")]
    ProgramNotWhitelisted,
    #[msg("Lockup duration must at least be the min stake duration.")]
    LockupDurationTooShort,
    #[msg("Lockup duration must at most be the max stake duration.")]
    LockupDurationTooLong,
    #[msg("A voting escrow refresh cannot shorten the escrow time remaining.")]
    RefreshCannotShorten,
    #[msg("Escrow has not ended.")]
    EscrowNotEnded,
    #[msg("Program whitelist enabled; please provide whitelist entry and instructions sysvar")]
    MustProvideWhitelist,
    #[msg("CPI caller not whitelisted for escrow owner to invoke lock instruction.")]
    EscrowOwnerNotWhitelisted,
}
