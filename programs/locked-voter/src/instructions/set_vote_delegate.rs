use crate::*;

impl<'info> SetVoteDelegate<'info> {
    pub fn set_vote_delegate(&mut self, new_delegate: Pubkey) -> ProgramResult {
        self.escrow.vote_delegate = new_delegate;

        Ok(())
    }
}

impl<'info> Validate<'info> for SetVoteDelegate<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);

        Ok(())
    }
}
