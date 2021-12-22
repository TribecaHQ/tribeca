//! Locker math.
#![deny(clippy::integer_arithmetic)]

use crate::*;
use num_traits::ToPrimitive;

impl LockerParams {
    /// Calculates the amount of voting power an [Escrow] has.
    pub fn calculate_voter_power(&self, escrow: &Escrow, now: i64) -> Option<u64> {
        // invalid `now` argument, should never happen.
        if now == 0 {
            return None;
        }
        if escrow.escrow_started_at == 0 {
            return Some(0);
        }
        // Lockup had zero power before the start time.
        // at the end time, lockup also has zero power.
        if now < escrow.escrow_started_at || now >= escrow.escrow_ends_at {
            return Some(0);
        }

        let seconds_until_lockup_expiry = escrow.escrow_ends_at.checked_sub(now)?;
        // elapsed seconds, clamped to the maximum duration
        let relevant_seconds_until_lockup_expiry = seconds_until_lockup_expiry
            .to_u64()?
            .min(self.max_stake_duration);

        // voting power at max lockup
        let power_if_max_lockup = escrow
            .amount
            .checked_mul(self.max_stake_vote_multiplier.into())?;

        // multiply the max lockup power by the fraction of the max stake duration
        let power = (power_if_max_lockup as u128)
            .checked_mul(relevant_seconds_until_lockup_expiry.into())?
            .checked_div(self.max_stake_duration.into())?
            .to_u64()?;

        Some(power)
    }
}

#[cfg(test)]
#[allow(clippy::integer_arithmetic)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const ONE_DAY: u64 = 24 * 60 * 60;
    const ONE_YEAR: u64 = 365 * ONE_DAY;
    /// Maximum seconds elapsed between two checkpoints.
    /// [i32::MAX] corresponds to about 70 years.
    const MAX_SECONDS_BETWEEN_CHECKPOINTS: i64 = i32::MAX as i64;

    prop_compose! {
        pub fn total_and_intermediate_ts()(
          elapsed_seconds in 0..MAX_SECONDS_BETWEEN_CHECKPOINTS,
          last_checkpoint_ts in 0..(i64::MAX - MAX_SECONDS_BETWEEN_CHECKPOINTS),
        ) -> (i64, i64) {
          (last_checkpoint_ts + elapsed_seconds, last_checkpoint_ts)
       }
    }

    #[test]
    fn test_max_lockup() {
        let locker_params = &LockerParams {
            max_stake_duration: 4 * ONE_YEAR,
            max_stake_vote_multiplier: 10,
            ..LockerParams::default()
        };
        let escrow = Escrow {
            escrow_started_at: 100,
            escrow_ends_at: (100 + locker_params.max_stake_duration).to_i64().unwrap(),
            amount: 100_000,
            ..Escrow::default()
        };
        let power = locker_params.calculate_voter_power(&escrow, 100).unwrap();
        assert_eq!(
            power,
            escrow.amount * locker_params.max_stake_vote_multiplier as u64,
            "full power"
        );

        assert_eq!(
            locker_params
                .calculate_voter_power(&escrow, 100 + 2 * ONE_YEAR as i64)
                .unwrap(),
            escrow.amount * locker_params.max_stake_vote_multiplier as u64 / 2,
            "half lockup"
        );

        assert_eq!(
            locker_params
                .calculate_voter_power(&escrow, 100 + 5 * ONE_YEAR as i64)
                .unwrap(),
            0,
            "expired lockup"
        );
    }

    #[test]
    fn test_decreased_max_lockup() {
        // if the max lockup is decreased later, the user should keep their lockup parameters
        // but only be subject to the max.
        let locker_params = &LockerParams {
            max_stake_duration: 4 * ONE_YEAR,
            max_stake_vote_multiplier: 10,
            ..LockerParams::default()
        };
        let escrow = Escrow {
            escrow_started_at: 100,
            escrow_ends_at: 1_000_000 + (locker_params.max_stake_duration).to_i64().unwrap(),
            amount: 100_000,
            ..Escrow::default()
        };
        let power = locker_params.calculate_voter_power(&escrow, 600).unwrap();
        assert_eq!(
            power,
            escrow.amount * locker_params.max_stake_vote_multiplier as u64
        );
    }

    proptest! {
        #[test]
        fn test_lockup_with_zero_balance(
            current_ts in 0..=i64::MAX,
            max_stake_vote_multiplier in 0..=u8::MAX,
            max_stake_duration_years in 0..=10u64,
            (escrow_ends_at, escrow_started_at) in total_and_intermediate_ts(),
        ) {
            let locker_params = &LockerParams {
                max_stake_duration: max_stake_duration_years * ONE_YEAR,
                max_stake_vote_multiplier,
                ..LockerParams::default()
            };
            let escrow = Escrow {
                escrow_started_at,
                escrow_ends_at,
                amount: 0,
                ..Escrow::default()
            };
            let power = locker_params.calculate_voter_power(&escrow, current_ts).unwrap();
            assert_eq!(power, 0);
        }
    }
}
