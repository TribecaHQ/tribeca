import type { SmartWalletWrapper } from "@gokiprotocol/client/dist/cjs/wrappers/smartWallet";
import * as anchor from "@project-serum/anchor";
import { chaiSolana, expectTX } from "@saberhq/chai-solana";
import type { Provider } from "@saberhq/solana-contrib";
import {
  SolanaAugmentedProvider,
  SolanaProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import {
  getOrCreateATA,
  SPLToken,
  TOKEN_PROGRAM_ID,
  u64,
} from "@saberhq/token-utils";
import type { PublicKey, Signer } from "@solana/web3.js";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
} from "@solana/web3.js";
import chai from "chai";

import type { TribecaPrograms } from "../../src";
import { TribecaSDK } from "../../src";

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

export const createUser = async (
  provider: Provider,
  govTokenMint: PublicKey,
  user: Signer = Keypair.generate()
): Promise<Signer> => {
  await (
    await new SolanaAugmentedProvider(provider).requestAirdrop(
      LAMPORTS_PER_SOL,
      user.publicKey
    )
  ).wait();

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
