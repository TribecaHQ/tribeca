use crate::*;

/// Accounts for [locked_voter::new_escrow].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewEscrow<'info> {
    /// [Locker].
    pub locker: Account<'info, Locker>,

    /// [Escrow].
    #[account(
        init,
        seeds = [
            b"Escrow".as_ref(),
            locker.key().to_bytes().as_ref(),
            escrow_owner.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub escrow: Account<'info, Escrow>,

    /// Authority of the [Escrow] to be created.
    pub escrow_owner: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

impl<'info> NewEscrow<'info> {
    /// Creates a new [Escrow].
    pub fn new_escrow(&mut self, bump: u8) -> ProgramResult {
        let escrow = &mut self.escrow;
        escrow.locker = self.locker.key();
        escrow.owner = self.escrow_owner.key();
        escrow.bump = bump;

        // token account of the escrow is the ATA.
        escrow.tokens = anchor_spl::associated_token::get_associated_token_address(
            &escrow.key(),
            &self.locker.token_mint,
        );
        escrow.amount = 0;
        escrow.escrow_started_at = 0;
        escrow.escrow_ends_at = 0;
        escrow.vote_delegate = self.escrow_owner.key();

        emit!(NewEscrowEvent {
            escrow: escrow.key(),
            escrow_owner: escrow.owner,
            locker: escrow.locker,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for NewEscrow<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

#[event]
/// Event called in [locked_voter::new_escrow].
pub struct NewEscrowEvent {
    /// The [Escrow] being created.
    pub escrow: Pubkey,
    /// The owner of the [Escrow].
    #[index]
    pub escrow_owner: Pubkey,
    /// The locker for the [Escrow].
    #[index]
    pub locker: Pubkey,
    /// Timestamp for the event.
    pub timestamp: i64,
}
