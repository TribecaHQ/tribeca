import type { GokiSDK, SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";

import type { GovernanceParameters, TribecaSDK } from "../..";
import {
  DEFAULT_GOVERNANCE_PARAMETERS,
  DEFAULT_GOVERNOR_SMART_WALLET_MAX_OWNERS,
  DEFAULT_GOVERNOR_SMART_WALLET_THRESHOLD,
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
  ) => Promise<{ key: PublicKey; tx: TransactionEnvelope }>;
  /**
   * Tribeca SDK.
   */
  sdk: TribecaSDK;
  /**
   * Goki SDK.
   */
  gokiSDK: GokiSDK;
  /**
   * Other signers on the governance smart wallet.
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
   * Number of signers required to execute a smart wallet transaction. This is useful for testing.
   */
  threshold?: number;
  /**
   * Maximum number of owners on the smart wallet.
   */
  maxOwners?: number;
}

/**
 * Creates a Governor.
 * @returns
 */
export const createGovernorWithElectorate = async ({
  createElectorate,
  sdk,
  gokiSDK,
  owners = [sdk.provider.wallet.publicKey],
  governanceParameters = DEFAULT_GOVERNANCE_PARAMETERS,
  governorBaseKP = Keypair.generate(),
  smartWalletBaseKP = Keypair.generate(),
  threshold = DEFAULT_GOVERNOR_SMART_WALLET_THRESHOLD,
  maxOwners = DEFAULT_GOVERNOR_SMART_WALLET_MAX_OWNERS,
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

  const { smartWalletWrapper, tx: tx1 } = await gokiSDK.newSmartWallet({
    owners: [...owners, governor],
    threshold: new BN(threshold),
    numOwners: maxOwners,
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

  createTXs.push({
    title: "Create Electorate",
    tx: createElectorateTX,
  });

  return {
    governorWrapper,
    smartWalletWrapper,
    createTXs,
    electorate,
  };
};
