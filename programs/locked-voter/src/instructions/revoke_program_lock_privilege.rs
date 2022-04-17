//! Handler for [locked_voter::revoke_program_lock_privilege].

use crate::*;

/// Accounts for [locked_voter::revoke_program_lock_privilege].
#[derive(Accounts)]
pub struct RevokeProgramLockPrivilege<'info> {
    /// The [Locker].
    pub locker: Account<'info, Locker>,
    /// [LockerWhitelistEntry].
    #[account(mut, has_one = locker, close = payer)]
    pub whitelist_entry: Account<'info, LockerWhitelistEntry>,

    /// Governor for the [Locker].
    pub governor: Account<'info, Governor>,

    /// Smart wallet on the [Governor].
    pub smart_wallet: Signer<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,
}

impl<'info> RevokeProgramLockPrivilege<'info> {
    /// Emit event that [LockerWhitelistEntry] was closed.
    pub fn revoke_program_lock_privilege(&mut self) -> Result<()> {
        emit!(RevokeLockPrivilegeEvent {
            locker: self.whitelist_entry.locker,
            program_id: self.whitelist_entry.program_id,
            timestamp: Clock::get()?.unix_timestamp
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for RevokeProgramLockPrivilege<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.governor.smart_wallet, self.smart_wallet);
        assert_keys_eq!(self.whitelist_entry.locker, self.locker);
        Ok(())
    }
}

#[event]
/// Event called in [locked_voter::revoke_program_lock_privilege].
pub struct RevokeLockPrivilegeEvent {
    /// The [Locker].
    #[index]
    pub locker: Pubkey,
    /// ProgramId approved to make CPI calls to [locked_voter::lock].
    pub program_id: Pubkey,
    /// Timestamp of the event.
    pub timestamp: i64,
}
