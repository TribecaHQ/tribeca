//! Instruction processors.

pub mod activate_proposal;
pub mod cast_vote;
pub mod exit;
pub mod lock;
pub mod new_escrow;
pub mod new_locker;
pub mod whitelist;

pub use activate_proposal::*;
pub use cast_vote::*;
pub use exit::*;
pub use lock::*;
pub use new_escrow::*;
pub use new_locker::*;
pub use whitelist::*;
