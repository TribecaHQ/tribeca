use crate::*;

/// Accounts for set_vote_delegate
#[derive(Accounts)]
pub struct SetVoteDelegate<'info> {
    /// The [Escrow].
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
    /// The owner of the [Escrow].
    pub escrow_owner: Signer<'info>,
}

impl<'info> SetVoteDelegate<'info> {
    pub fn set_vote_delegate(&mut self, new_delegate: Pubkey) -> Result<()> {
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
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.escrow.owner, self.escrow_owner);

        Ok(())
    }
}

#[event]
/// Event called in [locked_voter::set_vote_delegate].
pub struct SetVoteDelegateEvent {
    /// The owner of the Escrow.
    #[index]
    pub escrow_owner: Pubkey,
    /// The old escrow delegate.
    pub old_delegate: Pubkey,
    /// The new escrow delegate.
    pub new_delegate: Pubkey,
}
