use anchor_lang::{prelude::*, solana_program::pubkey::PUBKEY_BYTES};

#[account]
#[derive(Copy, Debug, Default)]
pub struct Electorate {
    /// TODO(michael): Docs
    pub bump: u8,
    /// TODO(michael): Docs
    pub base: Pubkey,
    /// TODO(michael): Docs
    pub governor: Pubkey,
    /// TODO(michael): Docs
    pub gov_token_mint: Pubkey,
    /// The number of votes required in order for a voter to activate a proposal
    pub proposal_threshold: u64,
}

impl Electorate {
    pub const LEN: usize = 1 + PUBKEY_BYTES * 3 + 8;
}

#[account]
#[derive(Copy, Debug, Default)]
pub struct TokenRecord {
    /// TODO(michael): Docs
    pub bump: u8,
    /// TODO(michael): Docs
    pub authority: Pubkey,
    /// TODO(michael): Docs
    pub electorate: Pubkey,
    /// TODO(michael): Docs
    pub token_vault_key: Pubkey,
    /// TODO(michael): Docs
    pub balance: u64,
    /// TODO(michael): Docs
    pub unfinalized_votes: u64,
}

impl TokenRecord {
    pub const LEN: usize = 1 + PUBKEY_BYTES * 3 + 8 + 8;
}
