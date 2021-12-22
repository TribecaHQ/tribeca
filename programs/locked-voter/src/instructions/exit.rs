use crate::*;
use anchor_spl::token;

impl<'info> Exit<'info> {
    pub fn exit(&mut self) -> ProgramResult {
        let seeds: &[&[&[u8]]] = escrow_seeds!(self.escrow);

        // transfer tokens from the escrow
        // if there are zero tokens in the escrow, short-circuit.
        if self.escrow.amount > 0 {
            token::transfer(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.escrow_tokens.to_account_info(),
                        to: self.destination_tokens.to_account_info(),
                        authority: self.escrow.to_account_info(),
                    },
                )
                .with_signer(seeds),
                self.escrow.amount,
            )?;
        }

        // update the locker
        let locker = &mut self.locker;
        locker.locked_supply = unwrap_int!(locker.locked_supply.checked_sub(self.escrow.amount));

        emit!(ExitEscrowEvent {
            escrow_owner: self.escrow.owner,
            locker: locker.key(),
            locker_supply: locker.locked_supply,
            timestamp: Clock::get()?.unix_timestamp,
            released_amount: self.escrow.amount,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for Exit<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.locker, self.escrow.locker);
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);
        assert_keys_eq!(self.escrow.tokens, self.escrow_tokens);
        let now = Clock::get()?.unix_timestamp;
        msg!(
            "now: {}; escrow_ends_at: {}",
            now,
            self.escrow.escrow_ends_at
        );
        invariant!(self.escrow.escrow_ends_at < now, "escrow has not ended");

        Ok(())
    }
}
