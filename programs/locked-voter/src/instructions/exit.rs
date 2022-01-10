use crate::*;
use anchor_spl::token;

/// Accounts for [locked_voter::exit].
#[derive(Accounts)]
pub struct Exit<'info> {
    /// The [Locker] being exited from.
    #[account(mut)]
    pub locker: Account<'info, Locker>,

    /// The [Escrow] that is being closed.
    #[account(mut, close = payer)]
    pub escrow: Account<'info, Escrow>,

    /// Authority of the [Escrow].
    pub escrow_owner: Signer<'info>,
    /// Tokens locked up in the [Escrow].
    #[account(mut)]
    pub escrow_tokens: Account<'info, TokenAccount>,
    /// Destination for the tokens to unlock.
    #[account(mut)]
    pub destination_tokens: Account<'info, TokenAccount>,

    /// The payer to receive the rent refund.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token program.
    pub token_program: Program<'info, Token>,
}

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
        invariant!(self.escrow.escrow_ends_at < now, EscrowNotEnded);

        Ok(())
    }
}

#[event]
/// Event called in [locked_voter::exit].
pub struct ExitEscrowEvent {
    /// The owner of the [Escrow].
    #[index]
    pub escrow_owner: Pubkey,
    /// The locker for the [Escrow].
    #[index]
    pub locker: Pubkey,
    /// Timestamp for the event.
    pub timestamp: i64,
    /// The amount of tokens locked inside the [Locker].
    pub locker_supply: u64,
    /// The amount released from the [Escrow].
    pub released_amount: u64,
}
