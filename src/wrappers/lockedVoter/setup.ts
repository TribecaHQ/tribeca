import type { SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";

import type { GovernorWrapper, LockerParams } from "../..";
import { DEFAULT_LOCKER_PARAMS } from "../../constants";
import type { CreateGovernorWithElectorateParams } from "../govern/setup";
import { createGovernorWithElectorate } from "../govern/setup";
import { LockerWrapper } from "./locker";

export interface CreateLockerParams
  extends Omit<CreateGovernorWithElectorateParams, "createElectorate"> {
  /**
   * Mint of the token staked for veTokens.
   */
  govTokenMint: PublicKey;
  /**
   * Parameters for the locker.
   */
  lockerParams?: Partial<LockerParams>;
  /**
   * Base of the locker.
   */
  lockerBaseKP?: Signer;
}

/**
 * Creates a new Locker.
 * @returns
 */
export const createLocker = async ({
  sdk,
  govTokenMint,
  lockerParams = DEFAULT_LOCKER_PARAMS,
  lockerBaseKP = Keypair.generate(),
  ...createGovernorParams
}: CreateLockerParams): Promise<{
  governorWrapper: GovernorWrapper;
  smartWalletWrapper: SmartWalletWrapper;
  lockerWrapper: LockerWrapper;
  createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[];
}> => {
  const { electorate, ...governor } = await createGovernorWithElectorate({
    createElectorate: async (governorKey) => {
      const { locker, tx: tx1 } = await sdk.createLocker({
        ...lockerParams,
        baseKP: lockerBaseKP,
        governor: governorKey,
        govTokenMint,
      });
      return {
        key: locker,
        tx: tx1,
      };
    },
    sdk,
    ...createGovernorParams,
  });
  return {
    ...governor,
    lockerWrapper: new LockerWrapper(
      sdk,
      electorate,
      governor.governorWrapper.governorKey
    ),
  };
};
