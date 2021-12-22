import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";

import type { GovernorWrapper } from "./governor";

export type PendingGovernor = {
  wrapper: GovernorWrapper;
  tx: TransactionEnvelope;
};

export type WhitelistArgs = {
  canPropose: boolean;
  activator: PublicKey;
  smartWalletOwner?: PublicKey;
};

/**
 * State of a proposal.
 */
export enum ProposalState {
  /*
   * Anyone can create a proposal on Tribeca. When a governance proposal is created,
   * it is considered a [ProposalState::Draft] and enters a review period, after which voting weights
   * are recorded and voting begins.
   */
  Draft,
  /*
   * Each DAO has requirements for who can activate proposals; a common way
   * is to require the user to have a minimum amount of tokens.
   * An [ProposalState::Active] proposal is one that is surfaced to the community to put up for voting.
   */
  Active,
  /*
   * If a proposal is still a [ProposalState::Draft], a proposal may be canceled by its creator.
   * A canceled proposal cannot be reactivated; it simply just exists as a record.
   */
  Canceled,
  /*
   * After the voting period ends, votes are tallied up. A proposal is [ProposalState::Defeated] if one of
   * two scenarios happen:
   * - More or equal votes are [VoteSide::Against] than [VoteSide::For].
   * - The sum of all votes does not meet quorum.
   */
  Defeated,
  /*
   * A proposal is [ProposalState::Succeeded] if it is not defeated and voting is over.
   */
  Succeeded,
  /*
   * A succeeded proposal may be [ProposalState::Queued] into the [SmartWallet].
   */
  Queued,
}

/**
 * Labels for proposal states.
 */
export const PROPOSAL_STATE_LABELS: { [K in ProposalState]: string } = {
  [ProposalState.Active]: "Active",
  [ProposalState.Draft]: "Draft",
  [ProposalState.Canceled]: "Canceled",
  [ProposalState.Defeated]: "Defeated",
  [ProposalState.Succeeded]: "Succeeded",
  [ProposalState.Queued]: "Queued",
} as const;

/**
 * Side of a vote.
 */
export enum VoteSide {
  /**
   * A vote that has not been set or has been unset.
   */
  Pending = 0,
  /**
   * Vote against the passing of the proposal.
   */
  Against = 1,
  /**
   * Vote to make the proposal pass.
   */
  For = 2,
  /**
   * This vote does not count as a `For` or `Against`, but it still contributes to quorum.
   */
  Abstain = 3,
}

/**
 * Labels for vote sides.
 */
export const VOTE_SIDE_LABELS: { [K in VoteSide]: string } = {
  [VoteSide.For]: "For",
  [VoteSide.Against]: "Against",
  [VoteSide.Abstain]: "Abstain",
  [VoteSide.Pending]: "Pending",
};
