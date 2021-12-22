import type { SmartWalletWrapper } from "@gokiprotocol/client";
import { GokiSDK } from "@gokiprotocol/client";
import { BN } from "@project-serum/anchor";
import { expectTX } from "@saberhq/chai-solana";
import {
  createMint,
  getATAAddress,
  getTokenAccount,
  sleep,
} from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";
import invariant from "tiny-invariant";

import { DEFAULT_GOVERNANCE_PARAMETERS } from "../src";
import { SimpleVoterWrapper, VoteSide } from "../src/wrappers";
import type { GovernorWrapper } from "../src/wrappers/govern/governor";
import { findVoteAddress } from "../src/wrappers/govern/pda";
import {
  findSimpleElectorateAddress,
  findTokenRecordAddress,
} from "../src/wrappers/simpleVoter/pda";
import {
  createUser,
  DUMMY_INSTRUCTIONS,
  INITIAL_MINT_AMOUNT,
  makeSDK,
  setupGovernor,
  ZERO,
} from "./workspace";

describe("Simple Voter", () => {
  const sdk = makeSDK();
  const gokiSDK = GokiSDK.load({ provider: sdk.provider });

  let base: PublicKey;
  let govTokenMint: PublicKey;

  let governorW: GovernorWrapper;
  let smartWalletW: SmartWalletWrapper;
  let voterW: SimpleVoterWrapper;

  before(async () => {
    govTokenMint = await createMint(sdk.provider);

    const baseKP = Keypair.generate();
    base = baseKP.publicKey;
    const [electorateKey] = await findSimpleElectorateAddress(base);

    const owners = [sdk.provider.wallet.publicKey];
    const { governorWrapper, smartWalletWrapper } = await setupGovernor({
      electorate: electorateKey,
      sdk,
      gokiSDK,
      owners,
    });

    const { electorate, tx: tx1 } = await sdk.createSimpleElectorate({
      baseKP,
      proposalThreshold: INITIAL_MINT_AMOUNT,
      governor: governorWrapper.governorKey,
      govTokenMint,
    });
    await expectTX(tx1, "initialize electorate").to.be.fulfilled;

    voterW = await SimpleVoterWrapper.load(sdk, electorate);
    governorW = governorWrapper;
    smartWalletW = smartWalletWrapper;
  });

  let proposal: PublicKey;
  let user: Keypair;

  beforeEach("create a proposal", async () => {
    user = await createUser(sdk.provider, govTokenMint);
    const { proposal: proposalInner, tx: createProposalTx } =
      await voterW.governor.createProposal({
        proposer: user.publicKey,
        instructions: DUMMY_INSTRUCTIONS,
      });
    createProposalTx.addSigners(user);
    await expectTX(createProposalTx, "creating a proposal").to.be.fulfilled;
    proposal = proposalInner;
  });

  it("Simple voter's electorate was initialized", async () => {
    const { electorate } = voterW;
    const electorateData = await voterW.fetchVoterMetadata();
    const [expectedElectorate, bump] = await findSimpleElectorateAddress(base);

    expect(electorate).eqAddress(expectedElectorate);
    expect(electorateData.bump).equal(bump);
    expect(electorateData.proposalThreshold.toString()).eq(
      INITIAL_MINT_AMOUNT.toString()
    );
    expect(electorateData.base).eqAddress(base);
    expect(electorateData.govTokenMint).eqAddress(govTokenMint);
    expect(electorateData.governor).eqAddress(governorW.governorKey);
  });

  describe("Token Record", () => {
    let user: Keypair;

    beforeEach("Create user and deposit tokens", async () => {
      user = await createUser(sdk.provider, govTokenMint);
      const depositTx = await voterW.depositTokens(
        INITIAL_MINT_AMOUNT,
        user.publicKey
      );
      depositTx.addSigners(user);
      await expectTX(depositTx, "deposit tokens").to.be.fulfilled;
    });

    it("Token record was initialized", async () => {
      const { electorate, electorateData } = voterW;
      invariant(electorate, "electorate not found");
      invariant(electorateData, "electorateData not found");

      const [tokenRecordKey, bump] = await findTokenRecordAddress(
        user.publicKey,
        electorate
      );

      const tokenRecord = await voterW.fetchTokenRecord(tokenRecordKey);
      expect(tokenRecord.bump).equal(bump);
      expect(tokenRecord.balance.toString()).equal(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(tokenRecord.authority).eqAddress(user.publicKey);
      expect(tokenRecord.electorate).eqAddress(electorate);
      expect(tokenRecord.tokenVaultKey).eqAddress(
        await getATAAddress({
          mint: electorateData.govTokenMint,
          owner: tokenRecordKey,
        })
      );
    });

    it("Withdraw tokens", async () => {
      const withdrawTx = await voterW.withdrawTokens(
        INITIAL_MINT_AMOUNT,
        user.publicKey
      );
      withdrawTx.addSigners(user);
      await expectTX(withdrawTx, "withdraw tokens").to.be.fulfilled;

      const { electorate, electorateData } = voterW;
      invariant(electorate, "electorate not found");
      invariant(electorateData, "electorateData not found");

      const [tokenRecordKey] = await findTokenRecordAddress(
        user.publicKey,
        electorate
      );
      const tokenRecord = await voterW.fetchTokenRecord(tokenRecordKey);
      expect(tokenRecord.balance.toString()).equal(ZERO.toString());

      const tokenVaultInfo = await getTokenAccount(
        sdk.provider,
        await getATAAddress({
          mint: electorateData.govTokenMint,
          owner: tokenRecordKey,
        })
      );
      expect(tokenVaultInfo.amount.toString()).equal(
        tokenRecord.balance.toString()
      );

      const userAccountInfo = await getTokenAccount(
        sdk.provider,
        await getATAAddress({
          mint: electorateData.govTokenMint,
          owner: user.publicKey,
        })
      );
      expect(userAccountInfo.amount.toString()).equal(
        INITIAL_MINT_AMOUNT.toString()
      );
    });
  });

  describe("Cast votes", () => {
    let user: Keypair;

    beforeEach("Create user and deposit tokens", async () => {
      user = await createUser(sdk.provider, govTokenMint);
      const depositTx = await voterW.depositTokens(
        INITIAL_MINT_AMOUNT,
        user.publicKey
      );
      depositTx.addSigners(user);
      await expectTX(depositTx, "deposit tokens").to.be.fulfilled;

      await sleep(3_000);

      await expectTX(voterW.activateProposal(proposal), "activate proposal").to
        .be.fulfilled;
    });

    it("Vote receipt was initialized properly", async () => {
      const tx = await voterW.castVotes({
        proposal,
        authority: user.publicKey,
        voteSide: VoteSide.For,
      });
      tx.addSigners(user);
      await expectTX(tx, "cast for vote").to.be.fulfilled;

      const { electorate, electorateData } = voterW;
      invariant(electorate && electorateData, "elecotrate not found");

      const [voteReceiptKey, bump] = await findVoteAddress(
        proposal,
        user.publicKey
      );
      const voteRecipetData = await governorW.sdk.govern.fetchVote(
        voteReceiptKey
      );
      expect(voteRecipetData.bump).equal(bump);
      expect(voteRecipetData.side).equal(VoteSide.For);
      expect(voteRecipetData.proposal).eqAddress(proposal);
      expect(voteRecipetData.voter).eqAddress(user.publicKey);
      expect(voteRecipetData.weight.toString()).equal(
        INITIAL_MINT_AMOUNT.toString()
      );
    });

    it("Cast for on proposal", async () => {
      const tx = await voterW.castVotes({
        proposal,
        authority: user.publicKey,
        voteSide: VoteSide.For,
      });
      tx.addSigners(user);
      await expectTX(tx, "cast for votes").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      expect(proposalData.forVotes.toString()).eql(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(proposalData.againstVotes.toString()).eql("0");
      expect(proposalData.abstainVotes.toString()).eql("0");
    });

    it("Cast against on proposal", async () => {
      const tx = await voterW.castVotes({
        proposal,
        authority: user.publicKey,
        voteSide: VoteSide.Against,
      });
      tx.addSigners(user);
      await expectTX(tx, "cast against votes").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      expect(proposalData.againstVotes.toString()).eql(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(proposalData.forVotes.toString()).eql("0");
      expect(proposalData.abstainVotes.toString()).eql("0");
    });

    it("Cast abstain on proposal", async () => {
      const tx = await voterW.castVotes({
        proposal,
        authority: user.publicKey,
        voteSide: VoteSide.Abstain,
      });
      tx.addSigners(user);
      await expectTX(tx, "cast abstain votes").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      expect(proposalData.abstainVotes.toString()).eql(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(proposalData.forVotes.toString()).eql("0");
      expect(proposalData.againstVotes.toString()).eql("0");
    });

    it("Cast votes on multiple proposals", async () => {
      const { proposal: anotherProposal, tx: createAnotherProposalTx } =
        await voterW.governor.createProposal({
          proposer: user.publicKey,
          instructions: DUMMY_INSTRUCTIONS,
        });
      createAnotherProposalTx.addSigners(user);
      await expectTX(createAnotherProposalTx, "creating another proposal").to.be
        .fulfilled;

      await sleep(3_000);

      await expectTX(
        voterW.activateProposal(anotherProposal),
        "activate proposal"
      ).to.be.fulfilled;

      const castForProposalTx = await voterW.castVotes({
        proposal,
        authority: user.publicKey,
        voteSide: VoteSide.For,
      });
      castForProposalTx.addSigners(user);
      await expectTX(castForProposalTx, "cast votes for first proposal").to.be
        .fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposal);
      expect(proposalData.forVotes.toString()).eql(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(proposalData.againstVotes.toString()).eql("0");
      expect(proposalData.abstainVotes.toString()).eql("0");

      const tx = await voterW.castVotes({
        proposal: anotherProposal,
        authority: user.publicKey,
        voteSide: VoteSide.For,
      });
      tx.addSigners(user);
      await expectTX(tx, "cast votes for second proposal").to.be.fulfilled;

      const anotherProposalData = await governorW.fetchProposalByKey(
        anotherProposal
      );
      expect(anotherProposalData.forVotes.toString()).eql(
        INITIAL_MINT_AMOUNT.toString()
      );
      expect(anotherProposalData.againstVotes.toString()).eql("0");
      expect(anotherProposalData.abstainVotes.toString()).eql("0");
    });

    it("unable to cast vote after proposal voting period ends", async () => {
      const setGovernanceIx = await governorW.setGovernanceParamsIx({
        ...DEFAULT_GOVERNANCE_PARAMETERS,
        votingPeriod: new BN(1),
      });
      const { tx, transactionKey } = await smartWalletW.newTransaction({
        proposer: sdk.provider.wallet.publicKey,
        payer: sdk.provider.wallet.publicKey,
        instructions: [setGovernanceIx],
      });
      await expectTX(tx, "new smart wallet transaction").to.be.fulfilled;
      await expectTX(
        smartWalletW.executeTransaction({
          transactionKey,
          owner: sdk.provider.wallet.publicKey,
        }),
        "execute transaction"
      ).to.be.fulfilled;

      const { proposal: obsoleteProposal, tx: createObsoleteProposalTx } =
        await voterW.governor.createProposal({
          proposer: user.publicKey,
          instructions: DUMMY_INSTRUCTIONS,
        });
      createObsoleteProposalTx.addSigners(user);
      await expectTX(createObsoleteProposalTx, "creating an obsolete proposal")
        .to.be.fulfilled;

      await sleep(1000);
      await expectTX(
        voterW.activateProposal(obsoleteProposal),
        "activate proposal"
      ).to.be.fulfilled;

      // Wait for voting period to pass
      await sleep(2000);

      const castVoteTx = await voterW.castVotes({
        proposal: obsoleteProposal,
        authority: user.publicKey,
        voteSide: VoteSide.For,
      });
      castVoteTx.addSigners(user);
      await expectTX(castVoteTx, "cast votes for obsolete proposal").to.be
        .rejected;
    });
  });
});
