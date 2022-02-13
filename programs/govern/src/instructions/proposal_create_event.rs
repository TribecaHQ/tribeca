use crate::*;

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

impl<'info> CreateGovernor<'info> {
    /// Creates a [Proposal].
    /// This may be called by anyone, since the [Proposal] does not do anything until
    /// it is activated in [activate_proposal].
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        bump: u8,
        instructions: Vec<ProposalInstruction>,
    ) -> ProgramResult {
        let governor = &mut ctx.accounts.governor;

        let proposal = &mut ctx.accounts.proposal;
        proposal.governor = governor.key();
        proposal.index = governor.proposal_count;
        proposal.bump = bump;

        proposal.proposer = ctx.accounts.proposer.key();

        proposal.quorum_votes = governor.params.quorum_votes;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.canceled_at = 0;
        proposal.activated_at = 0;
        proposal.voting_ends_at = 0;

        proposal.queued_at = 0;
        proposal.queued_transaction = Pubkey::default();

        proposal.instructions = instructions.clone();

        governor.proposal_count += 1;

        emit!(ProposalCreateEvent {
            governor: governor.key(),
            proposal: proposal.key(),
            index: proposal.index,
            instructions,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for CreateProposal<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

/// Event called in [govern::create_proposal].
#[event]
pub struct ProposalCreateEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being created.
    #[index]
    pub proposal: Pubkey,
    /// The index of the [Proposal].
    pub index: u64,
    /// Instructions in the proposal.
    pub instructions: Vec<ProposalInstruction>,
}

