/// Accounts for [govern::cancel_proposal].
#[derive(Accounts)]
pub struct CancelProposal<'info> {
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal] to activate.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The [Proposal::proposer].
    pub proposer: Signer<'info>,
}
