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