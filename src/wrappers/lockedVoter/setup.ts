import type { GokiSDK, SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";

import type {
  GovernanceParameters,
  GovernorWrapper,
  LockerParams,
} from "../..";
import {
  DEFAULT_GOVERNANCE_PARAMETERS,
  DEFAULT_LOCKER_PARAMS,
} from "../../constants";
import type { TribecaSDK } from "../../sdk";
import { createGovernorWithElectorate } from "../govern/setup";
import { LockerWrapper } from "./locker";

/**
 * Creates a new Locker.
 * @returns
 */
export const createLocker = async ({
  sdk,
  gokiSDK,
  govTokenMint,
  owners = [sdk.provider.wallet.publicKey],
  governanceParameters = DEFAULT_GOVERNANCE_PARAMETERS,
  lockerParams = DEFAULT_LOCKER_PARAMS,
  governorBaseKP = Keypair.generate(),
  lockerBaseKP = Keypair.generate(),
  smartWalletBaseKP = Keypair.generate(),
}: {
  sdk: TribecaSDK;
  gokiSDK: GokiSDK;
  govTokenMint: PublicKey;
  owners?: PublicKey[];
  governanceParameters?: Partial<GovernanceParameters>;
  lockerParams?: Partial<LockerParams>;
  /**
   * Base of the governor.
   */
  governorBaseKP?: Keypair;
  /**
   * Base of the governor.
   */
  lockerBaseKP?: Keypair;
  /**
   * Base of the smart wallet.
   */
  smartWalletBaseKP?: Keypair;
}): Promise<{
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
    gokiSDK,
    owners,
    governanceParameters,
    govBaseKP: governorBaseKP,
    smartWalletBaseKP,
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
