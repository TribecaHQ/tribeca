use crate::*;

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
