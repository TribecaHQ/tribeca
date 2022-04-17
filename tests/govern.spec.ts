import { GokiSDK } from "@gokiprotocol/client";
import { expectTX } from "@saberhq/chai-solana";
import type { SendTransactionError } from "@solana/web3.js";
import { Keypair, PublicKey } from "@solana/web3.js";
import type BN from "bn.js";
import { expect } from "chai";
import { zip } from "lodash";
import invariant from "tiny-invariant";

import { DEFAULT_VOTE_DELAY, DEFAULT_VOTE_PERIOD } from "../src";
import type { GovernorWrapper } from "../src/wrappers/govern/governor";
import {
  findGovernorAddress,
  findProposalAddress,
} from "../src/wrappers/govern/pda";
import { DUMMY_INSTRUCTIONS, makeSDK, setupGovernor, ZERO } from "./workspace";

describe("Govern", () => {
  const sdk = makeSDK();
  const gokiSDK = GokiSDK.load({ provider: sdk.provider });

  let governorW: GovernorWrapper;
  let smartWallet: PublicKey;

  before(async () => {
    const owners = [sdk.provider.wallet.publicKey];
    const electorate = Keypair.generate().publicKey;
    const { governorWrapper, smartWalletWrapper } = await setupGovernor({
      electorate,
      sdk,
      gokiSDK,
      owners,
    });

    smartWallet = smartWalletWrapper.key;
    governorW = governorWrapper;
  });

  it("Governor was initialized", async () => {
    const governorData = await governorW.data();
    const [governor, bump] = await findGovernorAddress(governorData.base);
    expect(governorW.governorKey).to.eqAddress(governor);

    invariant(governorData, "governor data was not loaded");

    expect(governorData.bump).to.equal(bump);
    expect(governorData.smartWallet).to.eqAddress(smartWallet);
    expect(governorData.proposalCount.toString()).to.eq(ZERO.toString());
    expect(governorData.params.votingDelay.toString()).eq(
      DEFAULT_VOTE_DELAY.toString()
    );
    expect(governorData.params.votingPeriod.toString()).eq(
      DEFAULT_VOTE_PERIOD.toString()
    );
  });

  describe("Proposal", () => {
    let proposalIndex: BN;
    let proposalKey: PublicKey;

    beforeEach("create a proposal", async () => {
      const { index, proposal, tx } = await governorW.createProposal({
        instructions: DUMMY_INSTRUCTIONS,
      });
      await expectTX(tx, "create a proposal").to.be.fulfilled;
      proposalIndex = index;
      proposalKey = proposal;
    });

    it("Proposal as initialized", async () => {
      const proposer = sdk.provider.wallet.publicKey;
      const { governorKey } = governorW;
      const [expectedProposalKey, bump] = await findProposalAddress(
        governorKey,
        proposalIndex
      );
      expect(proposalKey).to.eqAddress(expectedProposalKey);
      const governorData = await governorW.data();
      const proposalData = await governorW.fetchProposalByKey(proposalKey);
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
      expect(proposalData.governor).to.eqAddress(governorKey);

      zip(proposalData.instructions, DUMMY_INSTRUCTIONS).map(
        ([actual, expected]) => {
          invariant(expected);
          expect(actual).eql(expected);
        }
      );
    });

    it("Cancel a proposal", async () => {
      const tx = governorW.cancelProposal({
        proposal: proposalKey,
      });
      await expectTX(tx, "cancel a proposal").to.be.fulfilled;

      const proposalData = await governorW.fetchProposalByKey(proposalKey);
      expect(proposalData.canceledAt).to.be.bignumber.greaterThan(ZERO);
    });

    context("Proposal meta", () => {
      it("Cannot create proposal meta if not proposer", async () => {
        const fakeProposer = Keypair.generate();
        const createMetaTX = await governorW.createProposalMeta({
          proposer: fakeProposer.publicKey,
          proposal: proposalKey,
          title: "This is my Proposal",
          descriptionLink: "https://tribeca.so",
        });
        createMetaTX.addSigners(fakeProposer);

        try {
          await createMetaTX.confirm();
        } catch (e) {
          const error = e as SendTransactionError;
          expect(error.logs?.join("/n")).to.include(
            "Program log: self.proposer != self.proposal.proposer"
          );
        }
      });

      it("Can create proposal meta", async () => {
        const expectedTitle = "This is my Proposal";
        const expectedLink = "https://tribeca.so";
        const createMetaTX = await governorW.createProposalMeta({
          proposal: proposalKey,
          title: expectedTitle,
          descriptionLink: expectedLink,
        });
        await expectTX(createMetaTX, "creating proposal meta").to.be.fulfilled;
        const metadata = await governorW.fetchProposalMeta(proposalKey);
        expect(metadata.title).to.be.equal(expectedTitle);
        expect(metadata.descriptionLink).to.be.equal(expectedLink);
        expect(metadata.proposal).to.eqAddress(proposalKey);
      });
    });
  });
});
