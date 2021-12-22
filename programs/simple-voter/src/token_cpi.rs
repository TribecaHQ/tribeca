use crate::token_record_signer_seeds;
use crate::TokenContext;
use anchor_lang::{prelude::ProgramResult, CpiContext, ToAccountInfo};
use anchor_spl::token;

impl TokenContext<'_> {
    pub fn transfer_to_vault(&self, amount: u64) -> ProgramResult {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.gov_token_account.to_account_info().clone(),
                to: self.gov_token_vault.to_account_info().clone(),
                authority: self.authority.to_account_info().clone(),
            },
        );
        token::transfer(cpi_ctx, amount)
    }

    pub fn transfer_from_vault(&self, amount: u64) -> ProgramResult {
        let seeds = token_record_signer_seeds!(self.token_record);
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.gov_token_vault.to_account_info().clone(),
                to: self.gov_token_account.to_account_info().clone(),
                authority: self.token_record.to_account_info().clone(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx, amount)
    }
}
