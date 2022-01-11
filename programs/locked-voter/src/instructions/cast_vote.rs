use crate::*;
use govern::ProposalState;

/// Accounts for [locked_voter::cast_vote].
#[derive(Accounts)]
pub struct CastVote<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// The [Escrow] that is voting.
    pub escrow: Account<'info, Escrow>,
    /// Vote delegate of the [Escrow].
    pub vote_delegate: Signer<'info>,

    /// The [Proposal] being voted on.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [Vote].
    #[account(mut)]
    pub vote: Account<'info, Vote>,

    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [govern] program.
    pub govern_program: Program<'info, govern::program::Govern>,
}

impl<'info> CastVote<'info> {
    pub fn cast_vote(&mut self, side: u8) -> ProgramResult {
        let voting_power = self.future_voting_power()?;

        // zero votes should short circuit.
        if voting_power == 0 {
            return Ok(());
        }

        let seeds: &[&[&[u8]]] = locker_seeds!(self.locker);
        let cpi_ctx = CpiContext::new(
            self.govern_program.to_account_info(),
            govern::cpi::accounts::SetVote {
                governor: self.governor.to_account_info(),
                proposal: self.proposal.to_account_info(),
                vote: self.vote.to_account_info(),
                electorate: self.locker.to_account_info(),
            },
        )
        .with_signer(seeds);

        govern::cpi::set_vote(cpi_ctx, side, voting_power)?;
        Ok(())
    }

    /// The voting power of the escrow at the time the proposal's voting ends.
    fn future_voting_power(&self) -> Result<u64> {
        Ok(unwrap_int!(self.escrow.voting_power_at_time(
            &self.locker.params,
            self.proposal.voting_ends_at
        )))
    }
}

impl<'info> Validate<'info> for CastVote<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.escrow.locker, self.locker);
        assert_keys_eq!(self.escrow.vote_delegate, self.vote_delegate);
        assert_keys_eq!(self.locker.governor, self.governor);
        assert_keys_eq!(self.proposal.governor, self.governor);
        assert_keys_eq!(self.vote.proposal, self.proposal);
        assert_keys_eq!(self.vote.voter, self.escrow.owner);
        invariant!(
            self.proposal.get_state()? == ProposalState::Active,
            "proposal must be active"
        );
        Ok(())
    }
}
