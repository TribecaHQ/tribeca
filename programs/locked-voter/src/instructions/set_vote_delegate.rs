use crate::*;

impl<'info> SetVoteDelegate<'info> {
    pub fn set_vote_delegate(&mut self) -> ProgramResult {
        self.escrow.vote_delegate = self.vote_delegate.key();

        Ok(())
    }
}

impl<'info> Validate<'info> for SetVoteDelegate<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);

        Ok(())
    }
}
