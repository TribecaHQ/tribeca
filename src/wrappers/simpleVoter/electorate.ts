import {
  createMemoInstruction,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import type { u64 } from "@saberhq/token-utils";
import {
  getATAAddress,
  getOrCreateATA,
  TOKEN_PROGRAM_ID,
} from "@saberhq/token-utils";
import type { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { SystemProgram } from "@solana/web3.js";
import invariant from "tiny-invariant";

import { TRIBECA_ADDRESSES } from "../../constants";
import type {
  ElectorateData,
  ProposalData,
  SimpleVoterProgram,
  TokenRecordData,
} from "../../programs";
import type { TribecaSDK } from "../../sdk";
import { GovernorWrapper } from "../govern/governor";
import { findTokenRecordAddress } from "./pda";
import type { VoteArgs } from "./types";

/**
 * Helper methods around a Simple Voter electorate.
 */
export class SimpleVoterWrapper {
  readonly program: SimpleVoterProgram;
  readonly governor: GovernorWrapper;

  electorateData?: ElectorateData;

  constructor(
    readonly sdk: TribecaSDK,
    readonly electorate: PublicKey,
    readonly governorKey: PublicKey
  ) {
    this.program = sdk.programs.SimpleVoter;
    this.governor = new GovernorWrapper(sdk, governorKey);
  }

  get provider() {
    return this.sdk.provider;
  }

  static async load(
    sdk: TribecaSDK,
    electorateKey: PublicKey
  ): Promise<SimpleVoterWrapper> {
    const electorateData =
      await sdk.programs.SimpleVoter.account.electorate.fetch(electorateKey);
    const wrapper = new SimpleVoterWrapper(
      sdk,
      electorateKey,
      electorateData.governor
    );
    wrapper.electorateData = electorateData;
    return wrapper;
  }

  async fetchProposalData(proposalKey: PublicKey): Promise<ProposalData> {
    return await this.sdk.programs.Govern.account.proposal.fetch(proposalKey);
  }

  async fetchTokenRecord(tokenRecordKey: PublicKey): Promise<TokenRecordData> {
    return await this.program.account.tokenRecord.fetch(tokenRecordKey);
  }

  async fetchVoterMetadata(): Promise<ElectorateData> {
    invariant(this.electorate, "electorate not set");
    this.electorateData = await this.program.account.electorate.fetch(
      this.electorate
    );
    return this.electorateData;
  }

  async getOrCreateTokenRecord(
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<{
    tokenRecord: PublicKey;
    instruction: TransactionInstruction | null;
  }> {
    invariant(this.electorate, "electorate not set");

    const [tokenRecord] = await findTokenRecordAddress(
      authority,
      this.electorate
    );

    try {
      await this.program.account.tokenRecord.fetch(tokenRecord);
      return { tokenRecord, instruction: null };
    } catch {
      return {
        tokenRecord,
        instruction: await this.initializeTokenRecordIx(authority),
      };
    }
  }

  async depositTokens(
    amount: u64,
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    invariant(this.electorate, "electorate not set");

    const { tokenRecord, instruction: initTokenRecordIx } =
      await this.getOrCreateTokenRecord(authority);
    const { govTokenAccount, govTokenVault, instructions } =
      await this._getOrCreateGovTokenATAsInternal(authority, tokenRecord);
    if (initTokenRecordIx) {
      instructions.push(initTokenRecordIx);
    }
    instructions.push(
      this.program.instruction.depositTokens(amount, {
        accounts: {
          authority,
          govTokenAccount,
          govTokenVault,
          tokenRecord,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      })
    );

    return new TransactionEnvelope(this.sdk.provider, instructions);
  }

  async withdrawTokens(
    amount: u64,
    authority: PublicKey
  ): Promise<TransactionEnvelope> {
    invariant(this.electorate, "electorate not set");
    invariant(this.electorateData, "electorate data not loaded");

    const [tokenRecord] = await findTokenRecordAddress(
      authority,
      this.electorate
    );
    const { govTokenAccount, govTokenVault, instructions } =
      await this._getOrCreateGovTokenATAsInternal(authority, tokenRecord);
    instructions.push(
      this.program.instruction.withdrawTokens(amount, {
        accounts: {
          authority,
          govTokenAccount,
          govTokenVault,
          tokenRecord,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      })
    );

    return new TransactionEnvelope(this.sdk.provider, instructions);
  }

  activateProposal(proposal: PublicKey): TransactionEnvelope {
    const ix = this.program.instruction.activateProposal({
      accounts: {
        electorate: this.electorate,
        governor: this.governorKey,
        proposal,
        governProgram: TRIBECA_ADDRESSES.Govern,
      },
    });
    return new TransactionEnvelope(this.sdk.provider, [ix]);
  }

  async castVotes(args: VoteArgs): Promise<TransactionEnvelope> {
    invariant(this.electorateData, "electorate data not loaded");
    const {
      authority = this.provider.wallet.publicKey,
      proposal,
      voteSide,
      reason,
    } = args;
    const ixs: TransactionInstruction[] = [];
    const { tokenRecord, instruction: tokenRecordIx } =
      await this.getOrCreateTokenRecord(authority);
    if (tokenRecordIx) {
      ixs.push(tokenRecordIx);
    }

    const { voteKey, instruction: createVoteIX } =
      await this.governor.getOrCreateVote({
        proposal,
        voter: authority,
      });
    if (createVoteIX) {
      ixs.push(createVoteIX);
    }

    ixs.push(
      this.program.instruction.castVotes(voteSide, {
        accounts: {
          electorate: this.electorate,
          authority: authority ?? this.sdk.provider.wallet.publicKey,
          proposal,
          tokenRecord,
          vote: voteKey,
          tribeca: this._genTribecaContext(),
        },
      })
    );
    if (reason) {
      ixs.push(createMemoInstruction(reason, [authority]));
    }

    return new TransactionEnvelope(this.sdk.provider, ixs);
  }

  async withdrawVotes(
    proposal: PublicKey,
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionEnvelope> {
    invariant(this.electorateData, "electorate data not loaded");

    const ixs: TransactionInstruction[] = [];
    const { tokenRecord, instruction: tokenRecordIx } =
      await this.getOrCreateTokenRecord(authority);
    if (tokenRecordIx) {
      ixs.push(tokenRecordIx);
    }

    const { voteKey, instruction: voteRecieptIx } =
      await this.governor.getOrCreateVote({
        proposal,
      });
    if (voteRecieptIx) {
      ixs.push(voteRecieptIx);
    }

    ixs.push(
      this.program.instruction.withdrawVotes({
        accounts: {
          electorate: this.electorate,
          authority,
          proposal,
          tokenRecord,
          vote: voteKey,
          tribeca: this._genTribecaContext(),
        },
      })
    );

    return new TransactionEnvelope(this.sdk.provider, ixs);
  }

  async depositTokenAndCastVote(
    args: VoteArgs & { amount: u64 }
  ): Promise<TransactionEnvelope> {
    const {
      amount,
      proposal,
      voteSide,
      reason,
      authority = this.provider.wallet.publicKey,
    } = args;
    const { tokenRecord, instruction: tokenRecordIx } =
      await this.getOrCreateTokenRecord(authority);
    const { govTokenAccount, govTokenVault, instructions } =
      await this._getOrCreateGovTokenATAsInternal(authority, tokenRecord);
    if (tokenRecordIx) {
      instructions.push(tokenRecordIx);
    }

    const { voteKey, instruction: createVoteIX } =
      await this.governor.getOrCreateVote({
        proposal,
        voter: authority,
      });
    if (createVoteIX) {
      instructions.push(createVoteIX);
    }

    instructions.push(
      this.program.instruction.depositTokens(amount, {
        accounts: {
          authority,
          govTokenAccount,
          govTokenVault,
          tokenRecord,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      })
    );

    return new TransactionEnvelope(
      this.sdk.provider,
      instructions.concat(
        this._genCastVotesInstructions({
          proposal,
          authority,
          voteSide,
          reason,
          vote: voteKey,
          tokenRecord,
        })
      )
    );
  }

  async initializeTokenRecordIx(
    authority: PublicKey = this.sdk.provider.wallet.publicKey
  ): Promise<TransactionInstruction> {
    invariant(this.electorate, "electorate not set");
    invariant(this.electorateData, "electorate data not loaded");

    const [tokenRecord, bump] = await findTokenRecordAddress(
      authority,
      this.electorate
    );

    return this.program.instruction.initializeTokenRecord(bump, {
      accounts: {
        authority,
        tokenRecord,
        electorate: this.electorate,
        govTokenVault: await getATAAddress({
          mint: this.electorateData.govTokenMint,
          owner: tokenRecord,
        }),
        payer: this.sdk.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
    });
  }

  private _genCastVotesInstructions(
    args: VoteArgs & { tokenRecord: PublicKey; vote: PublicKey }
  ): TransactionInstruction[] {
    const { proposal, voteSide, reason, tokenRecord, vote } = args;
    const authority = args.authority ?? this.provider.wallet.publicKey;

    const ixs: TransactionInstruction[] = [];
    ixs.push(
      this.program.instruction.castVotes(voteSide, {
        accounts: {
          electorate: this.electorate,
          authority,
          proposal,
          tokenRecord,
          vote,
          tribeca: this._genTribecaContext(),
        },
      })
    );
    if (reason) {
      ixs.push(createMemoInstruction(reason, [authority]));
    }

    return ixs;
  }

  private _genTribecaContext(): { governor: PublicKey; program: PublicKey } {
    invariant(this.electorateData, "electrate data not loaded");
    return {
      governor: this.electorateData.governor,
      program: TRIBECA_ADDRESSES.Govern,
    };
  }

  private async _getOrCreateGovTokenATAsInternal(
    authority: PublicKey,
    tokenRecord: PublicKey
  ): Promise<{
    govTokenAccount: PublicKey;
    govTokenVault: PublicKey;
    instructions: TransactionInstruction[];
  }> {
    invariant(this.electorateData, "electorate data not loaded");

    const { provider } = this.sdk;

    const { address: govTokenAccount, instruction: ix1 } = await getOrCreateATA(
      {
        provider,
        mint: this.electorateData.govTokenMint,
        owner: authority,
        payer: authority,
      }
    );
    const { address: govTokenVault, instruction: ix2 } = await getOrCreateATA({
      provider,
      mint: this.electorateData.govTokenMint,
      owner: tokenRecord,
      payer: authority,
    });

    return {
      govTokenAccount,
      govTokenVault,
      instructions: [ix1, ix2].filter(
        (ix): ix is TransactionInstruction => !!ix
      ),
    };
  }
}
