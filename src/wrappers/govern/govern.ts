import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { PublicKey } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";
import type BN from "bn.js";

import { DEFAULT_GOVERNANCE_PARAMETERS } from "../..";
import type { GovernProgram, VoteData } from "../../programs/govern";
import type { TribecaSDK } from "../../sdk";
import { GovernorWrapper } from "./governor";
import { findGovernorAddress } from "./pda";
import type { PendingGovernor } from "./types";

export class GovernWrapper {
  readonly program: GovernProgram;

  constructor(readonly sdk: TribecaSDK) {
    this.program = sdk.programs.Govern;
  }

  get provider() {
    return this.sdk.provider;
  }

  async fetchVote(key: PublicKey): Promise<VoteData> {
    return await this.program.account.vote.fetch(key);
  }

  async createGovernor({
    electorate,
    smartWallet,
    baseKP = Keypair.generate(),
    ...governorParams
  }: {
    electorate: PublicKey;
    smartWallet: PublicKey;
    baseKP?: Keypair;
    quorumVotes?: BN;
    votingDelay?: BN;
    votingPeriod?: BN;
    smartWalletOwner?: PublicKey;
  }): Promise<PendingGovernor> {
    const [governor, bump] = await findGovernorAddress(baseKP.publicKey);
    const wrapper = new GovernorWrapper(this.sdk, governor);
    return {
      wrapper,
      tx: new TransactionEnvelope(
        this.provider,
        [
          this.sdk.programs.Govern.instruction.createGovernor(
            bump,
            electorate,
            {
              ...DEFAULT_GOVERNANCE_PARAMETERS,
              ...governorParams,
            },
            {
              accounts: {
                base: baseKP.publicKey,
                governor,
                smartWallet,
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
}
