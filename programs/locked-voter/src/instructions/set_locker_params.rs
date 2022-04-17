//! Instruction handler for [locked_voter::set_locker_params].

use crate::*;

/// Accounts for [locked_voter::set_locker_params].
#[derive(Accounts)]
pub struct SetLockerParams<'info> {
    /// The [Locker].
    #[account(mut)]
    pub locker: Account<'info, Locker>,
    /// The [Governor].
    pub governor: Account<'info, Governor>,
    /// The smart wallet on the [Governor].
    pub smart_wallet: Signer<'info>,
}

impl<'info> SetLockerParams<'info> {
    pub fn set_locker_params(&mut self, params: LockerParams) -> Result<()> {
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
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.governor, self.locker.governor, "governor mismatch");
        assert_keys_eq!(self.smart_wallet, self.governor.smart_wallet);
        Ok(())
    }
}

/// Event called in [locked_voter::set_locker_params].
#[event]
pub struct LockerSetParamsEvent {
    /// The [Locker].
    #[index]
    pub locker: Pubkey,
    /// Previous [LockerParams].
    pub prev_params: LockerParams,
    /// New [LockerParams].
    pub params: LockerParams,
}
