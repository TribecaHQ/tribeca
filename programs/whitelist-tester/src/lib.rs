use anchor_lang::prelude::*;
use anchor_spl::token::*;
use locked_voter::{program::LockedVoter, Escrow, Locker};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod whitelist_tester {
    use super::*;

    pub fn lock_tokens<'info>(
        ctx: Context<'_, '_, '_, 'info, LockTokens<'info>>,
        amount: u64,
        duration: i64,
    ) -> ProgramResult {
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
}

#[derive(Accounts)]
pub struct LockTokens<'info> {
    /// [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    /// [Escrow::tokens].
    #[account(mut)]
    pub escrow_tokens: Account<'info, TokenAccount>,

    /// Authority of the [Escrow] and of the [Self::source_tokens].
    pub escrow_owner: Signer<'info>,

    /// Source of the locked tokens.
    #[account(mut)]
    pub source_tokens: Account<'info, TokenAccount>,

    /// Token program.
    pub locked_voter_program: Program<'info, LockedVoter>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}
