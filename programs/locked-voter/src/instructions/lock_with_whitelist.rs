use crate::*;

use anchor_lang::{
    solana_program::sysvar, solana_program::sysvar::instructions::get_instruction_relative,
};

#[derive(Accounts)]
pub struct LockWithWhitelist<'info> {
    pub lock: Lock<'info>,
    /// CHECK: The instructions sysvar.
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, LockWithWhitelist<'info>>,
    amount: u64,
    duration: i64,
) -> Result<()> {
    ctx.accounts.lock.lock(amount, duration)
}

impl<'info> Validate<'info> for LockWithWhitelist<'info> {
    fn validate(&self) -> Result<()> {
        self.lock.validate()?;
        invariant!(self.lock.locker.params.whitelist_enabled);

        invariant!(sysvar::instructions::check_id(
            &self.instructions_sysvar.key()
        ));

        let program_id = get_instruction_relative(0, &self.instructions_sysvar)?.program_id;
        invariant!(program_id == crate::ID, MustCallLockWithWhitelistEntry);
        Ok(())
    }
}
