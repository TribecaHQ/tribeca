use crate::*;

use anchor_lang::{
    solana_program::sysvar, solana_program::sysvar::instructions::get_instruction_relative,
    system_program,
};

#[derive(Accounts)]
pub struct LockWithWhitelistEntry<'info> {
    pub lock: Lock<'info>,
    /// CHECK: The instructions sysvar.
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    pub whitelist_entry: Account<'info, LockerWhitelistEntry>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, LockWithWhitelistEntry<'info>>,
    amount: u64,
    duration: i64,
) -> Result<()> {
    ctx.accounts.lock.lock(amount, duration)
}

impl<'info> Validate<'info> for LockWithWhitelistEntry<'info> {
    fn validate(&self) -> Result<()> {
        self.lock.validate()?;
        invariant!(self.lock.locker.params.whitelist_enabled);
        assert_keys_eq!(self.whitelist_entry.locker, self.lock.locker);

        invariant!(sysvar::instructions::check_id(
            &self.instructions_sysvar.key()
        ));

        let program_id = get_instruction_relative(0, &self.instructions_sysvar)?.program_id;
        if program_id == crate::ID {
            return Ok(());
        }

        if self.whitelist_entry.owner != system_program::ID {
            assert_keys_eq!(
                self.whitelist_entry.owner,
                self.lock.escrow_owner,
                EscrowOwnerNotWhitelisted
            );
        }
        assert_keys_eq!(self.whitelist_entry.program_id, program_id);

        Ok(())
    }
}
