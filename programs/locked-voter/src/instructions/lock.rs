use crate::*;
use anchor_lang::solana_program::sysvar::instructions::get_instruction_relative;
use anchor_spl::token;
use num_traits::ToPrimitive;

impl<'info> SetLockerParams<'info> {
    pub fn set_locker_params(&mut self, params: LockerParams) -> ProgramResult {
        let prev_params = self.locker.params;
        self.locker.params = params;

        emit!(LockerSetParamsEvent {
            locker: self.locker.key(),
            prev_params,
            params,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for SetLockerParams<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.governor, self.locker.governor, "governor mismatch");
        assert_keys_eq!(
            self.smart_wallet,
            self.governor.smart_wallet,
            "smart wallet mismatch"
        );
        Ok(())
    }
}

impl<'info> Lock<'info> {
    pub fn lock(&mut self, amount: u64, duration: i64) -> ProgramResult {
        invariant!(
            unwrap_int!(duration.to_u64()) >= self.locker.params.min_stake_duration,
            LockupDurationTooShort
        );
        invariant!(
            unwrap_int!(duration.to_u64()) <= self.locker.params.max_stake_duration,
            LockupDurationTooLong
        );

        // check that the escrow refresh is valid
        let escrow = &self.escrow;
        let prev_escrow_ends_at = escrow.escrow_ends_at;
        let next_escrow_started_at = Clock::get()?.unix_timestamp;
        let next_escrow_ends_at = unwrap_int!(next_escrow_started_at.checked_add(duration));
        if prev_escrow_ends_at > next_escrow_ends_at {
            msg!(
                "next_escrow_ends_at: {}; prev_escrow_ends_at: {}",
                next_escrow_ends_at,
                prev_escrow_ends_at
            );
            invariant!(
                next_escrow_ends_at >= prev_escrow_ends_at,
                RefreshCannotShorten
            );
        }

        // transfer tokens to the escrow
        // if amount is 0, we can skip this call.
        // One would lock 0 tokens at a duration to be able to refresh their existing lockup.
        if amount > 0 {
            token::transfer(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.source_tokens.to_account_info(),
                        to: self.escrow_tokens.to_account_info(),
                        authority: self.escrow_owner.to_account_info(),
                    },
                ),
                amount,
            )?;
        }

        // update the escrow and locker

        let locker = &mut self.locker;
        let escrow = &mut self.escrow;
        escrow.record_lock_event(locker, amount, next_escrow_started_at, next_escrow_ends_at)?;

        emit!(LockEvent {
            locker: locker.key(),
            locker_supply: locker.locked_supply,
            escrow_owner: escrow.owner,
            token_mint: locker.token_mint,
            amount,
            duration,
            prev_escrow_ends_at,
            next_escrow_ends_at,
            next_escrow_started_at,
        });

        Ok(())
    }

    pub fn check_whitelisted(&self, ra: &[AccountInfo]) -> ProgramResult {
        invariant!(
            ra.len() == 2,
            "program whitelist enabled; please provide whitelist entry and instructions sysvar"
        );
        let accounts_iter = &mut ra.iter();
        let ix_sysvar_account_info = next_account_info(accounts_iter)?;
        let program_id = get_instruction_relative(0, ix_sysvar_account_info)?.program_id;
        if program_id == crate::ID {
            return Ok(());
        }

        let whitelist_entry_account_info = next_account_info(accounts_iter)?;
        invariant!(
            !whitelist_entry_account_info.data_is_empty(),
            ProgramNotWhitelisted
        );
        let whitelist_entry =
            Account::<LockerWhitelistEntry>::try_from(whitelist_entry_account_info)?;
        assert_keys_eq!(whitelist_entry.locker, self.locker);
        assert_keys_eq!(whitelist_entry.program_id, program_id);

        Ok(())
    }
}

impl<'info> Validate<'info> for Lock<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.locker, self.escrow.locker);
        assert_keys_eq!(self.escrow.tokens, self.escrow_tokens);
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);
        assert_keys_eq!(self.escrow_owner, self.source_tokens.owner);

        Ok(())
    }
}
