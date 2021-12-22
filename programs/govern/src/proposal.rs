//! Proposal logic.

use std::convert::TryFrom;

use crate::*;
use vipers::{program_err, unwrap_int, unwrap_opt};

/// The state of a proposal.
///
/// The `expired` state from Compound is missing here, because the
/// Smart Wallet handles execution.
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ProposalState {
    /// Anyone can create a proposal on Tribeca. When a governance proposal is created,
    /// it is considered a [ProposalState::Draft] and enters a review period, after which voting weights
    /// are recorded and voting begins.
    Draft,
    /// Each DAO has requirements for who can activate proposals; a common way
    /// is to require the user to have a minimum amount of tokens.
    /// An [ProposalState::Active] proposal is one that is surfaced to the community to put up for voting.
    Active,
    /// If a proposal is still a [ProposalState::Draft], a proposal may be canceled by its creator.
    /// A canceled proposal cannot be reactivated; it simply just exists as a record.
    Canceled,
    /// After the voting period ends, votes are tallied up. A proposal is [ProposalState::Defeated] if one of
    /// two scenarios happen:
    /// - More or equal votes are [VoteSide::Against] than [VoteSide::For].
    /// - The sum of all votes does not meet quorum.
    Defeated,
    /// A proposal is [ProposalState::Succeeded] if it is not defeated and voting is over.
    Succeeded,
    /// A succeeded proposal may be [ProposalState::Queued] into the [SmartWallet].
    Queued,
}

/// Side of a vote.
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum VoteSide {
    /// A vote that has not been set or has been unset.
    Pending = 0,
    /// Vote against the passing of the proposal.
    Against = 1,
    /// Vote to make the proposal pass.
    For = 2,
    /// This vote does not count as a `For` or `Against`, but it still contributes to quorum.
    Abstain = 3,
}

impl Default for VoteSide {
    fn default() -> Self {
        VoteSide::Pending
    }
}

impl From<VoteSide> for u8 {
    fn from(side: VoteSide) -> Self {
        side as u8
    }
}

impl TryFrom<u8> for VoteSide {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(VoteSide::Pending),
            1 => Ok(VoteSide::Against),
            2 => Ok(VoteSide::For),
            3 => Ok(VoteSide::Abstain),
            _ => program_err!(InvalidVoteSide),
        }
    }
}

impl Default for ProposalState {
    fn default() -> Self {
        Self::Draft
    }
}

impl Proposal {
    /// Subtracts from the total weight of a vote for a [Proposal].
    pub(crate) fn subtract_vote_weight(
        &mut self,
        vote_side: VoteSide,
        vote_weight: u64,
    ) -> ProgramResult {
        if vote_weight == 0 {
            return Ok(());
        }
        match vote_side {
            VoteSide::Pending => {}
            VoteSide::Against => {
                self.against_votes = unwrap_int!(self.against_votes.checked_sub(vote_weight));
            }
            VoteSide::For => {
                self.for_votes = unwrap_int!(self.for_votes.checked_sub(vote_weight));
            }
            VoteSide::Abstain => {
                self.abstain_votes = unwrap_int!(self.abstain_votes.checked_sub(vote_weight));
            }
        }
        Ok(())
    }

    /// Adds to the total weight of a vote for a [Proposal].
    pub(crate) fn add_vote_weight(
        &mut self,
        vote_side: VoteSide,
        vote_weight: u64,
    ) -> ProgramResult {
        if vote_weight == 0 {
            return Ok(());
        }
        match vote_side {
            VoteSide::Pending => {}
            VoteSide::Against => {
                self.against_votes = unwrap_int!(self.against_votes.checked_add(vote_weight));
            }
            VoteSide::For => {
                self.for_votes = unwrap_int!(self.for_votes.checked_add(vote_weight));
            }
            VoteSide::Abstain => {
                self.abstain_votes = unwrap_int!(self.abstain_votes.checked_add(vote_weight));
            }
        }
        Ok(())
    }

    /// Gets the state.
    pub fn get_state(&self) -> Result<ProposalState> {
        Ok(unwrap_opt!(
            self.state(Clock::get()?.unix_timestamp),
            "invalid state"
        ))
    }

    /// Checks if the proposal meets quorum; that is,
    /// enough votes were made on the proposal.
    pub fn meets_quorum(&self, quorum_votes: u64) -> Option<bool> {
        Some(
            self.for_votes
                .checked_add(self.against_votes)?
                .checked_add(self.abstain_votes)?
                >= quorum_votes,
        )
    }

    /// The state of the proposal. See [ProposalState] for more details.
    /// Adapted from <https://github.com/compound-finance/compound-protocol/blob/4a8648ec0364d24c4ecfc7d6cae254f55030d65f/contracts/Governance/GovernorBravoDelegate.sol#L205>
    pub fn state(&self, current_time: i64) -> Option<ProposalState> {
        if self.canceled_at > 0 {
            return Some(ProposalState::Canceled);
        } else if self.activated_at == 0 {
            return Some(ProposalState::Draft);
        } else if current_time < self.voting_ends_at {
            return Some(ProposalState::Active);
        } else if self.for_votes <= self.against_votes || !self.meets_quorum(self.quorum_votes)? {
            return Some(ProposalState::Defeated);
        } else if self.queued_at > 0 {
            return Some(ProposalState::Queued);
        }
        Some(ProposalState::Succeeded)
    }

    /// Converts this proposal to Smart Wallet [smart_wallet::TXInstruction]s.
    pub fn to_smart_wallet_instructions(&self) -> Vec<smart_wallet::TXInstruction> {
        self.instructions
            .iter()
            .map(
                |ProposalInstruction {
                     program_id,
                     keys,
                     data,
                 }| smart_wallet::TXInstruction {
                    program_id: *program_id,
                    keys: keys
                        .iter()
                        .map(
                            |&ProposalAccountMeta {
                                 pubkey,
                                 is_signer,
                                 is_writable,
                             }| smart_wallet::TXAccountMeta {
                                pubkey,
                                is_signer,
                                is_writable,
                            },
                        )
                        .collect(),
                    data: data.clone(),
                },
            )
            .collect()
    }
}

impl<'info> QueueProposal<'info> {
    /// Queues a Transaction into the Smart Wallet.
    pub fn queue_transaction(&mut self, tx_bump: u8) -> ProgramResult {
        let seeds = governor_seeds!(self.governor);
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.smart_wallet_program.to_account_info(),
            smart_wallet::cpi::accounts::CreateTransaction {
                smart_wallet: self.smart_wallet.to_account_info(),
                transaction: self.transaction.to_account_info(),
                proposer: self.governor.to_account_info(),
                payer: self.payer.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
            signer_seeds,
        );

        // no delay
        if self.governor.params.timelock_delay_seconds == 0 {
            smart_wallet::cpi::create_transaction(
                cpi_ctx,
                tx_bump,
                self.proposal.to_smart_wallet_instructions(),
            )?;
        } else {
            // delay; calculate ETA
            smart_wallet::cpi::create_transaction_with_timelock(
                cpi_ctx,
                tx_bump,
                self.proposal.to_smart_wallet_instructions(),
                unwrap_int!(Clock::get()?
                    .unix_timestamp
                    .checked_add(self.governor.params.timelock_delay_seconds)),
            )?;
        }

        let proposal = &mut self.proposal;
        proposal.queued_at = Clock::get()?.unix_timestamp;
        proposal.queued_transaction = self.transaction.key();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Maximum seconds elapsed between two checkpoints.
    /// [i32::MAX] corresponds to about 70 years.
    const MAX_SECONDS_BETWEEN_CHECKPOINTS: i64 = i32::MAX as i64;
    const MAX_TOTAL_TOKENS: u64 = u64::MAX / 1000_u64;

    prop_compose! { pub fn total_and_intermediate_ts()(
          elapsed_seconds in 0..MAX_SECONDS_BETWEEN_CHECKPOINTS,
          last_checkpoint_ts in 0..(i64::MAX - MAX_SECONDS_BETWEEN_CHECKPOINTS),
        ) -> (i64, i64) {
          (last_checkpoint_ts + elapsed_seconds, last_checkpoint_ts)
       }
    }

    prop_compose! {
        pub fn part_and_total()(
            total in 0..MAX_TOTAL_TOKENS
        )(
            // use a really small number here
            part in 0..total,
            total in Just(total)
        ) -> (u64, u64) {
           (part, total)
       }
    }

    #[derive(Default)]
    struct TestProposalParams {
        pub canceled_at: i64,
        pub current_ts: i64,
        pub activated_at: i64,
        pub created_at: i64,
        pub voting_ends_at: i64,
        pub queued_at: i64,
        pub abstain_votes: u64,
        pub against_votes: u64,
        pub for_votes: u64,
        pub quorum_votes: u64,
    }

    fn test_proposal_state(t: TestProposalParams) -> ProposalState {
        let proposal = Proposal {
            for_votes: t.for_votes,
            against_votes: t.against_votes,
            abstain_votes: t.abstain_votes,
            canceled_at: t.canceled_at,
            created_at: t.created_at,
            activated_at: t.activated_at,
            voting_ends_at: t.voting_ends_at,
            queued_at: t.queued_at,
            quorum_votes: t.quorum_votes,
            ..Proposal::default()
        };

        proposal.state(t.current_ts).unwrap()
    }

    #[test]
    fn test_draft_state() {
        let params = TestProposalParams {
            activated_at: 0,
            ..TestProposalParams::default()
        };
        assert_eq!(test_proposal_state(params), ProposalState::Draft);
    }

    proptest! {
        #[test]
        fn test_cancelled_state(
            canceled_at in 1..=i64::MAX,
        ) {
            let params = TestProposalParams {
                canceled_at,
                ..TestProposalParams::default()
            };
            assert_eq!(test_proposal_state(params), ProposalState::Canceled);
        }
    }

    proptest! {
        #[test]
        fn test_active_state(
            activated_at in 1..=i64::MAX,
            (voting_ends_at, current_ts) in total_and_intermediate_ts(),
        ) {
            let params = TestProposalParams {
                current_ts,
                activated_at,
                voting_ends_at,
                ..TestProposalParams::default()
            };
            assert_eq!(test_proposal_state(params), ProposalState::Active);
        }
    }

    proptest! {
        #[test]
        fn test_defeated_state(
            activated_at in 1..=i64::MAX,
            (for_votes, against_votes) in part_and_total(),
            (current_ts, voting_ends_at) in total_and_intermediate_ts(),
        ) {
            let params = TestProposalParams {
                current_ts,
                activated_at,
                voting_ends_at,
                for_votes,
                against_votes,
                ..TestProposalParams::default()
            };
            assert_eq!(test_proposal_state(params), ProposalState::Defeated);
        }
    }

    proptest! {
        #[test]
        fn test_not_meet_quorum(
            activated_at in 1..=i64::MAX,
            (all_votes, quorum_votes) in part_and_total(),
            (current_ts, voting_ends_at) in total_and_intermediate_ts(),
            for_shares in 1..=3u64,
            against_shares in 1..=3u64,
            abstain_shares in 1..=3u64,
        ) {

            let total_shares = for_shares + against_shares + abstain_shares;
            let for_votes = all_votes * for_shares / total_shares;
            let against_votes = all_votes * against_shares / total_shares;
            let abstain_votes = all_votes * abstain_shares / total_shares;
            let params = TestProposalParams {
                current_ts,
                activated_at,
                voting_ends_at,
                for_votes,
                against_votes,
                abstain_votes,
                quorum_votes,
                ..TestProposalParams::default()
            };
            let proposal = Proposal {
                for_votes,
                against_votes,
                abstain_votes,
                ..Proposal::default()
            };
            assert!(!proposal.meets_quorum(quorum_votes).unwrap(), "proposal should fail quorum; for_votes: {}, against_votes: {}, abstain: votes: {}", for_votes, against_votes, abstain_votes);
            assert_eq!(test_proposal_state(params), ProposalState::Defeated);
        }
    }

    proptest! {
        #[test]
        fn test_queued_state(
            activated_at in 1..=i64::MAX,
            queued_at in 1..i64::MAX,
            (quorum_votes, for_votes) in part_and_total(),
            (current_ts, voting_ends_at) in total_and_intermediate_ts(),
        ) {
            let params = TestProposalParams {
                activated_at,
                current_ts,
                for_votes,
                quorum_votes,
                voting_ends_at,
                queued_at,
                ..TestProposalParams::default()
            };
            assert_eq!(test_proposal_state(params), ProposalState::Queued);
        }
    }

    proptest! {
        #[test]
        fn test_success_state(
            activated_at in 1..=i64::MAX,
            (quorum_votes, for_votes) in part_and_total(),
            (current_ts, voting_ends_at) in total_and_intermediate_ts(),
        ) {
            let params = TestProposalParams {
                activated_at,
                current_ts,
                for_votes,
                quorum_votes,
                voting_ends_at,
                ..TestProposalParams::default()
            };
            assert_eq!(test_proposal_state(params), ProposalState::Succeeded);
        }
    }
}
