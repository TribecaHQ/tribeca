import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";
import type BN from "bn.js";

import type { VoteSide } from "../govern/types";

export type PendingProposal = {
  proposal: PublicKey;
  index: BN;
  tx: TransactionEnvelope;
};

export type PendingElectorate = {
  electorate: PublicKey;
  tx: TransactionEnvelope;
};

export type VoteArgs = {
  voteSide: VoteSide;
  proposal: PublicKey;
  authority?: PublicKey;
  reason?: string | undefined;
};
