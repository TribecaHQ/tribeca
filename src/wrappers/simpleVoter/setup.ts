import type { SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import type BN from "bn.js";

import type { GovernorWrapper } from "../..";
import { DEFAULT_PROPOSAL_THRESHOLD } from "../..";
import type { CreateGovernorWithElectorateParams } from "../govern/setup";
import { createGovernorWithElectorate } from "../govern/setup";
import { SimpleVoterWrapper } from ".";

/**
 * Creates a new simple electorate.
 */
export interface CreateSimpleElectorateParams
  extends Omit<CreateGovernorWithElectorateParams, "createElectorate"> {
  /**
   * Mint of the token staked for veTokens.
   */
  govTokenMint: PublicKey;
  /**
   * Proposal threshold.
   */
  proposalThreshold?: BN;
  /**
   * Base of the electorate.
   */
  electorateBaseKP?: Signer;
}

/**
 * Creates a new Locker.
 * @returns
 */
export const createSimpleElectorate = async ({
  sdk,
  govTokenMint,
  proposalThreshold = DEFAULT_PROPOSAL_THRESHOLD,
  electorateBaseKP = Keypair.generate(),
  ...createGovernorParams
}: CreateSimpleElectorateParams): Promise<{
  governorWrapper: GovernorWrapper;
  smartWalletWrapper: SmartWalletWrapper;
  simpleVoterWrapper: SimpleVoterWrapper;
  createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[];
}> => {
  const { electorate, ...governor } = await createGovernorWithElectorate({
    createElectorate: async (governorKey) => {
      const { electorate, tx } = await sdk.createSimpleElectorate({
        baseKP: electorateBaseKP,
        proposalThreshold,
        governor: governorKey,
        govTokenMint,
      });
      return {
        key: electorate,
        tx,
      };
    },
    sdk,
    ...createGovernorParams,
  });
  return {
    ...governor,
    simpleVoterWrapper: new SimpleVoterWrapper(
      sdk,
      electorate,
      governor.governorWrapper.governorKey
    ),
  };
};
