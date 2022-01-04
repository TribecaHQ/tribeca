use crate::*;

impl<'info> SetVoteDelegate<'info> {
    pub fn set_vote_delegate(&mut self, new_delegate: Pubkey) -> ProgramResult {
        let old_delegate = self.escrow.vote_delegate;
        self.escrow.vote_delegate = new_delegate;

        emit!(SetVoteDelegateEvent {
            escrow_owner: self.escrow.owner,
            old_delegate,
            new_delegate,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for SetVoteDelegate<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);

        Ok(())
    }
}
