/// Accounts for [govern::new_vote].
#[derive(Accounts)]
#[instruction(bump: u8, voter: Pubkey)]
pub struct NewVote<'info> {
    /// Proposal being voted on.
    pub proposal: Account<'info, Proposal>,

    /// The vote.
    #[account(
        init,
        seeds = [
            b"TribecaVote".as_ref(),
            proposal.key().as_ref(),
            voter.as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub vote: Account<'info, Vote>,

    /// Payer of the [Vote].
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}