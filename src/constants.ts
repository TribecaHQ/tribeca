import { buildCoderMap } from "@saberhq/anchor-contrib";
import { PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";

import { GovernJSON } from "./idls/govern";
import { LockedVoterJSON } from "./idls/locked_voter";
import { SimpleVoterJSON } from "./idls/simple_voter";
import type {
  GovernanceParameters,
  GovernProgram,
  GovernTypes,
  LockedVoterProgram,
  LockedVoterTypes,
  LockerParams,
  SimpleVoterProgram,
  SimpleVoterTypes,
} from "./programs";

/**
 * Tribeca program types.
 */
export interface TribecaPrograms {
  SimpleVoter: SimpleVoterProgram;
  Govern: GovernProgram;
  LockedVoter: LockedVoterProgram;
}

// See `Anchor.toml` for all addresses.
export const TRIBECA_ADDRESSES = {
  SimpleVoter: new PublicKey("Tok6iuA69RLN1QrpXgQKnDgE1YYbLzQsZGSoz75fQdz"),
  Govern: new PublicKey("Govz1VyoyLD5BL6CSCxUJLVLsQHRwjfFj1prNsdNg5Jw"),
  LockedVoter: new PublicKey("LocktDzaV1W2Bm9DeZeiyz4J9zs4fRqNiYqQyracRXw"),
};

/**
 * Program IDLs.
 */
export const TRIBECA_IDLS = {
  SimpleVoter: SimpleVoterJSON,
  Govern: GovernJSON,
  LockedVoter: LockedVoterJSON,
};

/**
 * Coders.
 */
export const TRIBECA_CODERS = buildCoderMap<{
  SimpleVoter: SimpleVoterTypes;
  Govern: GovernTypes;
  LockedVoter: LockedVoterTypes;
}>(TRIBECA_IDLS, TRIBECA_ADDRESSES);

export const DEFAULT_DECIMALS = 6;

export const ONE_DAY = new BN(24 * 60 * 60);
export const ONE_YEAR = new BN(365).mul(ONE_DAY);

// Default number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
// ~ 4% of 10 billion
export const DEFAULT_QUORUM_VOTES = new BN(10000000000 * 0.04).mul(
  new BN(10).pow(new BN(DEFAULT_DECIMALS))
);
// Default number of votes required in order for a voter to become a proposer
// ~ 1% of 10 billion
export const DEFAULT_PROPOSAL_THRESHOLD = new BN(10000000000 * 0.01).mul(
  new BN(10).pow(new BN(DEFAULT_DECIMALS))
);
// Default delay before voting on a proposal may take place, once proposed, ~ 1 second
export const DEFAULT_VOTE_DELAY = new BN(1);
// Default duration of voting on a proposal, in seconds, ~ 3 days
export const DEFAULT_VOTE_PERIOD = new BN(3).mul(ONE_DAY);

/**
 * Default parameters for a Governor.
 */
export const DEFAULT_GOVERNANCE_PARAMETERS: GovernanceParameters = {
  timelockDelaySeconds: new BN(0),
  quorumVotes: DEFAULT_QUORUM_VOTES,
  votingDelay: DEFAULT_VOTE_DELAY,
  votingPeriod: DEFAULT_VOTE_PERIOD,
};

/**
 * Default parameters for a Locker.
 */
export const DEFAULT_LOCKER_PARAMS: LockerParams = {
  // 1M tokens if max locked.
  proposalActivationMinVotes: new BN(10_000_000 * 10 ** 6),
  // 1 day.
  minStakeDuration: ONE_DAY,
  // 5 years.
  maxStakeDuration: new BN(5).mul(ONE_YEAR),
  maxStakeVoteMultiplier: 10,
  whitelistEnabled: false,
};
