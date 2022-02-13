//! Validates accounts structs.

use crate::*;
use vipers::{assert_keys_eq, invariant, unwrap_int, unwrap_opt, Validate};

impl<'info> Validate<'info> for SetVote<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.governor.electorate, self.electorate);
        assert_keys_eq!(
            self.governor,
            self.proposal.governor,
            "proposal should be under the governor"
        );
        assert_keys_eq!(
            self.vote.proposal,
            self.proposal,
            "vote proposal should match"
        );
        invariant!(
            self.proposal.get_state()? == ProposalState::Active,
            ProposalNotActive
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for CreateProposalMeta<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.proposer, self.proposal.proposer);
        Ok(())
    }
}

impl<'info> Validate<'info> for SetGovernanceParams<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.smart_wallet,
            self.governor.smart_wallet,
            "smart wallet should match"
        );
        Ok(())
    }
}
