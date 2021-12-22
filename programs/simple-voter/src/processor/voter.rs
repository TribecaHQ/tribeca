use anchor_lang::prelude::*;

use crate::{electorate_seeds, VoterContext};

pub fn process_cast_votes(ctx: Context<VoterContext>, vote_side: u8) -> ProgramResult {
    let seeds: &[&[&[u8]]] = electorate_seeds!(ctx.accounts.electorate);
    let cpi_ctx = CpiContext::new(
        ctx.accounts.tribeca.program.to_account_info(),
        ctx.accounts.to_set_vote_accounts(),
    )
    .with_signer(seeds);
    govern::cpi::set_vote(cpi_ctx, vote_side, ctx.accounts.token_record.balance)?;

    let token_record = &mut ctx.accounts.token_record;
    token_record.unfinalized_votes += 1;

    Ok(())
}

pub fn process_withdraw_votes(ctx: Context<VoterContext>) -> ProgramResult {
    let seeds: &[&[&[u8]]] = electorate_seeds!(ctx.accounts.electorate);
    let cpi_ctx = CpiContext::new(
        ctx.accounts.tribeca.program.clone(),
        ctx.accounts.to_set_vote_accounts(),
    )
    .with_signer(seeds);
    govern::cpi::set_vote(cpi_ctx, ctx.accounts.vote.side, 0)?;

    let token_record = &mut ctx.accounts.token_record;
    token_record.unfinalized_votes -= 1;

    Ok(())
}

impl<'info> VoterContext<'info> {
    /// Conversion.
    pub fn to_set_vote_accounts(&self) -> govern::cpi::accounts::SetVote<'info> {
        govern::cpi::accounts::SetVote {
            governor: self.tribeca.governor.to_account_info(),
            proposal: self.proposal.to_account_info(),
            vote: self.vote.to_account_info(),
            electorate: self.electorate.to_account_info(),
        }
    }
}
