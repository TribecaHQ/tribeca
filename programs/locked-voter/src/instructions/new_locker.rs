use crate::*;

/// Accounts for [locked_voter::new_locker].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewLocker<'info> {
    /// Base.
    pub base: Signer<'info>,

    /// [Locker].
    #[account(
        init,
        seeds = [
            b"Locker".as_ref(),
            base.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub locker: Account<'info, Locker>,

    /// Mint of the token that can be used to join the [Locker].
    pub token_mint: Account<'info, Mint>,

    /// [Governor] associated with the [Locker].
    pub governor: Account<'info, Governor>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

impl<'info> NewLocker<'info> {
    /// Creates a new [Locker].
    pub fn new_locker(&mut self, bump: u8, params: LockerParams) -> ProgramResult {
        let locker = &mut self.locker;
        locker.token_mint = self.token_mint.key();
        locker.governor = self.governor.key();
        locker.base = self.base.key();
        locker.bump = bump;
        locker.params = params;

        emit!(NewLockerEvent {
            governor: locker.governor,
            locker: locker.key(),
            token_mint: locker.token_mint,
            params,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for NewLocker<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

#[event]
/// Event called in [locked_voter::new_locker].
pub struct NewLockerEvent {
    /// The governor for the [Locker].
    #[index]
    pub governor: Pubkey,
    /// The [Locker] being created.
    pub locker: Pubkey,
    /// Mint of the token that can be used to join the [Locker].
    pub token_mint: Pubkey,
    /// New [LockerParams].
    pub params: LockerParams,
}
