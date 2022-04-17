use anchor_lang::prelude::*;
use anchor_spl::token::*;
use locked_voter::{program::LockedVoter, Escrow, Locker, LockerWhitelistEntry};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod whitelist_tester {
    use super::*;

    pub fn lock_tokens<'info>(
        ctx: Context<'_, '_, '_, 'info, LockTokens<'info>>,
        amount: u64,
        duration: i64,
    ) -> Result<()> {
        let cpi_accounts = locked_voter::cpi::accounts::Lock {
            locker: ctx.accounts.locker.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
            escrow_tokens: ctx.accounts.escrow_tokens.to_account_info(),
            escrow_owner: ctx.accounts.escrow_owner.to_account_info(),
            source_tokens: ctx.accounts.source_tokens.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.locked_voter_program.to_account_info(),
            cpi_accounts,
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        locked_voter::cpi::lock(cpi_context, amount, duration)
    }

    pub fn lock_tokens_with_whitelist_entry<'info>(
        ctx: Context<'_, '_, '_, 'info, LockTokensWithWhitelistEntry<'info>>,
        amount: u64,
        duration: i64,
    ) -> Result<()> {
        let cpi_accounts = locked_voter::cpi::accounts::LockWithWhitelistEntry {
            lock: locked_voter::cpi::accounts::Lock {
                locker: ctx.accounts.locker.to_account_info(),
                escrow: ctx.accounts.escrow.to_account_info(),
                escrow_tokens: ctx.accounts.escrow_tokens.to_account_info(),
                escrow_owner: ctx.accounts.escrow_owner.to_account_info(),
                source_tokens: ctx.accounts.source_tokens.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            instructions_sysvar: ctx.accounts.instructions_sysvar.to_account_info(),
            whitelist_entry: ctx.accounts.whitelist_entry.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.locked_voter_program.to_account_info(),
            cpi_accounts,
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        locked_voter::cpi::lock_with_whitelist_entry(cpi_context, amount, duration)
    }
}

#[derive(Accounts)]
pub struct LockTokensWithWhitelistEntry<'info> {
    /// [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(mut, has_one = locker)]
    pub escrow: Account<'info, Escrow>,

    /// [Escrow::tokens].
    #[account(mut, constraint = escrow.tokens == escrow_tokens.key())]
    pub escrow_tokens: Account<'info, TokenAccount>,

    /// Authority of the [Escrow] and of the [Self::source_tokens].
    #[account(constraint = escrow.owner == escrow_owner.key())]
    pub escrow_owner: Signer<'info>,

    /// Source of the locked tokens.
    #[account(mut, constraint = source_tokens.mint == locker.token_mint && source_tokens.owner == escrow_owner.key())]
    pub source_tokens: Account<'info, TokenAccount>,

    /// Token program.
    pub locked_voter_program: Program<'info, LockedVoter>,

    /// Token program.
    pub token_program: Program<'info, Token>,

    /// CHECK: The instructions Sysvar.
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    /// Whitelist entry.
    #[account(has_one = locker, constraint = whitelist_entry.program_id == crate::ID)]
    pub whitelist_entry: Account<'info, LockerWhitelistEntry>,
}

#[derive(Accounts)]
pub struct LockTokens<'info> {
    /// [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(mut, has_one = locker)]
    pub escrow: Account<'info, Escrow>,

    /// [Escrow::tokens].
    #[account(mut, constraint = escrow.tokens == escrow_tokens.key())]
    pub escrow_tokens: Account<'info, TokenAccount>,

    /// Authority of the [Escrow] and of the [Self::source_tokens].
    #[account(constraint = escrow.owner == escrow_owner.key())]
    pub escrow_owner: Signer<'info>,

    /// Source of the locked tokens.
    #[account(mut, constraint = source_tokens.mint == locker.token_mint && source_tokens.owner == escrow_owner.key())]
    pub source_tokens: Account<'info, TokenAccount>,

    /// Token program.
    pub locked_voter_program: Program<'info, LockedVoter>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}
