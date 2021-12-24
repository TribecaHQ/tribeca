import type { GokiSDK } from "@gokiprotocol/client";
import type { SmartWalletWrapper } from "@gokiprotocol/client/dist/cjs/wrappers/smartWallet";
import type { Idl } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor";
import { chaiSolana, expectTX } from "@saberhq/chai-solana";
import type { Provider } from "@saberhq/solana-contrib";
import { SolanaProvider, TransactionEnvelope } from "@saberhq/solana-contrib";
import {
  getOrCreateATA,
  SPLToken,
  TOKEN_PROGRAM_ID,
  u64,
} from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
} from "@solana/web3.js";
import chai, { assert } from "chai";

import type { TribecaPrograms } from "../../src";
import { TribecaSDK } from "../../src";
import type { GovernorWrapper } from "../../src/wrappers/govern/governor";
import { findGovernorAddress } from "../../src/wrappers/govern/pda";

chai.use(chaiSolana);

export const ZERO = new u64(0);
export const ONE = new u64(1);
export const INITIAL_MINT_AMOUNT = new u64(1_000_000_000);

export const DUMMY_INSTRUCTIONS = [
  Keypair.generate().publicKey,
  Keypair.generate().publicKey,
  Keypair.generate().publicKey,
].map(
  (pid) =>
    new TransactionInstruction({
      programId: pid,
      keys: [],
    })
);

export type Workspace = TribecaPrograms;

export const makeSDK = (): TribecaSDK => {
  const anchorProvider = anchor.Provider.env();
  anchor.setProvider(anchorProvider);

  const provider = SolanaProvider.load({
    connection: anchorProvider.connection,
    sendConnection: anchorProvider.connection,
    wallet: anchorProvider.wallet,
    opts: anchorProvider.opts,
  });
  return TribecaSDK.load({
    provider,
  });
};

type IDLError = NonNullable<Idl["errors"]>[number];

export const assertError = (error: IDLError, other: IDLError): void => {
  assert.strictEqual(error.code, other.code);
  assert.strictEqual(error.msg, other.msg);
};

export const setupGovernor = async ({
  electorate,
  sdk,
  gokiSDK,
  owners,
  ...governorParams
}: {
  electorate: PublicKey;
  sdk: TribecaSDK;
  gokiSDK: GokiSDK;
  owners: PublicKey[];
  quorumVotes?: anchor.BN;
  votingDelay?: anchor.BN;
  votingPeriod?: anchor.BN;
  smartWalletOwner?: PublicKey;
}): Promise<{
  governorWrapper: GovernorWrapper;
  smartWalletWrapper: SmartWalletWrapper;
}> => {
  const baseKP = Keypair.generate();
  const [governor] = await findGovernorAddress(baseKP.publicKey);

  const { smartWalletWrapper, tx: tx1 } = await gokiSDK.newSmartWallet({
    owners: [...owners, governor],
    threshold: ONE,
    numOwners: 3,
  });
  await expectTX(tx1, "create smart wallet").to.be.fulfilled;

  const { wrapper, tx: tx2 } = await sdk.govern.createGovernor({
    baseKP,
    electorate,
    smartWallet: smartWalletWrapper.key,
    ...governorParams,
  });
  await expectTX(tx2, "create governor").to.be.fulfilled;

  return {
    governorWrapper: wrapper,
    smartWalletWrapper,
  };
};

export const createUser = async (
  provider: Provider,
  govTokenMint: PublicKey
): Promise<Keypair> => {
  const user = Keypair.generate();

  await provider.connection.requestAirdrop(user.publicKey, LAMPORTS_PER_SOL);

  const { address, instruction } = await getOrCreateATA({
    provider,
    mint: govTokenMint,
    owner: user.publicKey,
  });
  const mintToIx = SPLToken.createMintToInstruction(
    TOKEN_PROGRAM_ID,
    govTokenMint,
    address,
    provider.wallet.publicKey,
    [],
    new u64(INITIAL_MINT_AMOUNT)
  );

  const tx = new TransactionEnvelope(
    provider,
    instruction ? [instruction, mintToIx] : [mintToIx]
  );
  await expectTX(tx, "mint gov tokens to user").to.be.fulfilled;

  return user;
};

export const executeTransactionBySmartWallet = async ({
  provider,
  smartWalletWrapper,
  instructions,
}: {
  provider: Provider;
  smartWalletWrapper: SmartWalletWrapper;
  instructions: TransactionInstruction[];
}): Promise<PublicKey> => {
  const { transactionKey, tx: tx1 } = await smartWalletWrapper.newTransaction({
    proposer: provider.wallet.publicKey,
    instructions,
  });
  await expectTX(tx1, "create new transaction").to.be.fulfilled;

  const tx2 = await smartWalletWrapper.executeTransaction({ transactionKey });
  await expectTX(tx2, "execute transaction").to.be.fulfilled;

  return transactionKey;
};
