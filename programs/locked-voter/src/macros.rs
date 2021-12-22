//! Macros

/// Generates the signer seeds for a [crate::Locker].
#[macro_export]
macro_rules! locker_seeds {
    ($locker: expr) => {
        &[&[
            b"Locker" as &[u8],
            &$locker.base.to_bytes(),
            &[$locker.bump],
        ]]
    };
}

/// Generates the signer seeds for an [crate::Escrow].
#[macro_export]
macro_rules! escrow_seeds {
    ($escrow: expr) => {
        &[&[
            b"Escrow" as &[u8],
            &$escrow.locker.to_bytes(),
            &$escrow.owner.to_bytes(),
            &[$escrow.bump],
        ]]
    };
}
