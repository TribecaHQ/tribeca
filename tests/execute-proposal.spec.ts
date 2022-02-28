import type { SmartWalletWrapper } from "@gokiprotocol/client";
import { findTransactionAddress } from "@gokiprotocol/client";
import { assertTXSuccess, expectTX } from "@saberhq/chai-solana";
import type { u64 } from "@saberhq/token-utils";
import { createMint, sleep } from "@saberhq/token-utils";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import { BN } from "bn.js";
import { expect } from "chai";

import type { GovernorWrapper } from "../src";
import {
  createGovernorWithElectorate,
  DEFAULT_GOVERNANCE_PARAMETERS,
  SimpleVoterWrapper,
  VoteSide,
} from "../src";
import { createUser, INITIAL_MINT_AMOUNT, makeSDK, ZERO } from "./workspace";

describe("Execute proposal", () => {
  const sdk = makeSDK();
  const { provider } = sdk;

  let govTokenMint: PublicKey;
  let governorW: GovernorWrapper;
  let smartWalletW: SmartWalletWrapper;
  let voterW: SimpleVoterWrapper;

  before("Set up", async () => {
    govTokenMint = await createMint(provider);

    const electorateBase = Keypair.generate();
    const govBase = Keypair.generate();

    const { electorate, createTXs, governorWrapper, smartWalletWrapper } =
      await createGovernorWithElectorate({
        createElectorate: async (governorKey) => {
          const { electorate, tx: tx3 } = await sdk.createSimpleElectorate({
            baseKP: electorateBase,
            proposalThreshold: INITIAL_MINT_AMOUNT,
            governor: governorKey,
            govTokenMint,
          });
          return {
            key: electorate,
            tx: tx3,
          };
        },
        sdk,
        owners: [provider.wallet.publicKey],
        governanceParameters: {
          quorumVotes: INITIAL_MINT_AMOUNT,
          votingDelay: ZERO,
          votingPeriod: new BN(2),
        },
        governorBaseKP: govBase,
        smartWalletParameters: {
          // threshold = 1 allows us to test the whitelist stuff
          threshold: 1,
        },
      });

    for (const { title, tx } of createTXs) {
      await assertTXSuccess(tx, title);
    }

    voterW = await SimpleVoterWrapper.load(sdk, electorate);
    governorW = governorWrapper;
    smartWalletW = smartWalletWrapper;
  });

  let user1: Signer;
  let proposal: PublicKey;
  let proposalIndex: u64;

  beforeEach("create and activate a proposal", async () => {
    user1 = await createUser(provider, govTokenMint);
    const {
      proposal: proposalInner,
      index,
      tx: createProposalTx,
    } = await voterW.governor.createProposal({
      proposer: user1.publicKey,
      instructions: [
        await governorW.setGovernanceParamsIx(DEFAULT_GOVERNANCE_PARAMETERS),
      ],
    });
    createProposalTx.addSigners(user1);
    await expectTX(createProposalTx, "creating a proposal").to.be.fulfilled;
    await expectTX(voterW.activateProposal(proposalInner), "activate proposal")
      .to.be.fulfilled;
    proposal = proposalInner;
    proposalIndex = index;
  });

  it("Happy path", async () => {
    let governorData = await governorW.data();
    const tx1 = await voterW.depositTokenAndCastVote({
      voteSide: VoteSide.For,
      proposal,
      authority: user1.publicKey,
      amount: governorData.params.quorumVotes,
    });
    tx1.addSigners(user1);
    await expectTX(tx1, "deposit tokens and cast votes").to.be.fulfilled;
    await sleep(2500);
    const tx3 = await governorW.queueProposal({
      index: proposalIndex,
    });
    await expectTX(tx3, "queue proposal for execution").to.be.fulfilled;

    const [transactionKey] = await findTransactionAddress(smartWalletW.key, 0);
    const tx4 = await smartWalletW.executeTransaction({
      transactionKey,
    });
    await expectTX(tx4, "execute the transaction via the smart wallet").to.be
      .fulfilled;

    governorData = await governorW.reload();
    expect(governorData.params.quorumVotes.toString()).to.equal(
      DEFAULT_GOVERNANCE_PARAMETERS.quorumVotes.toString()
    );
    expect(governorData.params.timelockDelaySeconds.toString()).to.equal(
      DEFAULT_GOVERNANCE_PARAMETERS.timelockDelaySeconds.toString()
    );
    expect(governorData.params.votingDelay.toString()).to.equal(
      DEFAULT_GOVERNANCE_PARAMETERS.votingDelay.toString()
    );
    expect(governorData.params.votingPeriod.toString()).to.equal(
      DEFAULT_GOVERNANCE_PARAMETERS.votingPeriod.toString()
    );
  });
});
