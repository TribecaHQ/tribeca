use crate::{electorate_seeds, ActivateProposal};
use anchor_lang::prelude::*;

pub fn activate_proposal(ctx: Context<ActivateProposal>) -> ProgramResult {
    let seeds: &[&[&[u8]]] = electorate_seeds!(ctx.accounts.electorate);
    govern::cpi::activate_proposal(
        CpiContext::new(
            ctx.accounts.govern_program.to_account_info(),
            ctx.accounts.to_activate_proposal_accounts(),
        )
        .with_signer(seeds),
    )
}

impl<'info> ActivateProposal<'info> {
    /// Conversion.
    pub fn to_activate_proposal_accounts(&self) -> govern::cpi::accounts::ActivateProposal<'info> {
        govern::cpi::accounts::ActivateProposal {
            governor: self.governor.to_account_info(),
            proposal: self.proposal.to_account_info(),
            electorate: self.electorate.to_account_info(),
        }
    }
}
