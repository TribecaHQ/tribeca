//! Voter which locks up governance tokens for a user-provided duration in exchange for increased voting power.
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

pub mod macros;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use govern::{Governor, Proposal, Vote};
use vipers::*;

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
    pub fn new_locker(ctx: Context<NewLocker>, bump: u8, params: LockerParams) -> ProgramResult {
        ctx.accounts.new_locker(bump, params)
    }

    /// Creates a new [Escrow] for an account.
    ///
    /// A Vote Escrow, or [Escrow] for short, is an agreement between an account (known as the `authority`) and the DAO to
    /// lock up tokens for a specific period of time, in exchange for voting rights
    /// linearly proportional to the amount of votes given.
    #[access_control(ctx.accounts.validate())]
    pub fn new_escrow(ctx: Context<NewEscrow>, bump: u8) -> ProgramResult {
        ctx.accounts.new_escrow(bump)
    }

    /// Stakes `amount` tokens into the [Escrow].
    #[access_control(ctx.accounts.validate())]
    pub fn lock<'info>(
        ctx: Context<'_, '_, '_, 'info, Lock<'info>>,
        amount: u64,
        duration: i64,
    ) -> ProgramResult {
        if ctx.accounts.locker.params.whitelist_enabled {
            ctx.accounts.check_whitelisted(ctx.remaining_accounts)?;
        }

        ctx.accounts.lock(amount, duration)
    }

    /// Exits the DAO; i.e., withdraws all staked tokens in an [Escrow] if the [Escrow] is unlocked.
    #[access_control(ctx.accounts.validate())]
    pub fn exit(ctx: Context<Exit>) -> ProgramResult {
        ctx.accounts.exit()
    }

    /// Activates a proposal.
    #[access_control(ctx.accounts.validate())]
    pub fn activate_proposal(ctx: Context<ActivateProposal>) -> ProgramResult {
        ctx.accounts.activate_proposal()
    }

    /// Casts a vote.
    #[access_control(ctx.accounts.validate())]
    pub fn cast_vote(ctx: Context<CastVote>, side: u8) -> ProgramResult {
        ctx.accounts.cast_vote(side)
    }

    /// Set locker params.
    #[access_control(ctx.accounts.validate())]
    pub fn set_locker_params(ctx: Context<SetLockerParams>, params: LockerParams) -> ProgramResult {
        ctx.accounts.set_locker_params(params)
    }

    /// Creates a new [LockerWhitelistEntry] to whitelist program from CPI.
    #[access_control(ctx.accounts.validate())]
    pub fn approve_program_lock_privilege(
        ctx: Context<ApproveProgramLockPrivilege>,
        bump: u8,
    ) -> ProgramResult {
        ctx.accounts.approve_program_lock_privilege(bump)
    }

    /// Close a [LockerWhitelistEntry] revoking program's CPI privilege.
    #[access_control(ctx.accounts.validate())]
    pub fn revoke_program_lock_privilege(
        ctx: Context<RevokeProgramLockPrivilege>,
    ) -> ProgramResult {
        ctx.accounts.revoke_program_lock_privilege()
    }
}

/// [locked_voter] errors.
#[error]
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
}
