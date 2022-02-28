import { GokiSDK } from "@gokiprotocol/client";
import type { BN } from "@project-serum/anchor";
import { newProgramMap } from "@saberhq/anchor-contrib";
import type { AugmentedProvider, Provider } from "@saberhq/solana-contrib";
import {
  SolanaAugmentedProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";

import type { TribecaPrograms } from "./constants";
import {
  DEFAULT_LOCKER_PARAMS,
  TRIBECA_ADDRESSES,
  TRIBECA_IDLS,
} from "./constants";
import type { LockerParams } from "./programs/lockedVoter";
import type { CreateLockerParams } from "./wrappers";
import { createLocker, GovernWrapper } from "./wrappers";
import { findLockerAddress } from "./wrappers/lockedVoter/pda";
import { findSimpleElectorateAddress } from "./wrappers/simpleVoter/pda";
import type { PendingElectorate } from "./wrappers/simpleVoter/types";

/**
 * Tribeca protocol SDK.
 */
export class TribecaSDK {
  /**
   * The Goki SDK.
   */
  readonly goki: GokiSDK;

  constructor(
    /**
     * Provider.
     */
    readonly provider: AugmentedProvider,
    /**
     * Programs.
     */
    readonly programs: TribecaPrograms
  ) {
    this.goki = GokiSDK.load({ provider });
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  withSigner(signer: Signer): TribecaSDK {
    return TribecaSDK.load({
      provider: this.provider.withSigner(signer),
    });
  }

  /**
   * Loads the SDK.
   * @returns
   */
  static load({ provider }: { provider: Provider }): TribecaSDK {
    const programs: TribecaPrograms = newProgramMap<TribecaPrograms>(
      provider,
      TRIBECA_IDLS,
      TRIBECA_ADDRESSES
    );
    return new TribecaSDK(new SolanaAugmentedProvider(provider), programs);
  }

  /**
   * Govern program helpers.
   */
  get govern(): GovernWrapper {
    return new GovernWrapper(this);
  }

  /**
   * Creates a new simple electorate.
   * @returns
   */
  async createSimpleElectorate({
    proposalThreshold,
    governor,
    govTokenMint,
    baseKP = Keypair.generate(),
  }: {
    proposalThreshold: BN;
    baseKP?: Signer;
    governor: PublicKey;
    govTokenMint: PublicKey;
  }): Promise<PendingElectorate> {
    const [electorate, bump] = await findSimpleElectorateAddress(
      baseKP.publicKey
    );
    return {
      electorate,
      tx: new TransactionEnvelope(
        this.provider,
        [
          this.programs.SimpleVoter.instruction.initializeElectorate(
            bump,
            proposalThreshold,
            {
              accounts: {
                base: baseKP.publicKey,
                governor,
                electorate,
                govTokenMint,
                payer: this.provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
              },
            }
          ),
        ],
        [baseKP]
      ),
    };
  }

  /**
   * Creates a new Locker and Governor.
   * @param params
   * @returns
   */
  async createLockerAndGovernor(params: Omit<CreateLockerParams, "sdk">) {
    return await createLocker({ ...params, sdk: this });
  }

  /**
   * Creates a Locker, which is an Electorate that supports vote locking.
   * @returns
   */
  async createLocker({
    governor,
    govTokenMint,
    baseKP = Keypair.generate(),
    ...providedLockerParams
  }: {
    baseKP?: Signer;
    governor: PublicKey;
    govTokenMint: PublicKey;
  } & Partial<LockerParams>): Promise<{
    locker: PublicKey;
    tx: TransactionEnvelope;
  }> {
    const [locker, bump] = await findLockerAddress(baseKP.publicKey);
    const lockerParams = {
      ...DEFAULT_LOCKER_PARAMS,
      ...providedLockerParams,
    };
    return {
      locker,
      tx: new TransactionEnvelope(
        this.provider,
        [
          this.programs.LockedVoter.instruction.newLocker(bump, lockerParams, {
            accounts: {
              base: baseKP.publicKey,
              governor,
              locker,
              tokenMint: govTokenMint,
              payer: this.provider.wallet.publicKey,
              systemProgram: SystemProgram.programId,
            },
          }),
        ],
        [baseKP]
      ),
    };
  }
}
