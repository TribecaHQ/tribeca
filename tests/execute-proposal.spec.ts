import type { SmartWalletWrapper } from "@gokiprotocol/client";
import { findTransactionAddress, GokiSDK } from "@gokiprotocol/client";
import { expectTX } from "@saberhq/chai-solana";
import type { u64 } from "@saberhq/token-utils";
import { createMint, sleep } from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import { BN } from "bn.js";
import { expect } from "chai";

import type { GovernorWrapper } from "../src";
import {
  DEFAULT_GOVERNANCE_PARAMETERS,
  findGovernorAddress,
  findSimpleElectorateAddress,
  SimpleVoterWrapper,
  VoteSide,
} from "../src";
import {
  createUser,
  INITIAL_MINT_AMOUNT,
  makeSDK,
  ONE,
  ZERO,
} from "./workspace";

describe("Execute proposal", () => {
  const sdk = makeSDK();
  const { provider } = sdk;
  const gokiSDK = GokiSDK.load({ provider });

  let govTokenMint: PublicKey;
  let governorW: GovernorWrapper;
  let smartWalletW: SmartWalletWrapper;
  let voterW: SimpleVoterWrapper;

  before("Set up", async () => {
    govTokenMint = await createMint(provider);

    const electorateBase = Keypair.generate();
    const [electorateKey] = await findSimpleElectorateAddress(
      electorateBase.publicKey
    );
    const govBase = Keypair.generate();
    const [governor] = await findGovernorAddress(govBase.publicKey);
    const owners = [provider.wallet.publicKey, governor];

    const { smartWalletWrapper, tx: tx1 } = await gokiSDK.newSmartWallet({
      owners,
      threshold: ONE,
      numOwners: owners.length,
    });
    await expectTX(tx1, "create smart wallet").to.be.fulfilled;

    const { wrapper, tx: tx2 } = await sdk.govern.createGovernor({
      baseKP: govBase,
      electorate: electorateKey,
      smartWallet: smartWalletWrapper.key,
      quorumVotes: INITIAL_MINT_AMOUNT,
      votingDelay: ZERO,
      votingPeriod: new BN(2),
    });
    await expectTX(tx2, "create governor").to.be.fulfilled;

    const { electorate, tx: tx3 } = await sdk.createSimpleElectorate({
      baseKP: electorateBase,
      proposalThreshold: INITIAL_MINT_AMOUNT,
      governor,
      govTokenMint,
    });
    await expectTX(tx3, "initialize electorate").to.be.fulfilled;

    voterW = await SimpleVoterWrapper.load(sdk, electorate);
    governorW = wrapper;
    smartWalletW = smartWalletWrapper;
  });

  let user1: Keypair;
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
