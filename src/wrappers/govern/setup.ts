import type { GokiSDK, SmartWalletWrapper } from "@gokiprotocol/client";
import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";

import type { GovernanceParameters, TribecaSDK } from "../..";
import { DEFAULT_GOVERNANCE_PARAMETERS } from "../..";
import type { GovernorWrapper } from "..";
import { findGovernorAddress } from "..";

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
  govBaseKP = Keypair.generate(),
  smartWalletBaseKP = Keypair.generate(),
}: {
  createElectorate: (
    governor: PublicKey
  ) => Promise<{ key: PublicKey; tx: TransactionEnvelope }>;
  sdk: TribecaSDK;
  gokiSDK: GokiSDK;
  owners?: PublicKey[];
  governanceParameters?: Partial<GovernanceParameters>;
  /**
   * Base of the governor.
   */
  govBaseKP?: Keypair;
  /**
   * Base of the smart wallet.
   */
  smartWalletBaseKP?: Keypair;
}): Promise<{
  governorWrapper: GovernorWrapper;
  smartWalletWrapper: SmartWalletWrapper;
  electorate: PublicKey;
  createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[];
}> => {
  const [governor] = await findGovernorAddress(govBaseKP.publicKey);

  const createTXs: {
    title: string;
    tx: TransactionEnvelope;
  }[] = [];

  const { smartWalletWrapper, tx: tx1 } = await gokiSDK.newSmartWallet({
    owners: [...owners, governor],
    threshold: new BN(2),
    numOwners: 3,
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
      baseKP: govBaseKP,
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
