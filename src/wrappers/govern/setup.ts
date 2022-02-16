import type { SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";

import type {
  GovernanceParameters,
  SmartWalletParameters,
  TribecaSDK,
} from "../..";
import {
  DEFAULT_GOVERNANCE_PARAMETERS,
  DEFAULT_GOVERNOR_SMART_WALLET_PARAMS,
} from "../..";
import type { GovernorWrapper } from "..";
import { findGovernorAddress } from "..";

/**
 * Parameters for the {@link createGovernorWithElectorate} function.
 */
export interface CreateGovernorWithElectorateParams {
  /**
   * Function to create an electorate.
   */
  createElectorate: (
    governor: PublicKey
  ) => Promise<{ key: PublicKey; tx?: TransactionEnvelope }>;
  /**
   * Tribeca SDK.
   */
  sdk: TribecaSDK;
  /**
   * Additional owners on the governance smart wallet.
   *
   * For the Tribeca Trinity, this should be an owner invoker and an "emergency DAO" Smart Wallet.
   */
  owners?: PublicKey[];
  /**
   * Additional governance parameters.
   */
  governanceParameters?: Partial<GovernanceParameters>;
  /**
   * Base of the governor.
   */
  governorBaseKP?: Signer;
  /**
   * Base of the smart wallet.
   */
  smartWalletBaseKP?: Keypair;
  /**
   * Additional smart wallet parameters.
   */
  smartWalletParameters?: Partial<SmartWalletParameters>;
}

/**
 * Creates a Governor.
 * @returns
 */
export const createGovernorWithElectorate = async ({
  createElectorate,
  sdk,
  owners = [sdk.provider.wallet.publicKey],
  governanceParameters = DEFAULT_GOVERNANCE_PARAMETERS,
  governorBaseKP = Keypair.generate(),
  smartWalletBaseKP = Keypair.generate(),
  smartWalletParameters = DEFAULT_GOVERNOR_SMART_WALLET_PARAMS,
}: CreateGovernorWithElectorateParams): Promise<{
  governorWrapper: GovernorWrapper;
  smartWalletWrapper: SmartWalletWrapper;
  electorate: PublicKey;
  createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[];
}> => {
  const [governor] = await findGovernorAddress(governorBaseKP.publicKey);

  const createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[] = [];

  if (owners.find((owner) => owner.equals(governor))) {
    throw new Error("governor should not be provided in owners list");
  }

  const allOwners = [...owners, governor];
  const smartWalletParams: SmartWalletParameters = {
    ...DEFAULT_GOVERNOR_SMART_WALLET_PARAMS,
    ...smartWalletParameters,
    maxOwners: allOwners.length,
  };

  const { smartWalletWrapper, tx: tx1 } = await sdk.goki.newSmartWallet({
    owners: allOwners,
    threshold: new BN(smartWalletParams.threshold),
    numOwners: smartWalletParams.maxOwners,
    delay: new BN(smartWalletParams.delay),
    base: smartWalletBaseKP,
  });
  createTXs.push({
    title: "Create Smart Wallet",
    tx: tx1,
  });

  const { key: electorate, tx: createElectorateTX } = await createElectorate(
    governor
  );

  const { wrapper: governorWrapper, tx: tx2 } = await sdk.govern.createGovernor(
    {
      ...governanceParameters,
      baseKP: governorBaseKP,
      electorate,
      smartWallet: smartWalletWrapper.key,
    }
  );
  createTXs.push({
    title: "Create Governor",
    tx: tx2,
  });

  if (createElectorateTX) {
    createTXs.push({
      title: "Create Electorate",
      tx: createElectorateTX,
    });
  }

  return {
    governorWrapper,
    smartWalletWrapper,
    createTXs,
    electorate,
  };
};
