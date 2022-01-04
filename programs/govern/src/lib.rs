//! Handles proposals, voting, and queueing of transactions into a [Smart Wallet](https://docs.tribeca.so/goki/smart-wallet).
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

use anchor_lang::prelude::*;
use num_traits::cast::ToPrimitive;
use smart_wallet::SmartWallet;
use std::convert::TryInto;
use vipers::invariant;
use vipers::unwrap_int;
use vipers::Validate;

mod account_structs;
mod account_validators;
mod events;
mod macros;
pub mod proposal;
mod state;

use account_structs::*;

pub use events::*;
pub use proposal::*;
pub use state::*;

declare_id!("Govz1VyoyLD5BL6CSCxUJLVLsQHRwjfFj1prNsdNg5Jw");

/// The [govern] program.
#[program]
pub mod govern {
    use super::*;

    /// Creates a [Governor].
    #[access_control(ctx.accounts.validate())]
    pub fn create_governor(
        ctx: Context<CreateGovernor>,
        bump: u8,
        electorate: Pubkey,
        params: GovernanceParameters,
    ) -> ProgramResult {
        invariant!(
            params.timelock_delay_seconds >= 0,
            "timelock delay must be at least 0 seconds"
        );

        let governor = &mut ctx.accounts.governor;
        governor.base = ctx.accounts.base.key();
        governor.bump = bump;

        governor.proposal_count = 0;
        governor.electorate = electorate;
        governor.smart_wallet = ctx.accounts.smart_wallet.key();

        governor.params = params;

        emit!(GovernorCreateEvent {
            governor: governor.key(),
            electorate,
            smart_wallet: ctx.accounts.smart_wallet.key(),
            parameters: params,
        });

        Ok(())
    }

    /// Creates a [Proposal].
    /// This may be called by anyone, since the [Proposal] does not do anything until
    /// it is activated in [activate_proposal].
    #[access_control(ctx.accounts.validate())]
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

    /// Activates a proposal.
    /// Only the [Governor::electorate] may call this; that program
    /// may ensure that only certain types of users can activate proposals.
    #[access_control(ctx.accounts.validate())]
    pub fn activate_proposal(ctx: Context<ActivateProposal>) -> ProgramResult {
        let proposal = &mut ctx.accounts.proposal;
        let now = Clock::get()?.unix_timestamp;
        proposal.activated_at = now;
        proposal.voting_ends_at = unwrap_int!(ctx
            .accounts
            .governor
            .params
            .voting_period
            .to_i64()
            .and_then(|v: i64| now.checked_add(v)));

        emit!(ProposalActivateEvent {
            governor: proposal.governor.key(),
            proposal: proposal.key(),
            voting_ends_at: proposal.voting_ends_at,
        });

        Ok(())
    }

    /// Cancels a proposal.
    /// This is only callable by the creator of the proposal.
    #[access_control(ctx.accounts.validate())]
    pub fn cancel_proposal(ctx: Context<CancelProposal>) -> ProgramResult {
        let proposal = &mut ctx.accounts.proposal;
        proposal.canceled_at = Clock::get()?.unix_timestamp;

        emit!(ProposalCancelEvent {
            governor: proposal.governor.key(),
            proposal: proposal.key(),
        });

        Ok(())
    }

    /// Queues a proposal for execution by the [SmartWallet].
    #[access_control(ctx.accounts.validate())]
    pub fn queue_proposal(ctx: Context<QueueProposal>, tx_bump: u8) -> ProgramResult {
        ctx.accounts.queue_transaction(tx_bump)?;

        emit!(ProposalQueueEvent {
            governor: ctx.accounts.proposal.governor.key(),
            proposal: ctx.accounts.proposal.key(),
            transaction: ctx.accounts.transaction.key(),
        });

        Ok(())
    }

    /// Creates a new [Vote]. Anyone can call this.
    #[access_control(ctx.accounts.validate())]
    pub fn new_vote(ctx: Context<NewVote>, bump: u8, voter: Pubkey) -> ProgramResult {
        let vote = &mut ctx.accounts.vote;
        vote.proposal = ctx.accounts.proposal.key();
        vote.voter = voter;
        vote.bump = bump;

        vote.side = VoteSide::Pending.into();
        vote.weight = 0;

        Ok(())
    }

    /// Sets a [Vote] weight and side.
    /// This may only be called by the [Governor::electorate].
    #[access_control(ctx.accounts.validate())]
    pub fn set_vote(ctx: Context<SetVote>, side: u8, weight: u64) -> ProgramResult {
        let vote = &ctx.accounts.vote;

        let proposal = &mut ctx.accounts.proposal;
        proposal.subtract_vote_weight(vote.side.try_into()?, vote.weight)?;
        proposal.add_vote_weight(side.try_into()?, weight)?;

        let vote = &mut ctx.accounts.vote;
        vote.side = side;
        vote.weight = weight;

        emit!(VoteSetEvent {
            governor: proposal.governor.key(),
            proposal: proposal.key(),
            voter: vote.voter,
            vote: vote.key(),
            side,
            weight,
        });

        Ok(())
    }

    /// Sets the [GovernanceParameters].
    /// This may only be called by the [Governor::smart_wallet].
    #[access_control(ctx.accounts.validate())]
    pub fn set_governance_params(
        ctx: Context<SetGovernanceParams>,
        params: GovernanceParameters,
    ) -> ProgramResult {
        let prev_params = ctx.accounts.governor.params;
        ctx.accounts.governor.params = params;

        emit!(GovernorSetParamsEvent {
            governor: ctx.accounts.governor.key(),
            prev_params,
            params,
        });

        Ok(())
    }

    /// Sets the electorate of the [Governor].
    #[access_control(ctx.accounts.validate())]
    pub fn set_electorate(
        ctx: Context<SetGovernanceParams>,
        new_electorate: Pubkey,
    ) -> ProgramResult {
        let prev_electorate = ctx.accounts.governor.electorate;
        ctx.accounts.governor.electorate = new_electorate;

        emit!(GovernorSetElectorateEvent {
            governor: ctx.accounts.governor.key(),
            prev_electorate,
            new_electorate,
        });

        Ok(())
    }

    /// Creates a [ProposalMeta].
    #[access_control(ctx.accounts.validate())]
    pub fn create_proposal_meta(
        ctx: Context<CreateProposalMeta>,
        _bump: u8,
        title: String,
        description_link: String,
    ) -> ProgramResult {
        let proposal_meta = &mut ctx.accounts.proposal_meta;
        proposal_meta.proposal = ctx.accounts.proposal.key();
        proposal_meta.title = title.clone();
        proposal_meta.description_link = description_link.clone();

        emit!(ProposalMetaCreateEvent {
            governor: ctx.accounts.proposal.governor.key(),
            proposal: ctx.accounts.proposal.key(),
            title,
            description_link,
        });

        Ok(())
    }
}

/// Errors.
#[error]
pub enum ErrorCode {
    #[msg("Invalid vote side.")]
    InvalidVoteSide,
    #[msg("The owner of the smart wallet doesn't match with current.")]
    GovernorNotFound,
    #[msg("The proposal cannot be activated since it has not yet passed the voting delay.")]
    VotingDelayNotMet,
    #[msg("Only drafts can be canceled.")]
    ProposalNotDraft,
    #[msg("The proposal must be active.")]
    ProposalNotActive,
}
