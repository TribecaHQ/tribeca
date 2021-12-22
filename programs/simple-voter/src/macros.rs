//! Macros.

/// Generates the signer seeds for a ....
#[macro_export]
macro_rules! electorate_seeds {
    ($electorate: expr) => {
        &[&[
            b"SimpleElectorate" as &[u8],
            &$electorate.base.to_bytes(),
            &[$electorate.bump],
        ]]
    };
}

/// Generates the signer seeds for a ....
#[macro_export]
macro_rules! token_record_signer_seeds {
    ($token_record:expr) => {
        &[
            b"SimpleTokenRecord" as &[u8],
            &$token_record.authority.to_bytes(),
            &$token_record.electorate.to_bytes(),
            &[$token_record.bump],
        ]
    };
}
