import type { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { TokenAmount } from "@saberhq/token-utils";
import {
  getATAAddress,
  getOrCreateATA,
  TOKEN_PROGRAM_ID,
} from "@saberhq/token-utils";
import type { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { SystemProgram } from "@solana/web3.js";
import BN from "bn.js";

import { TRIBECA_ADDRESSES } from "../../constants";
import type { EscrowData, LockerData } from "../../programs/lockedVoter";
import type { TribecaSDK } from "../../sdk";
import { findVoteAddress } from "../govern/pda";
import type { VoteSide } from "../govern/types";

export class VoteEscrow {
  private _lockerData: LockerData | null = null;
  private _escrowData: EscrowData | null = null;

  constructor(
    readonly sdk: TribecaSDK,
    readonly locker: PublicKey,
    readonly governorKey: PublicKey,
    readonly escrowKey: PublicKey,
    readonly owner: PublicKey
  ) {}

  get provider() {
    return this.sdk.provider;
  }

  get lockerProgram() {
    return this.sdk.programs.LockedVoter;
  }

  /**
   * Locker data.
   */
  async lockerData() {
    if (!this._lockerData) {
      this._lockerData = await this.lockerProgram.account.locker.fetch(
        this.locker
      );
    }
    return this._lockerData;
  }

  /**
   * Escrow data.
   */
  async data() {
    if (!this._escrowData) {
      this._escrowData = await this.lockerProgram.account.escrow.fetch(
        this.escrowKey
      );
    }
    return this._escrowData;
  }

  /**
   * Creates a function to calculate the voting power of this escrow.
   * @returns
   */
  async makeCalculateVotingPower(): Promise<(timestampSeconds: number) => BN> {
    const escrowData = await this.data();
    const lockerData = await this.lockerData();
    return (timestampSeconds: number) => {
      if (escrowData.escrowStartedAt.eq(new BN(0))) {
        return new BN(0);
      }
      if (
        timestampSeconds < escrowData.escrowStartedAt.toNumber() ||
        timestampSeconds >= escrowData.escrowEndsAt.toNumber()
      ) {
        return new BN(0);
      }
      const secondsUntilLockupExpiry = escrowData.escrowEndsAt
        .sub(new BN(timestampSeconds))
        .toNumber();
      const relevantSecondsUntilLockupExpiry = Math.min(
        secondsUntilLockupExpiry,
        lockerData.params.maxStakeDuration.toNumber()
      );
      const powerIfMaxLockup = escrowData.amount.mul(
        new BN(lockerData.params.maxStakeVoteMultiplier)
      );
      return powerIfMaxLockup
        .mul(new BN(relevantSecondsUntilLockupExpiry))
        .div(lockerData.params.maxStakeDuration);
    };
  }

  /**
   * Calculates the voting power of this escrow.
   * @param time Optional time to calculate power for.
   * @returns
   */
  async calculateVotingPower(time: Date = new Date()): Promise<BN> {
    return (await this.makeCalculateVotingPower())(
      Math.floor(time.getTime() / 1_000)
    );
  }

  /**
   * Activates a proposal.
   * @returns
   */
  activateProposal(proposal: PublicKey): TransactionEnvelope {
    return this.provider.newTX([
      this.lockerProgram.instruction.activateProposal({
        accounts: {
          locker: this.locker,
          governor: this.governorKey,
          proposal,
          escrow: this.escrowKey,
          escrowOwner: this.owner,
          governProgram: TRIBECA_ADDRESSES.Govern,
        },
      }),
    ]);
  }

  /**
   * Casts a vote on a proposal.
   * @returns
   */
  async castVote({
    proposal,
    side,
  }: {
    proposal: PublicKey;
    side: VoteSide;
  }): Promise<TransactionEnvelope> {
    const [voteKey, voteBump] = await findVoteAddress(proposal, this.owner);
    const vote = await this.provider.getAccountInfo(voteKey);
    let createVoteIX: TransactionInstruction | null = null;
    if (!vote) {
      createVoteIX = this.sdk.programs.Govern.instruction.newVote(
        voteBump,
        this.owner,
        {
          accounts: {
            proposal,
            vote: voteKey,
            payer: this.provider.wallet.publicKey,
            systemProgram: SystemProgram.programId,
          },
        }
      );
    }
    return this.provider.newTX([
      createVoteIX,
      this.lockerProgram.instruction.castVote(side, {
        accounts: {
          locker: this.locker,
          escrow: this.escrowKey,
          voteDelegate: this.owner,
          proposal,
          vote: voteKey,
          governor: this.governorKey,
          governProgram: TRIBECA_ADDRESSES.Govern,
        },
      }),
    ]);
  }

  /**
   * Locks tokens into the escrow.
   * @param amount
   * @param durationSeconds The duration of the lock, in seconds
   * @param authority
   * @returns
   */
  async lock(
    amount: TokenAmount,
    durationSeconds: number
  ): Promise<TransactionEnvelope> {
    const escrowData = await this.data();
    const sourceTokens = await getATAAddress({
      mint: amount.token.mintAccount,
      owner: escrowData.owner,
    });
    return this.provider.newTX([
      this.lockerProgram.instruction.lock(
        amount.toU64(),
        new BN(durationSeconds),
        {
          accounts: {
            locker: this.locker,
            escrow: this.escrowKey,
            escrowTokens: escrowData.tokens,
            escrowOwner: escrowData.owner,
            sourceTokens,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
        }
      ),
    ]);
  }

  /**
   * Exits the escrow.
   * @returns
   */
  async exit(): Promise<TransactionEnvelope> {
    const lockerData = await this.lockerData();
    const escrowData = await this.data();
    const destinationTokens = await getOrCreateATA({
      provider: this.provider,
      mint: lockerData.tokenMint,
      owner: escrowData.owner,
    });
    return this.provider.newTX([
      destinationTokens.instruction,
      this.lockerProgram.instruction.exit({
        accounts: {
          locker: this.locker,
          escrow: this.escrowKey,
          escrowOwner: escrowData.owner,
          escrowTokens: escrowData.tokens,
          destinationTokens: destinationTokens.address,
          payer: this.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }),
    ]);
  }
}
