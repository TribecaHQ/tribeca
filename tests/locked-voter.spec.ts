import type { SmartWalletWrapper } from "@gokiprotocol/client";
import { GokiSDK } from "@gokiprotocol/client";
import { newProgram } from "@saberhq/anchor-contrib";
import { expectTX } from "@saberhq/chai-solana";
import { TransactionEnvelope } from "@saberhq/solana-contrib";
import {
  createMint,
  getATAAddress,
  getOrCreateATA,
  getTokenAccount,
  sleep,
  TOKEN_PROGRAM_ID,
} from "@saberhq/token-utils";
import type {
  SendTransactionError,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  Keypair,
  PublicKey,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import BN from "bn.js";
import { expect } from "chai";
import { zip } from "lodash";
import invariant from "tiny-invariant";

import {
  DEFAULT_LOCKER_PARAMS,
  LockedVoterErrors,
  ONE_DAY,
  ONE_YEAR,
  TRIBECA_ADDRESSES,
} from "../src";
import {
  findProposalAddress,
  LockerWrapper,
  VoteEscrow,
} from "../src/wrappers";
import type { GovernorWrapper } from "../src/wrappers/govern/governor";
import { VoteSide } from "../src/wrappers/govern/types";
import {
  findEscrowAddress,
  findLockerAddress,
  findWhitelistAddress,
} from "../src/wrappers/lockedVoter/pda";
import type { WhitelistTesterProgram } from "./workspace";
import {
  createUser,
  DUMMY_INSTRUCTIONS,
  executeTransactionBySmartWallet,
  INITIAL_MINT_AMOUNT,
  makeSDK,
  setupGovernor,
  WhitelistTesterJSON,
  ZERO,
} from "./workspace";

const expectLockedSupply = async (
  locker: LockerWrapper,
  expectedSupply: BN
): Promise<void> => {
  const lockerData = await locker.reload();
  expect(lockerData.lockedSupply).to.bignumber.eq(expectedSupply);
};

describe("Locked Voter", () => {
  const sdk = makeSDK();
  const gokiSDK = GokiSDK.load({ provider: sdk.provider });

  let base: PublicKey;
  let govTokenMint: PublicKey;

  let governorW: GovernorWrapper;
  let lockerW: LockerWrapper;
  let smartWalletW: SmartWalletWrapper;

  before(async () => {
    govTokenMint = await createMint(sdk.provider);

    const baseKP = Keypair.generate();
    base = baseKP.publicKey;
    const [lockerKey] = await findLockerAddress(base);

    const owners = [sdk.provider.wallet.publicKey];
    const { governorWrapper, smartWalletWrapper } = await setupGovernor({
      electorate: lockerKey,
      sdk,
      gokiSDK,
      owners,
    });

    const { locker, tx: tx1 } = await sdk.createLocker({
      baseKP,
      proposalActivationMinVotes: INITIAL_MINT_AMOUNT,
      governor: governorWrapper.governorKey,
      govTokenMint,
    });
    await expectTX(tx1, "initialize locker").to.be.fulfilled;

    lockerW = await LockerWrapper.load(
      sdk,
      locker,
      governorWrapper.governorKey
    );
    governorW = governorWrapper;
    smartWalletW = smartWalletWrapper;
  });

  let proposal: PublicKey;
  let proposalIndex: BN;
  let user: Keypair;

  beforeEach("create a proposal", async () => {
    user = await createUser(sdk.provider, govTokenMint);
    const {
      proposal: proposalInner,
      index,
      tx: createProposalTx,
    } = await lockerW.governor.createProposal({
      proposer: user.publicKey,
      instructions: DUMMY_INSTRUCTIONS,
    });
    createProposalTx.addSigners(user);
    await expectTX(createProposalTx, "creating a proposal").to.be.fulfilled;
    proposal = proposalInner;
    proposalIndex = index;
  });

  it("Locked voter's electorate was initialized", async () => {
    const { locker: electorate } = lockerW;
    const electorateData = await lockerW.data();
    const [expectedLocker, bump] = await findLockerAddress(base);

    expect(electorate).eqAddress(expectedLocker);
    expect(electorateData.bump).equal(bump);
    expect(electorateData.base).eqAddress(base);
    expect(electorateData.tokenMint).eqAddress(govTokenMint);
    expect(electorateData.governor).eqAddress(governorW.governorKey);
    expect(electorateData.lockedSupply).to.bignumber.eq(ZERO);

    const { params } = electorateData;
    expect(params.maxStakeVoteMultiplier).to.eq(
      DEFAULT_LOCKER_PARAMS.maxStakeVoteMultiplier
    );
    expect(params.minStakeDuration).to.bignumber.eq(
      DEFAULT_LOCKER_PARAMS.minStakeDuration
    );
    expect(params.maxStakeDuration).to.bignumber.to.bignumber.eq(
      DEFAULT_LOCKER_PARAMS.maxStakeDuration
    );
    expect(params.proposalActivationMinVotes).to.bignumber.eq(
      INITIAL_MINT_AMOUNT
    );
  });

  it("Proposal was initialized", async () => {
    const proposer = user.publicKey;
    const electorateData = await lockerW.data();
    const [expectedProposalKey, bump] = await findProposalAddress(
      electorateData.governor,
      proposalIndex
    );
    expect(proposal).to.eqAddress(expectedProposalKey);

    const governorData = await governorW.data();
    const proposalData = await lockerW.fetchProposalData(proposal);
    expect(proposalData.bump).to.equal(bump);
    expect(proposalData.index).to.bignumber.equal(proposalIndex);
    expect(proposalData.canceledAt).to.bignumber.equal(ZERO);
    expect(proposalData.queuedAt).to.bignumber.equal(ZERO);
    expect(proposalData.activatedAt).to.bignumber.equal(ZERO);
    expect(proposalData.votingEndsAt).to.bignumber.equal(ZERO);
    expect(proposalData.abstainVotes).to.bignumber.equal(ZERO);
    expect(proposalData.againstVotes).to.bignumber.equal(ZERO);
    expect(proposalData.forVotes).to.bignumber.equal(ZERO);
    expect(proposalData.quorumVotes).to.bignumber.equal(
      governorData.params.quorumVotes
    );
    expect(proposalData.queuedTransaction).to.eqAddress(PublicKey.default);
    expect(proposalData.proposer).to.eqAddress(proposer);
    expect(proposalData.governor).to.eqAddress(governorW.governorKey);

    zip(proposalData.instructions, DUMMY_INSTRUCTIONS).map(
      ([actual, expected]) => {
        invariant(expected);
        expect(actual).eql(expected);
      }
    );
  });

  it("Cannot lock duration below min stake duration", async () => {
    user = await createUser(sdk.provider, govTokenMint);
    const tx = await lockerW.lockTokens({
      amount: INITIAL_MINT_AMOUNT,
      duration: new BN(1),
      authority: user.publicKey,
    });
    tx.addSigners(user);

    try {
      await tx.send();
    } catch (e) {
      const error = e as SendTransactionError;
      expect(
        error.logs
          ?.join("/n")
          .includes(
            "LockupDurationTooShort: Lockup duration must at least be the min stake duration."
          )
      );
    }
  });

  describe("Escrow", () => {
    let user: Keypair;

    beforeEach("Create user and deposit tokens", async () => {
      user = await createUser(sdk.provider, govTokenMint);
      const lockTx = await lockerW.lockTokens({
        amount: INITIAL_MINT_AMOUNT,
        duration: DEFAULT_LOCKER_PARAMS.maxStakeDuration,
        authority: user.publicKey,
      });
      lockTx.addSigners(user);
      await expectTX(lockTx, "lock tokens").to.be.fulfilled;
    });

    it("Escrow was initialized and locker was updated", async () => {
      const { locker } = lockerW;
      const lockerData = await lockerW.data();

      const [escrowKey, bump] = await findEscrowAddress(locker, user.publicKey);
      const escrowATA = await getATAAddress({
        mint: lockerData.tokenMint,
        owner: escrowKey,
      });
      const escrow = await lockerW.fetchEscrow(escrowKey);
      expect(escrow.bump).equal(bump);
      expect(escrow.amount.toString()).equal(INITIAL_MINT_AMOUNT.toString());
      expect(escrow.owner).eqAddress(user.publicKey);
      expect(escrow.locker).eqAddress(locker);
      expect(escrow.tokens).eqAddress(escrowATA);
      expect(escrow.voteDelegate).eqAddress(user.publicKey);
      expect(escrow.escrowEndsAt.sub(escrow.escrowStartedAt).toString()).equal(
        DEFAULT_LOCKER_PARAMS.maxStakeDuration.toString()
      );
      await expectLockedSupply(lockerW, INITIAL_MINT_AMOUNT);

      const tokenAccount = await getTokenAccount(sdk.provider, escrowATA);
      expect(tokenAccount.amount).to.bignumber.eq(INITIAL_MINT_AMOUNT);
    });

    it("Exit should fail", async () => {
      const exitTx = await lockerW.exit({ authority: user.publicKey });
      exitTx.addSigners(user);

      try {
        await exitTx.confirm();
      } catch (e) {
        const error = e as SendTransactionError;
        expect(error.logs?.join("\n")).to.include(
          "EscrowNotEnded: Escrow has not ended."
        );
      }
    });

    it("Cannot vote on inactive proposal", async () => {
      const voteTx = await lockerW.castVotes({
        voteSide: VoteSide.Abstain,
        proposal,
        authority: user.publicKey,
      });
      voteTx.addSigners(user);
      try {
        await voteTx.confirm();
      } catch (e) {
        const error = e as SendTransactionError;
        expect(error.logs?.join("\n")).to.include(
          "Invariant failed: proposal must be active"
        );
      }
    });

    it("Activate proposal", async () => {
      await sleep(3000); // sleep to pass voting delay
      const activateTx = await lockerW.activateProposal({
        proposal,
        authority: user.publicKey,
      });
      activateTx.addSigners(user);
      await expectTX(activateTx, "activate").to.be.fulfilled;
      const proposalData = await lockerW.fetchProposalData(proposal);
      expect(proposalData.activatedAt).to.bignumber.greaterThan(ZERO);
    });

    it("Escrow refresh cannot shorten the escrow time remaining", async () => {
      const lockTx = await lockerW.lockTokens({
        amount: INITIAL_MINT_AMOUNT,
        duration: ONE_YEAR.mul(new BN("4")), // 4 years < 5 years
        authority: user.publicKey,
      });
      lockTx.addSigners(user);

      try {
        await lockTx.confirm();
      } catch (e) {
        const error = e as SendTransactionError;
        expect(
          error.logs
            ?.join("\n")
            .includes("escrow refresh cannot shorten the escrow time remaining")
        );
      }
    });
  });

  it("Exit escrow", async () => {
    const { governorKey } = governorW;
    const { locker, tx } = await sdk.createLocker({
      minStakeDuration: new BN(1),
      proposalActivationMinVotes: INITIAL_MINT_AMOUNT,
      governor: governorKey,
      govTokenMint,
    });
    await expectTX(tx, "initialize locker").to.be.fulfilled;

    const shortLockerW = await LockerWrapper.load(sdk, locker, governorKey);
    const lockTx = await shortLockerW.lockTokens({
      amount: INITIAL_MINT_AMOUNT,
      duration: new BN(1),
      authority: user.publicKey,
    });
    lockTx.addSigners(user);
    await expectTX(lockTx, "short lock up").to.be.fulfilled;
    await expectLockedSupply(shortLockerW, INITIAL_MINT_AMOUNT);

    await sleep(2500); // sleep to lockup
    const exitTx = await shortLockerW.exit({ authority: user.publicKey });
    exitTx.addSigners(user);
    await expectTX(exitTx, "exit lock up").to.be.fulfilled;

    const [escrowKey] = await findEscrowAddress(locker, user.publicKey);
    try {
      await lockerW.fetchEscrow(escrowKey);
    } catch (e) {
      const error = e as Error;
      expect(error.message).to.equal(
        `Account does not exist ${escrowKey.toString()}`
      );
    }

    const userATA = await getATAAddress({
      mint: govTokenMint,
      owner: user.publicKey,
    });
    const tokenAccount = await getTokenAccount(sdk.provider, userATA);
    expect(tokenAccount.amount).to.bignumber.eq(INITIAL_MINT_AMOUNT);

    await expectLockedSupply(shortLockerW, ZERO);
  });

  describe("Voting", () => {
    let user: Keypair;
    let escrowW: VoteEscrow;

    beforeEach("lock token and activate proposal", async () => {
      user = await createUser(sdk.provider, govTokenMint);
      const lockTx = await lockerW.lockTokens({
        amount: INITIAL_MINT_AMOUNT,
        duration: DEFAULT_LOCKER_PARAMS.maxStakeDuration,
        authority: user.publicKey,
      });
      lockTx.addSigners(user);
      await expectTX(lockTx, "lock tokens").to.be.fulfilled;
      await sleep(3000); // sleep to pass voting delay
      const activateTx = await lockerW.activateProposal({
        proposal,
        authority: user.publicKey,
      });
      activateTx.addSigners(user);
      await expectTX(activateTx, "activate").to.be.fulfilled;
      const { governorKey } = governorW;
      const { locker } = lockerW;

      const [escrowKey] = await findEscrowAddress(locker, user.publicKey);
      escrowW = new VoteEscrow(
        sdk,
        locker,
        governorKey,
        escrowKey,
        user.publicKey
      );
    });

    it("Cast for a proposal", async () => {
      const voteTx = await escrowW.castVote({ proposal, side: VoteSide.For });

      voteTx.addSigners(user);
      await expectTX(voteTx, "voting successful").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      const calculator = await escrowW.makeCalculateVotingPower();
      expect(proposalData.forVotes).to.bignumber.eq(
        calculator(proposalData.votingEndsAt.toNumber())
      );
      expect(proposalData.againstVotes).to.bignumber.eq(ZERO);
      expect(proposalData.abstainVotes).to.bignumber.eq(ZERO);
    });

    it("Cast against a proposal", async () => {
      const voteTx = await escrowW.castVote({
        proposal,
        side: VoteSide.Against,
      });

      voteTx.addSigners(user);
      await expectTX(voteTx, "voting successful").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      const calculator = await escrowW.makeCalculateVotingPower();
      expect(proposalData.forVotes).to.bignumber.eq(ZERO);
      expect(proposalData.againstVotes).to.bignumber.eq(
        calculator(proposalData.votingEndsAt.toNumber())
      );
      expect(proposalData.abstainVotes).to.bignumber.eq(ZERO);
    });

    it("Cast abstain on a proposal", async () => {
      const voteTx = await escrowW.castVote({
        proposal,
        side: VoteSide.Abstain,
      });

      voteTx.addSigners(user);
      await expectTX(voteTx, "voting successful").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      const calculator = await escrowW.makeCalculateVotingPower();
      expect(proposalData.againstVotes).to.bignumber.eq(ZERO);
      expect(proposalData.forVotes).to.bignumber.eq(ZERO);
      expect(proposalData.abstainVotes).to.bignumber.eq(
        calculator(proposalData.votingEndsAt.toNumber())
      );
    });
  });

  describe("CPI Whitelist", () => {
    const TEST_PROGRAM_ID = new PublicKey(
      "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
    );
    const testProgram = newProgram<WhitelistTesterProgram>(
      WhitelistTesterJSON,
      TEST_PROGRAM_ID,
      sdk.provider
    );

    beforeEach("Enable whitelist on the locker", async () => {
      await executeTransactionBySmartWallet({
        provider: sdk.provider,
        smartWalletWrapper: smartWalletW,
        instructions: [
          await lockerW.setLockerParamsIx({
            ...DEFAULT_LOCKER_PARAMS,
            whitelistEnabled: true,
          }),
        ],
      });
    });

    const buildCPITX = async (): Promise<TransactionEnvelope> => {
      const { provider } = sdk;
      const user = await createUser(provider, govTokenMint);
      const authority = user.publicKey;
      const { escrow, instruction: initEscrowIx } =
        await lockerW.getOrCreateEscrow(authority);

      const { address: sourceTokens, instruction: ataIx1 } =
        await getOrCreateATA({
          provider,
          mint: govTokenMint,
          owner: authority,
          payer: authority,
        });
      const { address: escrowTokens, instruction: ataIx2 } =
        await getOrCreateATA({
          provider,
          mint: govTokenMint,
          owner: escrow,
          payer: authority,
        });
      const instructions = [initEscrowIx, ataIx1, ataIx2].filter(
        (ix): ix is TransactionInstruction => !!ix
      );

      const [whitelistEntry] = await findWhitelistAddress(
        lockerW.locker,
        TEST_PROGRAM_ID
      );

      instructions.push(
        testProgram.instruction.lockTokens(INITIAL_MINT_AMOUNT, ONE_YEAR, {
          accounts: {
            locker: lockerW.locker,
            escrow,
            escrowOwner: authority,
            escrowTokens,
            sourceTokens,
            lockedVoterProgram: TRIBECA_ADDRESSES.LockedVoter,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          remainingAccounts: [
            {
              pubkey: SYSVAR_INSTRUCTIONS_PUBKEY,
              isWritable: false,
              isSigner: false,
            },
            {
              pubkey: whitelistEntry,
              isWritable: false,
              isSigner: false,
            },
          ],
        })
      );

      return new TransactionEnvelope(sdk.provider, instructions, [user]);
    };

    it("CPI fails when program is not whitelisted", async () => {
      const tx = await buildCPITX();
      try {
        await tx.confirm();
      } catch (e) {
        const error = e as Error;
        expect(error.message).to.include(
          `0x${LockedVoterErrors.ProgramNotWhitelisted.code.toString(16)}`
        );
      }
    });

    it("CPI succeeds after program has been whitelisted", async () => {
      await executeTransactionBySmartWallet({
        provider: sdk.provider,
        smartWalletWrapper: smartWalletW,
        instructions: [
          await lockerW.createApproveProgramLockPrivilegeIx(TEST_PROGRAM_ID),
        ],
      });
      const tx = await buildCPITX();
      await expectTX(tx, "successfully locked tokens via the whitelist tester")
        .to.be.fulfilled;
    });

    it("Non CPI lock invocation should succeed", async () => {
      const { provider } = sdk;
      const user = await createUser(provider, govTokenMint);
      const tx = await lockerW.lockTokens({
        amount: INITIAL_MINT_AMOUNT,
        duration: ONE_DAY,
        authority: user.publicKey,
      });
      tx.addSigners(user);
      await expectTX(tx, "lock tokens").to.be.fulfilled;
    });
  });
});
