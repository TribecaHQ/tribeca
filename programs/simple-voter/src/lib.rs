//! A simple Tribeca voter program where 1 token = 1 vote.

use anchor_lang::prelude::*;
use anchor_spl::token::*;
use govern::{proposal::ProposalState, Governor, Proposal, Vote};
use vipers::*;

mod account_validators;
pub mod macros;
mod processor;
mod state;
mod token_cpi;

pub use state::*;

declare_id!("Tok6iuA69RLN1QrpXgQKnDgE1YYbLzQsZGSoz75fQdz");

#[program]
pub mod simple_voter {
    use super::*;

    #[access_control(ctx.accounts.validate())]
    pub fn initialize_electorate(
        ctx: Context<InitializeElectorate>,
        bump: u8,
        proposal_threshold: u64,
    ) -> ProgramResult {
        let electorate = &mut ctx.accounts.electorate;
        electorate.bump = bump;
        electorate.proposal_threshold = proposal_threshold;
        electorate.base = ctx.accounts.base.key();
        electorate.governor = ctx.accounts.governor.key();
        electorate.gov_token_mint = ctx.accounts.gov_token_mint.key();

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn initialize_token_record(ctx: Context<InitializeTokenRecord>, bump: u8) -> ProgramResult {
        let token_record = &mut ctx.accounts.token_record;
        token_record.bump = bump;
        token_record.balance = ctx.accounts.gov_token_vault.amount;
        token_record.authority = ctx.accounts.authority.key();
        token_record.electorate = ctx.accounts.electorate.key();
        token_record.token_vault_key = ctx.accounts.gov_token_vault.key();

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn activate_proposal(ctx: Context<ActivateProposal>) -> ProgramResult {
        processor::proposer::activate_proposal(ctx)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn deposit_tokens(ctx: Context<TokenContext>, amount: u64) -> ProgramResult {
        ctx.accounts.transfer_to_vault(amount)?;

        let token_record = &mut ctx.accounts.token_record;
        let vault = &mut ctx.accounts.gov_token_vault;
        vault.reload()?;
        token_record.balance = vault.amount;

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn withdraw_tokens(ctx: Context<TokenContext>, amount: u64) -> ProgramResult {
        ctx.accounts.transfer_from_vault(amount)?;

        let token_record = &mut ctx.accounts.token_record;
        invariant!(
            token_record.unfinalized_votes == 0,
            "some votes not finalized"
        );
        let vault = &mut ctx.accounts.gov_token_vault;
        vault.reload()?;
        token_record.balance = vault.amount;

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn cast_votes(ctx: Context<VoterContext>, vote_side: u8) -> ProgramResult {
        processor::voter::process_cast_votes(ctx, vote_side)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn withdraw_votes(ctx: Context<VoterContext>) -> ProgramResult {
        processor::voter::process_withdraw_votes(ctx)
    }

    pub fn finalize_votes(ctx: Context<FinalizeVote>) -> ProgramResult {
        invariant!(ctx.accounts.proposal.get_state()? != ProposalState::Active);
        let token_record = &mut ctx.accounts.token_record;
        token_record.unfinalized_votes -= 1;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeElectorate<'info> {
    /// Base used to create the voter.
    pub base: Signer<'info>,
    /// The electorate.
    #[account(
        init,
        seeds = [b"SimpleElectorate".as_ref(), base.key().to_bytes().as_ref()],
        bump = bump,
        payer = payer,
    )]
    pub electorate: Account<'info, Electorate>,
    /// TODO(michael): Docs
    pub governor: Account<'info, Governor>,
    /// TODO(michael): Docs
    pub gov_token_mint: Account<'info, Mint>,
    /// TODO(michael): Docs
    pub payer: AccountInfo<'info>,
    /// TODO(michael): Docs
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeTokenRecord<'info> {
    pub authority: Signer<'info>,
    /// TODO(michael): Docs
    #[account(
        init,
        seeds = [
            b"SimpleTokenRecord".as_ref(),
            authority.key().to_bytes().as_ref(),
            electorate.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer,
    )]
    pub token_record: Account<'info, state::TokenRecord>,
    #[account(mut)]
    pub electorate: Account<'info, state::Electorate>,
    /// TODO(michael): Docs
    pub gov_token_vault: Account<'info, TokenAccount>,
    /// TODO(michael): Docs
    pub payer: AccountInfo<'info>,
    /// TODO(michael): Docs
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FinalizeVote<'info> {
    /// TODO(michael): Docs
    pub authority: Signer<'info>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub governor: Account<'info, Governor>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub token_record: Account<'info, state::TokenRecord>,
}

#[derive(Accounts)]
pub struct TribecaContext<'info> {
    /// TODO(michael): Docs
    #[account(mut)]
    pub governor: Account<'info, Governor>,
    /// TODO(michael): Docs
    pub program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ActivateProposal<'info> {
    pub electorate: Account<'info, state::Electorate>,
    pub governor: Account<'info, Governor>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [govern] program.
    pub govern_program: Program<'info, govern::program::Govern>,
}

#[derive(Accounts)]
pub struct TokenContext<'info> {
    /// TODO(michael): Docs
    pub authority: Signer<'info>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub token_record: Account<'info, state::TokenRecord>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub gov_token_account: Account<'info, TokenAccount>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub gov_token_vault: Account<'info, TokenAccount>,
    /// TODO(michael): Docs
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VoterContext<'info> {
    /// The [Electorate].
    pub electorate: Account<'info, Electorate>,
    /// TODO(michael): Docs
    pub authority: Signer<'info>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub token_record: Account<'info, state::TokenRecord>,
    /// TODO(michael): Docs
    #[account(mut)]
    pub vote: Account<'info, Vote>,
    /// TODO(michael): Docs
    pub tribeca: TribecaContext<'info>,
}

#[error]
pub enum ErrorCode {
    #[msg("Below proposing threshold.")]
    BelowProposingThreshold,
}
