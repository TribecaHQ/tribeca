import type { SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";

import type { GovernorWrapper, LockerParams } from "../..";
import {
  DEFAULT_GOVERNANCE_PARAMETERS,
  DEFAULT_LOCKER_PARAMS,
} from "../../constants";
import type { CreateGovernorWithElectorateParams } from "../govern/setup";
import { createGovernorWithElectorate } from "../govern/setup";
import { LockerWrapper } from "./locker";

export interface CreateLockerParams extends CreateGovernorWithElectorateParams {
  govTokenMint: PublicKey;
  lockerParams?: Partial<LockerParams>;
  /**
   * Base of the locker.
   */
  lockerBaseKP?: Keypair;
}

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
    gokiSDK,
    owners,
    governanceParameters,
    governorBaseKP,
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
