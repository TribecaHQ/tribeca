use crate::*;

/// Accounts for [locked_voter::activate_proposal].
#[derive(Accounts)]
pub struct ActivateProposal<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal].
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The user's [Escrow].
    pub escrow: Account<'info, Escrow>,
    /// The [Escrow]'s owner.
    pub escrow_owner: Signer<'info>,
    /// The [govern] program.
    pub govern_program: Program<'info, govern::program::Govern>,
}

impl<'info> ActivateProposal<'info> {
    /// Activates the proposal.
    pub fn activate_proposal(&mut self) -> ProgramResult {
        let seeds: &[&[&[u8]]] = locker_seeds!(self.locker);

        govern::cpi::activate_proposal(
            CpiContext::new(
                self.govern_program.to_account_info(),
                self.to_activate_proposal_accounts(),
            )
            .with_signer(seeds),
        )?;

        Ok(())
    }

    /// Conversion.
    fn to_activate_proposal_accounts(&self) -> govern::cpi::accounts::ActivateProposal<'info> {
        govern::cpi::accounts::ActivateProposal {
            governor: self.governor.to_account_info(),
            proposal: self.proposal.to_account_info(),
            electorate: self.locker.to_account_info(),
        }
    }

    /// The current voting power of the escrow.
    fn current_voting_power(&self) -> Result<u64> {
        self.escrow.voting_power(&self.locker.params)
    }
}

impl<'info> Validate<'info> for ActivateProposal<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.locker, self.governor.electorate);
        assert_keys_eq!(self.governor, self.locker.governor);
        assert_keys_eq!(self.proposal.governor, self.governor);
        assert_keys_eq!(self.escrow.locker, self.locker);
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);

        invariant!(
            self.current_voting_power()? >= self.locker.params.proposal_activation_min_votes,
            "insufficient voting power to activate a proposal"
        );

        Ok(())
    }
}
