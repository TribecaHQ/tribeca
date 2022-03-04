//! Known program IDs related to Tribeca.
//!
//! This is used to allow core Tribeca protocol programs to bypass the Locker whitelist.
//!
//! # License
//!
//! Tribeca Protocol is licensed under the GNU Affero General Public License v3.0.
//!
//! In short, this means that any changes to this code must be made open source and
//! available under the AGPL-v3.0 license, even if only used privately. If you have
//! a need to use this program and cannot respect the terms of the license, please
//! message us our team directly at [team@tribeca.so](mailto:team@tribeca.so).

use solana_program::pubkey::Pubkey;
use static_pubkey::static_pubkey;

/// The [SAVE](https://github.com/TribecaHQ/save) program.
pub const SAVE_PROGRAM_ID: Pubkey = static_pubkey!("SAVEd9pHcncknnMWdP8RSbhDUhw3nrzwmZ6F6RAUiio");

/// Checks to see if the given [Pubkey] is in the `well-known` whitelist.
pub fn is_well_known(key: &Pubkey) -> bool {
    key == &SAVE_PROGRAM_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist() {
        let save_program_id_2: Pubkey =
            static_pubkey!("SAVEd9pHcncknnMWdP8RSbhDUhw3nrzwmZ6F6RAUiio");

        assert!(is_well_known(&SAVE_PROGRAM_ID));
        assert!(is_well_known(&save_program_id_2));
    }

    #[test]
    fn test_not_in_whitelist() {
        let unknown_program: Pubkey = static_pubkey!("SBVEd9pHcncknnMWdP8RSbhDUhw3nrzwmZ6F6RAUiio");

        assert!(!is_well_known(&unknown_program));
    }
}
