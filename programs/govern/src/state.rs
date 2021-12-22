//! Struct definitions for accounts that hold state.

use anchor_lang::prelude::*;

/// A Governor is the "DAO": it is the account that holds control over important protocol functions,
/// including treasury, protocol parameters, and more.
#[account]
#[derive(Copy, Debug, Default)]
pub struct Governor {
    /// Base.
    pub base: Pubkey,
    /// Bump seed
    pub bump: u8,

    /// The total number of [Proposal]s
    pub proposal_count: u64,
    /// The voting body associated with the Governor.
    /// This account is responsible for handling vote proceedings, such as:
    /// - activating proposals
    /// - setting the number of votes per voter
    pub electorate: Pubkey,
    /// The public key of the [smart_wallet::SmartWallet] account.
    /// This smart wallet executes proposals.
    pub smart_wallet: Pubkey,

    /// Governance parameters.
    pub params: GovernanceParameters,
}

/// Governance parameters.
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct GovernanceParameters {
    /// The delay before voting on a proposal may take place, once proposed, in seconds
    pub voting_delay: u64,
    /// The duration of voting on a proposal, in seconds
    pub voting_period: u64,
    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: u64,
    /// The timelock delay of the DAO's created proposals.
    pub timelock_delay_seconds: i64,
}

/// A Proposal is a pending transaction that may or may not be executed by the DAO.
#[account]
#[derive(Debug, Default)]
pub struct Proposal {
    /// The public key of the governor.
    pub governor: Pubkey,
    /// The unique ID of the proposal, auto-incremented.
    pub index: u64,
    /// Bump seed
    pub bump: u8,

    /// The public key of the proposer.
    pub proposer: Pubkey,

    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: u64,
    /// Current number of votes in favor of this proposal
    pub for_votes: u64,
    /// Current number of votes in opposition to this proposal
    pub against_votes: u64,
    /// Current number of votes for abstaining for this proposal
    pub abstain_votes: u64,

    /// The timestamp when the proposal was canceled.
    pub canceled_at: i64,
    /// The timestamp when the proposal was created.
    pub created_at: i64,
    /// The timestamp in which the proposal was activated.
    /// This is when voting begins.
    pub activated_at: i64,
    /// The timestamp when voting ends.
    /// This only applies to active proposals.
    pub voting_ends_at: i64,

    /// The timestamp in which the proposal was queued, i.e.
    /// approved for execution on the Smart Wallet.
    pub queued_at: i64,
    /// If the transaction was queued, this is the associated Goki Smart Wallet transaction.
    pub queued_transaction: Pubkey,

    /// The instructions associated with the proposal.
    pub instructions: Vec<ProposalInstruction>,
}

impl Proposal {
    /// Space that the [Proposal] takes up.
    pub fn space(instructions: Vec<ProposalInstruction>) -> usize {
        4  // Anchor discriminator.
        + 4 // Vec discriminator
            + std::mem::size_of::<Proposal>()
            + (instructions.iter().map(|ix| ix.space()).sum::<usize>())
    }
}

/// Metadata about a proposal.
#[account]
#[derive(Debug, Default)]
pub struct ProposalMeta {
    /// The [Proposal].
    pub proposal: Pubkey,
    /// Title of the proposal.
    pub title: String,
    /// Link to a description of the proposal.
    pub description_link: String,
}

/// A [Vote] is a vote made by a `voter` by an `electorate`.
#[account]
#[derive(Debug, Default)]
pub struct Vote {
    /// The proposal being voted on.
    pub proposal: Pubkey,
    /// The voter.
    pub voter: Pubkey,
    /// Bump seed
    pub bump: u8,

    /// The side of the vote taken.
    pub side: u8,
    /// The number of votes this vote holds.
    pub weight: u64,
}

/// Instruction.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ProposalInstruction {
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Pubkey,
    /// Metadata for what accounts should be passed to the instruction processor
    pub keys: Vec<ProposalAccountMeta>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

impl ProposalInstruction {
    /// Space that a [ProposalInstruction] takes up.
    pub fn space(&self) -> usize {
        std::mem::size_of::<Pubkey>()
            + (self.keys.len() as usize) * std::mem::size_of::<AccountMeta>()
            + (self.data.len() as usize)
    }
}

/// Account metadata used to define Instructions
#[derive(AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Copy, Clone)]
pub struct ProposalAccountMeta {
    /// An account's public key
    pub pubkey: Pubkey,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}
