import BN from "bn.js";

import type { ProposalData } from "../../programs/govern";
import { ProposalState } from "./types";

/**
 * Gets the state of a proposal.
 * @returns
 */
export const getProposalState = ({
  proposalData,
  currentTimeSeconds = Math.floor(new Date().getTime() / 1_000),
}: {
  proposalData: ProposalData;
  currentTimeSeconds?: number;
}): ProposalState => {
  if (proposalData.canceledAt.gt(new BN(0))) {
    return ProposalState.Canceled;
  } else if (proposalData.activatedAt.eq(new BN(0))) {
    return ProposalState.Draft;
  } else if (proposalData.votingEndsAt.gte(new BN(currentTimeSeconds))) {
    return ProposalState.Active;
  } else if (
    proposalData.forVotes.lte(proposalData.againstVotes) ||
    proposalData.forVotes
      .add(proposalData.abstainVotes)
      .add(proposalData.againstVotes)
      .lt(proposalData.quorumVotes)
  ) {
    return ProposalState.Defeated;
  } else if (proposalData.queuedAt.gt(new BN(0))) {
    return ProposalState.Queued;
  }
  return ProposalState.Succeeded;
};
