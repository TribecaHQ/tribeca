//! Events for the [govern] program.

use crate::*;

/// Event called in [govern::create_governor].
#[event]
pub struct GovernorCreateEvent {
    /// The governor being created.
    #[index]
    pub governor: Pubkey,
    /// The electorate of the created [Governor].
    pub electorate: Pubkey,
    /// The [SmartWallet].
    pub smart_wallet: Pubkey,
    /// Governance parameters.
    pub parameters: GovernanceParameters,
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

/// Event called in [govern::cancel_proposal].
#[event]
pub struct ProposalActivateEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being activated.
    #[index]
    pub proposal: Pubkey,
    /// When voting ends for the [Proposal].
    pub voting_ends_at: i64,
}

/// Event called in [govern::cancel_proposal].
#[event]
pub struct ProposalCancelEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being canceled.
    #[index]
    pub proposal: Pubkey,
}

/// Event called in [govern::queue_proposal].
#[event]
pub struct ProposalQueueEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being queued.
    #[index]
    pub proposal: Pubkey,
    /// The transaction key.
    #[index]
    pub transaction: Pubkey,
}

/// Event called in [govern::set_vote].
#[event]
pub struct VoteSetEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being voted on.
    #[index]
    pub proposal: Pubkey,
    /// The voter.
    #[index]
    pub voter: Pubkey,
    /// The vote.
    #[index]
    pub vote: Pubkey,
    /// The vote side.
    #[index]
    pub side: u8,
    /// The vote's weight.
    pub weight: u64,
}

/// Event called in [govern::create_proposal_meta].
#[event]
pub struct ProposalMetaCreateEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being voted on.
    #[index]
    pub proposal: Pubkey,
    /// The title.
    pub title: String,
    /// The description.
    pub description_link: String,
}

/// Event called in [govern::set_governance_params].
#[event]
pub struct GovernorSetParamsEvent {
    /// The governor being created.
    #[index]
    pub governor: Pubkey,
    /// Previous [GovernanceParameters].
    pub prev_params: GovernanceParameters,
    /// New [GovernanceParameters].
    pub params: GovernanceParameters,
}

/// Event called in [govern::set_electorate].
#[event]
pub struct GovernorSetElectorateEvent {
    /// The governor being created.
    #[index]
    pub governor: Pubkey,
    /// Previous [Governor::electorate].
    pub prev_electorate: Pubkey,
    /// New [Governor::electorate].
    pub new_electorate: Pubkey,
}
