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

impl<'info> Validate<'info> for ActivateProposal<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.governor, self.proposal.governor);
        assert_keys_eq!(self.electorate, self.governor.electorate);
        invariant!(
            self.proposal.get_state()? == ProposalState::Draft,
            ProposalNotDraft
        );

        let earliest_activation_time = unwrap_int!(self
            .governor
            .params
            .voting_delay
            .checked_add(self.proposal.created_at as u64));
        let now = Clock::get()?.unix_timestamp as u64;
        if earliest_activation_time > now {
            msg!(
                "Earliest activation time {}; now: {}",
                earliest_activation_time,
                now
            );
            invariant!(now >= earliest_activation_time, VotingDelayNotMet);
        }

        Ok(())
    }
}