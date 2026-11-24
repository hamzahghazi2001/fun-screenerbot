// Copyright (c) 2024-2026 ScreenerBot (screenerbot.io)
// SPDX-License-Identifier: BUSL-1.1

//! F3 money math, pure: dynamic entry size as a percent of the SOL balance
//! (clamped floor..ceiling) and the slot cap that adds a third slot above a USD
//! balance threshold. No I/O; callers pass the cached balance and SOL/USD price.

use anyhow::{bail, Result};

use crate::config::TraderConfig;

/// SOL to spend on one automatic entry: `balance * dynamic_size_pct / 100`,
/// clamped to `[dynamic_size_floor_sol, dynamic_size_ceiling_sol]`.
///
/// Errors on an unknown, NaN, infinite or non-positive balance, on invalid sizing
/// config, and when the clamped size would exceed the balance. Does not read
/// `dynamic_size_enabled`; the caller decides whether dynamic sizing applies.
pub fn dynamic_trade_size(balance_sol: Option<f64>, cfg: &TraderConfig) -> Result<f64> {
    let Some(balance) = balance_sol else {
        bail!("dynamic sizing: SOL balance unknown");
    };
    if !balance.is_finite() || balance <= 0.0 {
        bail!("dynamic sizing: invalid SOL balance {balance}");
    }

    let pct = cfg.dynamic_size_pct;
    if !pct.is_finite() || pct <= 0.0 || pct > 100.0 {
        bail!("dynamic sizing: dynamic_size_pct {pct} outside (0, 100]");
    }

    let floor = cfg.dynamic_size_floor_sol;
    let ceiling = cfg.dynamic_size_ceiling_sol;
    if !floor.is_finite() || !ceiling.is_finite() || floor <= 0.0 || floor > ceiling {
        bail!("dynamic sizing: invalid floor {floor} / ceiling {ceiling}");
    }

    let size = (balance * pct / 100.0).clamp(floor, ceiling);
    if !size.is_finite() {
        bail!("dynamic sizing: non-finite size");
    }
    if size > balance {
        bail!("dynamic sizing: size {size} SOL exceeds balance {balance} SOL");
    }
    Ok(size)
}

/// Most open positions allowed for this balance.
///
/// `max(max_open_positions, third_slot_max_positions)` when `balance * sol_usd`
/// is at or above `third_slot_threshold_usd`; otherwise, or when the balance,
/// price or threshold is missing / NaN / non-positive, `max_open_positions`.
/// Does not read `dynamic_size_enabled`; the caller decides.
pub fn slot_cap(balance_sol: Option<f64>, sol_usd: Option<f64>, cfg: &TraderConfig) -> usize {
    let base = cfg.max_open_positions;
    let (Some(balance), Some(price)) = (balance_sol, sol_usd) else {
        return base;
    };
    if !balance.is_finite() || balance <= 0.0 || !price.is_finite() || price <= 0.0 {
        return base;
    }
    let threshold = cfg.third_slot_threshold_usd;
    let usd = balance * price;
    if !usd.is_finite() || !threshold.is_finite() {
        return base;
    }
    if usd >= threshold {
        base.max(cfg.third_slot_max_positions)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Upstream defaults: pct 10, floor 0.001, ceiling 10, threshold $40, 2 → 3 slots.
    fn cfg() -> TraderConfig {
        TraderConfig::default()
    }

    #[test]
    fn size_is_pct_of_balance() {
        let size = dynamic_trade_size(Some(0.5), &cfg()).unwrap();
        assert!((size - 0.05).abs() < 1e-12, "got {size}");
    }

    #[test]
    fn size_floor_applies() {
        // 10% of 0.005 = 0.0005 < floor 0.001
        assert_eq!(dynamic_trade_size(Some(0.005), &cfg()).unwrap(), 0.001);
    }

    #[test]
    fn size_ceiling_applies() {
        // 10% of 500 = 50 > ceiling 10
        assert_eq!(dynamic_trade_size(Some(500.0), &cfg()).unwrap(), 10.0);
    }

    #[test]
    fn size_nan_balance_err() {
        assert!(dynamic_trade_size(Some(f64::NAN), &cfg()).is_err());
        assert!(dynamic_trade_size(Some(f64::INFINITY), &cfg()).is_err());
    }

    #[test]
    fn size_unknown_balance_err() {
        assert!(dynamic_trade_size(None, &cfg()).is_err());
    }

    #[test]
    fn size_zero_or_negative_balance_err() {
        assert!(dynamic_trade_size(Some(0.0), &cfg()).is_err());
        assert!(dynamic_trade_size(Some(-1.0), &cfg()).is_err());
    }

    #[test]
    fn size_above_balance_err() {
        // floor 0.001 > balance 0.0005
        assert!(dynamic_trade_size(Some(0.0005), &cfg()).is_err());
        // exactly the floor is fine
        assert_eq!(dynamic_trade_size(Some(0.001), &cfg()).unwrap(), 0.001);
    }

    #[test]
    fn size_bad_config_err() {
        let bad: [fn(&mut TraderConfig); 7] = [
            |c| c.dynamic_size_pct = f64::NAN,
            |c| c.dynamic_size_pct = 0.0,
            |c| c.dynamic_size_pct = 100.1,
            |c| c.dynamic_size_floor_sol = f64::NAN,
            |c| c.dynamic_size_floor_sol = 0.0,
            |c| c.dynamic_size_ceiling_sol = f64::INFINITY,
            |c| {
                c.dynamic_size_floor_sol = 1.0;
                c.dynamic_size_ceiling_sol = 0.5;
            },
        ];
        for (i, mutate) in bad.iter().enumerate() {
            let mut c = cfg();
            mutate(&mut c);
            assert!(dynamic_trade_size(Some(5.0), &c).is_err(), "case {i}");
        }
    }

    #[test]
    fn slot_flip_exactly_40_usd() {
        // 0.25 SOL * $160 = exactly $40.0 in f64
        assert_eq!(slot_cap(Some(0.25), Some(160.0), &cfg()), 3);
    }

    #[test]
    fn slot_below_40_usd_is_2() {
        assert_eq!(slot_cap(Some(0.25), Some(159.99), &cfg()), 2);
        assert_eq!(slot_cap(Some(0.1), Some(100.0), &cfg()), 2);
    }

    #[test]
    fn missing_price_gives_2_slots() {
        assert_eq!(slot_cap(Some(10.0), None, &cfg()), 2);
    }

    #[test]
    fn slot_bad_price_or_balance_gives_base() {
        let c = cfg();
        assert_eq!(slot_cap(None, Some(200.0), &c), 2);
        assert_eq!(slot_cap(Some(10.0), Some(f64::NAN), &c), 2);
        assert_eq!(slot_cap(Some(10.0), Some(0.0), &c), 2);
        assert_eq!(slot_cap(Some(10.0), Some(-1.0), &c), 2);
        assert_eq!(slot_cap(Some(f64::NAN), Some(200.0), &c), 2);
        assert_eq!(slot_cap(Some(f64::MAX), Some(f64::MAX), &c), 2);
        let mut nan_threshold = cfg();
        nan_threshold.third_slot_threshold_usd = f64::NAN;
        assert_eq!(slot_cap(Some(10.0), Some(200.0), &nan_threshold), 2);
    }

    #[test]
    fn slot_cap_never_below_base() {
        let mut c = cfg();
        c.max_open_positions = 5;
        assert_eq!(slot_cap(Some(10.0), Some(200.0), &c), 5);
        assert_eq!(slot_cap(Some(0.01), Some(200.0), &c), 5);
    }
}
