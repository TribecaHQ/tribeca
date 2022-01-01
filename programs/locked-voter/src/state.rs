//! State accounts.
#![deny(missing_docs)]

use crate::*;

/// A group of [Escrow]s.
#[account]
#[derive(Copy, Debug, Default)]
pub struct Locker {
    /// Base account used to generate signer seeds.
    pub base: Pubkey,
    /// Bump seed.
    pub bump: u8,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Pubkey,
    /// Total number of tokens locked in [Escrow]s.
    pub locked_supply: u64,
    /// Governor associated with the [Locker].
    pub governor: Pubkey,
    /// Mutable parameters of how a [Locker] should behave.
    pub params: LockerParams,
}

/// Contains parameters for the [Locker].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LockerParams {
    /// Whether or not the locking whitelist system is enabled.
    pub whitelist_enabled: bool,
    /// The weight of a maximum vote lock relative to the total number of tokens locked.
    /// For example, veCRV is 10 because 1 CRV locked for 4 years = 10 veCRV.
    pub max_stake_vote_multiplier: u8,
    /// Minimum staking duration.
    pub min_stake_duration: u64,
    /// Maximum staking duration.
    pub max_stake_duration: u64,
    /// Minimum number of votes required to activate a proposal.
    pub proposal_activation_min_votes: u64,
}

/// An entry in the [Locker]'s whitelist.
///
/// The whitelist controls which programs are allowed to stake tokens into the system.
#[account]
#[derive(Copy, Debug, Default)]
pub struct LockerWhitelistEntry {
    /// Bump seed.
    pub bump: u8,
    /// [Locker] this whitelist entry belongs to.
    pub locker: Pubkey,
    /// Key of program_id allowed to be owner of the [Escrow::owner] account.
    pub program_id: Pubkey,
}

/// Locks tokens on behalf of a user.
#[account]
#[derive(Copy, Debug, Default)]
pub struct Escrow {
    /// The [Locker] that this [Escrow] is part of.
    pub locker: Pubkey,
    /// The key of the account that is authorized to stake into/withdraw from this [Escrow].
    pub owner: Pubkey,
    /// Bump seed.
    pub bump: u8,

    /// The token account holding the escrow tokens.
    pub tokens: Pubkey,
    /// Amount of tokens staked.
    pub amount: u64,
    /// When the [Escrow::owner] started their escrow.
    pub escrow_started_at: i64,
    /// When the escrow unlocks; i.e. the [Escrow::owner] is scheduled to be allowed to withdraw their tokens.
    pub escrow_ends_at: i64,

    /// Account that is authorized to vote on behalf of this [Escrow].
    /// Defaults to the [Escrow::owner].
    pub vote_delegate: Pubkey,
}

impl Escrow {
    /// Gets the amount of voting power the [Escrow] will have at the given time.
    pub fn voting_power_at_time(&self, locker: &LockerParams, timestamp: i64) -> Option<u64> {
        locker.calculate_voter_power(self, timestamp)
    }

    /// Gets the amount of voting power the [Escrow] currently has.
    pub fn voting_power(&self, locker: &LockerParams) -> Result<u64> {
        Ok(unwrap_int!(self.voting_power_at_time(
            locker,
            Clock::get()?.unix_timestamp
        )))
    }

    /// Update the escrow and its locker to account for a lock event.
    pub fn record_lock_event(
        &mut self,
        locker: &mut Locker,
        lock_amount: u64,
        next_escrow_started_at: i64,
        next_escrow_ends_at: i64,
    ) -> Result<()> {
        self.amount = unwrap_int!(self.amount.checked_add(lock_amount));
        self.escrow_started_at = next_escrow_started_at;
        self.escrow_ends_at = next_escrow_ends_at;

        locker.locked_supply = unwrap_int!(locker.locked_supply.checked_add(lock_amount));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOURS_PER_DAY: i64 = 24;
    const DAYS_PER_WEEK: i64 = 7;
    const DAYS_PER_YEAR: i64 = 365;

    const HOUR: i64 = 3600;
    const DAY: i64 = HOURS_PER_DAY * HOUR;
    const WEEK: i64 = DAYS_PER_WEEK * DAY;

    const MAX_TIME: i64 = 4 * DAYS_PER_YEAR * DAY; // 126144000
    const CANONICAL_START_TIME: i64 = 1635379200;

    const DEFAULT_STAKE_MULTIPLIER: u8 = 1;
    const DEFAULT_LOCK_AMOUNT: u64 = 1_000_000_000_000_000;

    fn reset_escrow(locker: &mut Locker, escrow: &mut Escrow) {
        locker.locked_supply -= escrow.amount;

        escrow.amount = 0;
        escrow.escrow_started_at = 0;
        escrow.escrow_ends_at = 0;
    }

    fn assert_escrow(
        locker_params: &LockerParams,
        escrow: &Escrow,
        current_ts: i64,
        expected_amount: f64,
    ) {
        let actual_amount = escrow
            .voting_power_at_time(locker_params, current_ts)
            .unwrap();
        if actual_amount == 0 && expected_amount == 0.0 {
            assert_eq!(actual_amount, expected_amount as u64);
        } else {
            let precision = 120.0 / WEEK as f64;
            let actual_f64 = actual_amount as f64;

            assert!(
                2.0 * ((actual_f64 - expected_amount).abs() / (actual_f64 + expected_amount))
                    <= precision,
                "actual: {}, expected: {}, precision: {}",
                actual_f64,
                expected_amount,
                precision
            );
        }
    }

    #[test]
    fn test_voting_powers_one_week() {
        let locker = &mut Locker {
            params: LockerParams {
                max_stake_duration: MAX_TIME as u64,
                max_stake_vote_multiplier: DEFAULT_STAKE_MULTIPLIER,
                ..LockerParams::default()
            },
            ..Locker::default()
        };
        let alice = &mut Escrow::default();

        let mut current_ts = CANONICAL_START_TIME;
        assert_eq!(locker.locked_supply, 0);
        assert_escrow(&locker.params, alice, current_ts, 0.0);

        current_ts += HOUR;
        // Alice deposits and locks for 1 week
        alice
            .record_lock_event(locker, DEFAULT_LOCK_AMOUNT, current_ts, current_ts + WEEK)
            .unwrap();
        assert_eq!(locker.locked_supply, alice.amount);

        current_ts += HOUR;
        let expected_amount: f64 =
            DEFAULT_LOCK_AMOUNT as f64 / MAX_TIME as f64 * (WEEK - HOUR) as f64;
        assert_escrow(&locker.params, alice, current_ts, expected_amount);

        let t0 = current_ts;
        for _ in 0..DAYS_PER_WEEK {
            for _ in 0..HOURS_PER_DAY {
                current_ts += HOUR;
            }

            let dt = current_ts - t0;
            assert_eq!(locker.locked_supply, alice.amount);
            let expected_amount: f64 =
                DEFAULT_LOCK_AMOUNT as f64 / MAX_TIME as f64 * (WEEK - HOUR - dt) as f64;
            assert_escrow(&locker.params, alice, current_ts, expected_amount);
        }

        current_ts += HOUR;
        assert_escrow(&locker.params, alice, current_ts, 0.0);

        // Alice exits from escrow
        reset_escrow(locker, alice);
        assert_eq!(locker.locked_supply, 0);
        assert_escrow(&locker.params, alice, current_ts, 0.0);
    }

    // Test voting power in the following scenario:
    // Alice:
    // ~~~~~~~
    // ^
    // | *
    // | |  \
    // | |    \
    //-+-------+---> t
    // Bob:
    // ~~~~~~~
    // ^
    // |*
    // || \
    // ||  \
    // +----+---> t
    #[test]
    fn test_voting_powers_two_users() {
        let locker = &mut Locker {
            params: LockerParams {
                max_stake_duration: MAX_TIME as u64,
                max_stake_vote_multiplier: DEFAULT_STAKE_MULTIPLIER,
                ..LockerParams::default()
            },
            ..Locker::default()
        };
        let alice = &mut Escrow::default();
        let bob = &mut Escrow::default();

        let mut current_ts = CANONICAL_START_TIME;
        assert_eq!(locker.locked_supply, 0);
        assert_escrow(&locker.params, alice, current_ts, 0.0);
        assert_escrow(&locker.params, bob, current_ts, 0.0);

        current_ts += HOUR;

        // Alice deposits and locks for 2 weeks
        alice
            .record_lock_event(
                locker,
                DEFAULT_LOCK_AMOUNT,
                current_ts,
                current_ts + 2 * WEEK,
            )
            .unwrap();
        let expected_amount = alice.amount as f64 / MAX_TIME as f64 * 2.0 * WEEK as f64;
        assert_escrow(&locker.params, alice, current_ts, expected_amount);
        // Bob deposits and locks for 1 week
        bob.record_lock_event(locker, DEFAULT_LOCK_AMOUNT, current_ts, current_ts + WEEK)
            .unwrap();
        let expected_amount = bob.amount as f64 / MAX_TIME as f64 * WEEK as f64;
        assert_escrow(&locker.params, bob, current_ts, expected_amount);
        assert_eq!(locker.locked_supply, bob.amount + alice.amount);

        let t0 = current_ts;
        for _ in 0..DAYS_PER_WEEK {
            for _ in 0..HOURS_PER_DAY {
                current_ts += HOUR;
            }
            let dt = current_ts - t0;
            let expected_alice_amount =
                DEFAULT_LOCK_AMOUNT as f64 / MAX_TIME as f64 * (2 * WEEK - dt) as f64;
            assert_escrow(&locker.params, alice, current_ts, expected_alice_amount);
            let expected_bob_amount =
                DEFAULT_LOCK_AMOUNT as f64 / MAX_TIME as f64 * (WEEK - dt) as f64;
            assert_escrow(&locker.params, bob, current_ts, expected_bob_amount);
            assert_eq!(locker.locked_supply, bob.amount + alice.amount);
        }

        // Alice should have half vote power ...
        // Bob's vote power should have expired ...
        let t0 = current_ts;
        for _ in 0..DAYS_PER_WEEK {
            for _ in 0..HOURS_PER_DAY {
                current_ts += HOUR;
            }

            let dt = current_ts - t0;
            let expected_alice_amount =
                DEFAULT_LOCK_AMOUNT as f64 / MAX_TIME as f64 * (WEEK - dt) as f64;
            assert_escrow(&locker.params, alice, current_ts, expected_alice_amount);
            assert_escrow(&locker.params, bob, current_ts, 0.0);
            assert_eq!(locker.locked_supply, bob.amount + alice.amount);
        }

        current_ts += HOUR;
        assert_escrow(&locker.params, alice, current_ts, 0.0);
        assert_escrow(&locker.params, bob, current_ts, 0.0);

        current_ts += HOUR;
        // Alice exits from escrow
        reset_escrow(locker, alice);
        assert_eq!(locker.locked_supply, bob.amount);
        assert_escrow(&locker.params, alice, current_ts, 0.0);
        // Bob exits from escrow
        reset_escrow(locker, bob);
        assert_eq!(locker.locked_supply, 0);
        assert_escrow(&locker.params, bob, current_ts, 0.0);
    }
}
