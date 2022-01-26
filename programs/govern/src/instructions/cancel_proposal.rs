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

impl<'info> Validate<'info> for CancelProposal<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.proposer,
            self.proposal.proposer,
            "proposer should match recorded"
        );
        assert_keys_eq!(
            self.governor,
            self.proposal.governor,
            "proposal should be under the governor"
        );
        invariant!(
            self.proposal.get_state()? == ProposalState::Draft,
            ProposalNotDraft
        );
        Ok(())
    }
}