//! Accounts structs for the [govern] program.

use crate::*;

/// Accounts for [govern::create_governor].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateGovernor<'info> {
    /// Base of the [Governor] key.
    pub base: Signer<'info>,
    /// Governor.
    #[account(
        init,
        seeds = [
            b"TribecaGovernor".as_ref(),
            base.key().as_ref()
        ],
        bump = bump,
        payer = payer,
    )]
    pub governor: Account<'info, Governor>,
    /// The Smart Wallet.
    pub smart_wallet: Account<'info, SmartWallet>,
    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [govern::create_proposal].
#[derive(Accounts)]
#[instruction(bump: u8, instructions: Vec<ProposalInstruction>)]
pub struct CreateProposal<'info> {
    /// The [Governor].
    #[account(mut)]
    pub governor: Account<'info, Governor>,
    /// The [Proposal].
    #[account(
        init,
        seeds = [
            b"TribecaProposal".as_ref(),
            governor.key().as_ref(),
            governor.proposal_count.to_le_bytes().as_ref()
        ],
        bump = bump,
        payer = payer,
        space = Proposal::space(instructions),
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    /// Proposer of the proposal.
    pub proposer: Signer<'info>,
    /// Payer of the proposal.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [govern::activate_proposal].
#[derive(Accounts)]
pub struct ActivateProposal<'info> {
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal] to activate.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The electorate of the [Governor] that may activate the proposal.
    pub electorate: Signer<'info>,
}

/// Accounts for [govern::cancel_proposal].
#[derive(Accounts)]
pub struct CancelProposal<'info> {
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal] to activate.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [Proposal::proposer].
    pub proposer: Signer<'info>,
}

/// Accounts for [govern::queue_proposal].
#[derive(Accounts)]
pub struct QueueProposal<'info> {
    /// The Governor.
    pub governor: Account<'info, Governor>,
    /// The Proposal to queue.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The transaction key of the proposal.
    #[account(mut)]
    pub transaction: UncheckedAccount<'info>,
    /// The Smart Wallet.
    #[account(mut)]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// Payer of the queued transaction.
    pub payer: Signer<'info>,
    /// The Smart Wallet program.
    pub smart_wallet_program: Program<'info, smart_wallet::program::SmartWallet>,
    /// The System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [govern::new_vote].
#[derive(Accounts)]
#[instruction(bump: u8, voter: Pubkey)]
pub struct NewVote<'info> {
    /// Proposal being voted on.
    pub proposal: Account<'info, Proposal>,

    /// The vote.
    #[account(
        init,
        seeds = [
            b"TribecaVote".as_ref(),
            proposal.key().as_ref(),
            voter.as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub vote: Account<'info, Vote>,

    /// Payer of the [Vote].
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

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
