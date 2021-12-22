use crate::*;

impl<'info> ApproveProgramLockPrivilege<'info> {
    /// Creates a new [LockerWhitelistEntry].
    pub fn approve_program_lock_privilege(&mut self, bump: u8) -> ProgramResult {
        let whitelist_entry = &mut self.whitelist_entry;
        whitelist_entry.bump = bump;
        whitelist_entry.locker = self.locker.key();
        whitelist_entry.program_id = self.executable_id.key();

        emit!(ApproveLockPrivilegeEvent {
            locker: whitelist_entry.locker,
            program_id: whitelist_entry.program_id,
            timestamp: Clock::get()?.unix_timestamp
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for ApproveProgramLockPrivilege<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.governor.smart_wallet, self.smart_wallet);
        invariant!(
            self.executable_id.executable,
            "program_id must be an executable"
        );

        Ok(())
    }
}

impl<'info> RevokeProgramLockPrivilege<'info> {
    /// Emit event that [LockerWhitelistEntry] was closed.
    pub fn revoke_program_lock_privilege(&mut self) -> ProgramResult {
        emit!(RevokeLockPrivilegeEvent {
            locker: self.whitelist_entry.locker,
            program_id: self.whitelist_entry.program_id,
            timestamp: Clock::get()?.unix_timestamp
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for RevokeProgramLockPrivilege<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.governor.smart_wallet, self.smart_wallet);
        assert_keys_eq!(self.whitelist_entry.program_id, self.executable_id);

        Ok(())
    }
}
