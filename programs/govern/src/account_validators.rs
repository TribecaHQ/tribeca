//! Validates accounts structs.

use crate::*;
use vipers::{assert_keys_eq, invariant, unwrap_int, unwrap_opt, Validate};

impl<'info> Validate<'info> for CreateGovernor<'info> {
    fn validate(&self) -> ProgramResult {
        invariant!(
            self.smart_wallet.owners.contains(&self.governor.key()),
            GovernorNotFound
        );

        Ok(())
    }
}

impl<'info> Validate<'info> for CreateProposal<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
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

impl<'info> Validate<'info> for QueueProposal<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.governor,
            self.proposal.governor,
            "proposal should be under the governor"
        );
        assert_keys_eq!(
            self.smart_wallet,
            self.governor.smart_wallet,
            "smart wallet should match"
        );
        let now = Clock::get()?.unix_timestamp;
        let proposal_state = unwrap_opt!(self.proposal.state(now), "invalid state");
        if proposal_state != ProposalState::Succeeded {
            msg!(
                "now: {}, voting_ends_at: {}",
                now,
                self.proposal.voting_ends_at
            );
            msg!(
                "for votes: {}, against votes: {}",
                self.proposal.for_votes,
                self.proposal.against_votes,
            );
            msg!(
                "quorum req: {}, abstain votes: {}",
                self.governor.params.quorum_votes,
                self.proposal.abstain_votes,
            );
            invariant!(
                proposal_state == ProposalState::Succeeded,
                "proposal must be succeeded to be queued"
            );
        }
        Ok(())
    }
}

impl<'info> Validate<'info> for NewVote<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

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
