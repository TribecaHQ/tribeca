//! Instruction processors.

pub mod create_governor;
pub mod activate_proposal;
pub mod cancel_proposal;
pub mod create_governor;
pub mod new_vote;
pub mod proposal_create_event;
pub mod queue_proposal;

pub use create_governor::*;
pub use activate_proposal::*;
pub use cancel_proposal::*;
pub use create_governor::*;
pub use new_vote::*;
pub use proposal_create_event::*;
pub use queue_proposal::*;
