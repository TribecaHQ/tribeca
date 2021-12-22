use crate::*;
use vipers::Validate;

impl<'info> Validate<'info> for InitializeElectorate<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for InitializeTokenRecord<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for ActivateProposal<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for TokenContext<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for VoterContext<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}
