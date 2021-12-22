//! Macros

/// Generates the signer seeds for a Governor.
#[macro_export]
macro_rules! governor_seeds {
    ($governor: expr) => {
        &[
            b"TribecaGovernor" as &[u8],
            &$governor.base.to_bytes(),
            &[$governor.bump],
        ]
    };
}
