/// Accounts for [govern::activate_proposal].
#[derive(Accounts)]
pub struct ActivateProposal<'info> {
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The [Proposal] to activate.
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// The electorate of the [Governor] that may activate the proposal.
    pub electorate: Signer<'info>,
}