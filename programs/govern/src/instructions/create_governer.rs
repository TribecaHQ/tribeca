use crate::*;

/// Accounts for [govern::create_governor].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateGovernor<'info> {
    /// Base of the [Governor] key.
    pub base: Signer<'info>,
    /// Governor.
    #[account(
        init,
        seeds = [
            b"TribecaGovernor".as_ref(),
            base.key().as_ref()
        ],
        bump = bump,
        payer = payer,
    )]
    pub governor: Account<'info, Governor>,
    /// The Smart Wallet.
    pub smart_wallet: Account<'info, SmartWallet>,
    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}

impl<'info> CreateGovernor<'info> {
    /// Creates a [Governor].
    pub fn create_governor(
        ctx: Context<CreateGovernor>,
        bump: u8,
        electorate: Pubkey,
        params: GovernanceParameters,
    ) -> ProgramResult {
        invariant!(
            params.timelock_delay_seconds >= 0,
            "timelock delay must be at least 0 seconds"
        );

        let governor = &mut ctx.accounts.governor;
        governor.base = ctx.accounts.base.key();
        governor.bump = bump;

        governor.proposal_count = 0;
        governor.electorate = electorate;
        governor.smart_wallet = ctx.accounts.smart_wallet.key();

        governor.params = params;

        emit!(GovernorCreateEvent {
            governor: governor.key(),
            electorate,
            smart_wallet: ctx.accounts.smart_wallet.key(),
            parameters: params,
        });

        Ok(())
    }
        
}

impl<'info> Validate<'info> for CreateGovernor<'info> {
    fn validate(&self) -> ProgramResult {
        invariant!(
            self.smart_wallet.owners.contains(&self.governor.key()),
            GovernorNotFound
        );

        Ok(())
    }
}

/// Event called in [govern::create_governor].
#[event]
pub struct GovernorCreateEvent {
    /// The governor being created.
    #[index]
    pub governor: Pubkey,
    /// The electorate of the created [Governor].
    pub electorate: Pubkey,
    /// The [SmartWallet].
    pub smart_wallet: Pubkey,
    /// Governance parameters.
    pub parameters: GovernanceParameters,
}