//! Accounts structs for the [govern] program.

use crate::*;

/// Accounts for [govern::set_vote].
#[derive(Accounts)]
pub struct SetVote<'info> {
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal].
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [Vote].
    #[account(mut)]
    pub vote: Account<'info, Vote>,
    /// The [Governor::electorate].
    pub electorate: Signer<'info>,
}

/// Accounts for [govern::create_proposal_meta].
#[derive(Accounts)]
#[instruction(bump: u8, title: String, description_link: String)]
pub struct CreateProposalMeta<'info> {
    /// The [Proposal].
    pub proposal: Box<Account<'info, Proposal>>,
    /// Proposer of the proposal.
    pub proposer: Signer<'info>,
    /// The [ProposalMeta].
    #[account(
        init,
        seeds = [
            b"TribecaProposalMeta".as_ref(),
            proposal.key().as_ref()
        ],
        bump = bump,
        payer = payer,
        space = 4 + std::mem::size_of::<ProposalMeta>()
            + 4 + title.as_bytes().len()
            + 4 + description_link.as_bytes().len()
    )]
    pub proposal_meta: Box<Account<'info, ProposalMeta>>,
    /// Payer of the [ProposalMeta].
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [govern::set_governance_params] and [govern::set_electorate].
#[derive(Accounts)]
pub struct SetGovernanceParams<'info> {
    /// The [Governor]
    #[account(mut)]
    pub governor: Account<'info, Governor>,
    /// The Smart Wallet.
    pub smart_wallet: Signer<'info>,
}
