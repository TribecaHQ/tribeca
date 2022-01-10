//! Instruction processors.

mod activate_proposal;
mod cast_vote;
mod exit;
mod lock;
mod new_escrow;
mod new_locker;
mod whitelist;

pub use activate_proposal::*;
pub use cast_vote::*;
pub use exit::*;
pub use lock::*;
pub use new_escrow::*;
pub use new_locker::*;
pub use whitelist::*;
