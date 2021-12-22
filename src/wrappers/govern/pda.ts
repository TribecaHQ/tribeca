import { utils } from "@project-serum/anchor";
import type { u64 } from "@saberhq/token-utils";
import { PublicKey } from "@solana/web3.js";

import { TRIBECA_ADDRESSES } from "../../constants";

/**
 * Finds the PDA of a Governor.
 */
export const findGovernorAddress = async (
  base: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("TribecaGovernor"), base.toBuffer()],
    TRIBECA_ADDRESSES.Govern
  );
};

/**
 * Finds the PDA of a Proposal.
 */
export const findProposalAddress = async (
  governorKey: PublicKey,
  index: u64
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("TribecaProposal"),
      governorKey.toBuffer(),
      index.toArrayLike(Buffer, "le", 8),
    ],
    TRIBECA_ADDRESSES.Govern
  );
};

/**
 * Finds the PDA of a Vote.
 * @param proposalKey
 * @param voterKey
 * @returns
 */
export const findVoteAddress = async (
  proposalKey: PublicKey,
  voterKey: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("TribecaVote"),
      proposalKey.toBuffer(),
      voterKey.toBuffer(),
    ],
    TRIBECA_ADDRESSES.Govern
  );
};

/**
 * Finds the address of a ProposalMeta.
 * @param proposalKey
 * @returns
 */
export const findProposalMetaAddress = async (
  proposalKey: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("TribecaProposalMeta"), proposalKey.toBuffer()],
    TRIBECA_ADDRESSES.Govern
  );
};
