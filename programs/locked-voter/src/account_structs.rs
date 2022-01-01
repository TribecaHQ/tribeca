//! Accounts structs for the [locked_voter] program.

use crate::*;
use govern::{Proposal, Vote};

/// Accounts for [locked_voter::new_locker].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewLocker<'info> {
    /// Base.
    pub base: Signer<'info>,

    /// [Locker].
    #[account(
        init,
        seeds = [
            b"Locker".as_ref(),
            base.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub locker: Account<'info, Locker>,

    /// Mint of the token that can be used to join the [Locker].
    pub token_mint: Account<'info, Mint>,

    /// [Governor] associated with the [Locker].
    pub governor: Account<'info, Governor>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [locked_voter::new_escrow].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewEscrow<'info> {
    /// [Locker].
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(
        init,
        seeds = [
            b"Escrow".as_ref(),
            locker.key().to_bytes().as_ref(),
            escrow_owner.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub escrow: Account<'info, Escrow>,

    /// Authority of the [Escrow] to be created.
    pub escrow_owner: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [locked_voter::lock].
#[derive(Accounts)]
pub struct Lock<'info> {
    /// [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    /// Token account held by the [Escrow].
    #[account(mut)]
    pub escrow_tokens: Account<'info, TokenAccount>,

    /// Authority of the [Escrow] and [Self::source_tokens].
    pub escrow_owner: Signer<'info>,

    /// The source of deposited tokens.
    #[account(mut)]
    pub source_tokens: Account<'info, TokenAccount>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for [locked_voter::exit].
#[derive(Accounts)]
pub struct Exit<'info> {
    /// The [Locker] being exited from.
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// The [Escrow] that is being closed.
    #[account(mut, close = payer)]
    pub escrow: Account<'info, Escrow>,

    /// Authority of the [Escrow].
    pub escrow_owner: Signer<'info>,
    /// Tokens locked up in the [Escrow].
    #[account(mut)]
    pub escrow_tokens: Account<'info, TokenAccount>,
    /// Destination for the tokens to unlock.
    #[account(mut)]
    pub destination_tokens: Account<'info, TokenAccount>,

    /// The payer to receive the rent refund.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for [locked_voter::activate_proposal].
#[derive(Accounts)]
pub struct ActivateProposal<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal].
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The user's [Escrow].
    pub escrow: Account<'info, Escrow>,
    /// The [Escrow]'s owner.
    pub escrow_owner: Signer<'info>,
    /// The [govern] program.
    pub govern_program: Program<'info, govern::program::Govern>,
}

/// Accounts for [locked_voter::cast_vote].
#[derive(Accounts)]
pub struct CastVote<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// The [Escrow] that is voting.
    pub escrow: Account<'info, Escrow>,
    /// Vote delegate of the [Escrow].
    pub vote_delegate: Signer<'info>,

    /// The [Proposal] being voted on.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [Vote].
    #[account(mut)]
    pub vote: Account<'info, Vote>,

    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [govern] program.
    pub govern_program: Program<'info, govern::program::Govern>,
}

/// Accounts for [locked_voter::cast_vote].
#[derive(Accounts)]
pub struct SetLockerParams<'info> {
    /// The [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The smart wallet on the [Governor].
    pub smart_wallet: Signer<'info>,
}

/// Accounts for [locked_voter::approve_program_lock_privilege].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ApproveProgramLockPrivilege<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// [LockerWhitelistEntry].
    #[account(
        init,
        seeds = [
            b"LockerWhitelistEntry".as_ref(),
            locker.key().to_bytes().as_ref(),
            executable_id.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub whitelist_entry: Account<'info, LockerWhitelistEntry>,

    /// Governor for the [Locker].
    pub governor: Account<'info, Governor>,

    /// Smart wallet on the [Governor].
    pub smart_wallet: Signer<'info>,

    /// ProgramId of the program to whitelist.
    pub executable_id: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [locked_voter::revoke_program_lock_privilege].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct RevokeProgramLockPrivilege<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// [LockerWhitelistEntry].
    #[account(mut, close = payer)]
    pub whitelist_entry: Account<'info, LockerWhitelistEntry>,

    /// Governor for the [Locker].
    pub governor: Account<'info, Governor>,

    /// Smart wallet on the [Governor].
    pub smart_wallet: Signer<'info>,

    /// ProgramId of the program to whitelist.
    pub executable_id: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,
}
