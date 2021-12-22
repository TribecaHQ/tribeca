use crate::*;

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
