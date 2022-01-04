//! Events for the [locked-voter] program.

use crate::*;

#[event]
/// Event called in [locked_voter::new_locker].
pub struct NewLockerEvent {
    /// The governor for the [Locker].
    #[index]
    pub governor: Pubkey,
    /// The [Locker] being created.
    pub locker: Pubkey,
    /// Mint of the token that can be used to join the [Locker].
    pub token_mint: Pubkey,
    /// New [LockerParams].
    pub params: LockerParams,
}

#[event]
/// Event called in [locked_voter::new_escrow].
pub struct NewEscrowEvent {
    /// The [Escrow] being created.
    pub escrow: Pubkey,
    /// The owner of the [Escrow].
    #[index]
    pub escrow_owner: Pubkey,
    /// The locker for the [Escrow].
    #[index]
    pub locker: Pubkey,
    /// Timestamp for the event.
    pub timestamp: i64,
}

#[event]
/// Event called in [locked_voter::exit].
pub struct ExitEscrowEvent {
    /// The owner of the [Escrow].
    #[index]
    pub escrow_owner: Pubkey,
    /// The locker for the [Escrow].
    #[index]
    pub locker: Pubkey,
    /// Timestamp for the event.
    pub timestamp: i64,
    /// The amount of tokens locked inside the [Locker].
    pub locker_supply: u64,
    /// The amount released from the [Escrow].
    pub released_amount: u64,
}

#[event]
/// Event called in [locked_voter::lock].
pub struct LockEvent {
    /// The locker of the [Escrow]
    #[index]
    pub locker: Pubkey,
    /// The owner of the [Escrow].
    #[index]
    pub escrow_owner: Pubkey,
    /// Mint of the token that for the [Locker].
    pub token_mint: Pubkey,
    /// Amount of tokens locked.
    pub amount: u64,
    /// Amount of tokens locked inside the [Locker].
    pub locker_supply: u64,
    /// Duration of lock time.
    pub duration: i64,
    /// The previous timestamp that the [Escrow] ended at.
    pub prev_escrow_ends_at: i64,
    /// The new [Escrow] end time.
    pub next_escrow_ends_at: i64,
    /// The new [Escrow] start time.
    pub next_escrow_started_at: i64,
}

#[event]
/// Event called in [locked_voter::approve_program_lock_privilege].
pub struct ApproveLockPrivilegeEvent {
    /// The [Locker].
    #[index]
    pub locker: Pubkey,
    /// ProgramId approved to make CPI calls to [locked_voter::lock].
    pub program_id: Pubkey,
    /// Timestamp of the event.
    pub timestamp: i64,
}

#[event]
/// Event called in [locked_voter::revoke_program_lock_privilege].
pub struct RevokeLockPrivilegeEvent {
    /// The [Locker].
    #[index]
    pub locker: Pubkey,
    /// ProgramId approved to make CPI calls to [locked_voter::lock].
    pub program_id: Pubkey,
    /// Timestamp of the event.
    pub timestamp: i64,
}

/// Event called in [locked_voter::set_locker_params].
#[event]
pub struct LockerSetParamsEvent {
    /// The [Locker].
    #[index]
    pub locker: Pubkey,
    /// Previous [LockerParams].
    pub prev_params: LockerParams,
    /// New [LockerParams].
    pub params: LockerParams,
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
