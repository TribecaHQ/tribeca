/// Accounts for [govern::queue_proposal].
#[derive(Accounts)]
pub struct QueueProposal<'info> {
    /// The Governor.
    pub governor: Account<'info, Governor>,
    /// The Proposal to queue.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The transaction key of the proposal.
    #[account(mut)]
    pub transaction: UncheckedAccount<'info>,
    /// The Smart Wallet.
    #[account(mut)]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// Payer of the queued transaction.
    pub payer: Signer<'info>,
    /// The Smart Wallet program.
    pub smart_wallet_program: Program<'info, smart_wallet::program::SmartWallet>,
    /// The System program.
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for QueueProposal<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.governor,
            self.proposal.governor,
            "proposal should be under the governor"
        );
        assert_keys_eq!(
            self.smart_wallet,
            self.governor.smart_wallet,
            "smart wallet should match"
        );
        let now = Clock::get()?.unix_timestamp;
        let proposal_state = unwrap_opt!(self.proposal.state(now), "invalid state");
        if proposal_state != ProposalState::Succeeded {
            msg!(
                "now: {}, voting_ends_at: {}",
                now,
                self.proposal.voting_ends_at
            );
            msg!(
                "for votes: {}, against votes: {}",
                self.proposal.for_votes,
                self.proposal.against_votes,
            );
            msg!(
                "quorum req: {}, abstain votes: {}",
                self.governor.params.quorum_votes,
                self.proposal.abstain_votes,
            );
            invariant!(
                proposal_state == ProposalState::Succeeded,
                "proposal must be succeeded to be queued"
            );
        }
        Ok(())
    }
}