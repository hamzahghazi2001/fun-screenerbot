// Copyright (c) 2024-2026 ScreenerBot (screenerbot.io)
// SPDX-License-Identifier: BUSL-1.1

//! Trading configuration utilities.

use crate::config::{with_config, DcaLevel};
// FORK: F3 dynamic sizing + third slot
use crate::config::TraderConfig;
use crate::fork::balance::{dynamic_trade_size, slot_cap};
use crate::logger::{self, LogTag};

/// Get the maximum number of open positions allowed
pub fn get_max_open_positions() -> usize {
    // FORK: F3. Off = upstream: `max_open_positions`.
    if !is_dynamic_size_enabled() {
        return with_config(|cfg| cfg.trader.max_open_positions);
    }
    let balance = cached_sol_balance();
    let sol_usd = crate::apis::native_price::known_native_price();
    with_config(|cfg| max_open_positions_for(&cfg.trader, balance, sol_usd))
}

/// Get the default trade size in SOL
pub fn get_trade_size_native() -> f64 {
    // FORK: F3. Off = upstream: `trade_size_sol`.
    match dynamic_trade_size_now() {
        None => with_config(|cfg| cfg.trader.trade_size_sol),
        Some(Ok(size)) => size,
        Some(Err(err)) => {
            // Live entries are refused before this point (safety::check_position_limits);
            // this fallback only serves readers that need a number (UI, DCA default, paper).
            let fixed = with_config(|cfg| cfg.trader.trade_size_sol);
            logger::debug(
                LogTag::Trader,
                &format!("Dynamic trade size unavailable ({err}); using trade_size_sol {fixed}"),
            );
            fixed
        }
    }
}

// ==================== FORK: F3 DYNAMIC SIZING + THIRD SLOT ====================

/// Oldest live wallet snapshot whose SOL balance dynamic sizing trusts.
const DYNAMIC_BALANCE_MAX_AGE_SECS: i64 = 300;
/// Clock skew tolerated for a snapshot stamped in the future.
const DYNAMIC_BALANCE_MAX_FUTURE_SECS: i64 = 60;

/// Whether F3 dynamic sizing (and the third slot) is on.
pub fn is_dynamic_size_enabled() -> bool {
    with_config(|cfg| cfg.trader.dynamic_size_enabled)
}

/// `balance` when its snapshot at `snapshot_time` is fresh at `now`, else `None` (unknown).
pub fn fresh_balance(
    balance: f64,
    snapshot_time: chrono::DateTime<chrono::Utc>,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<f64> {
    let age = now.signed_duration_since(snapshot_time).num_seconds();
    (-DYNAMIC_BALANCE_MAX_FUTURE_SECS..=DYNAMIC_BALANCE_MAX_AGE_SECS)
        .contains(&age)
        .then_some(balance)
}

/// The SOL balance from the balance monitor's live snapshot (no RPC), or `None` while
/// there is no snapshot or it is stale.
pub fn cached_sol_balance() -> Option<f64> {
    let snapshot = crate::wallet::live_wallet_snapshot()?;
    fresh_balance(
        snapshot.native_balance,
        snapshot.snapshot_time,
        chrono::Utc::now(),
    )
}

/// Entry size for `trader`: `trade_size_sol` when dynamic sizing is off, else the dynamic
/// size of `balance` (Err on unknown/invalid balance or config).
pub fn trade_size_for(trader: &TraderConfig, balance: Option<f64>) -> anyhow::Result<f64> {
    if !trader.dynamic_size_enabled {
        return Ok(trader.trade_size_sol);
    }
    dynamic_trade_size(balance, trader)
}

/// Slot limit for `trader`: `max_open_positions` when dynamic sizing is off, else
/// [`slot_cap`] of `balance` at `sol_usd`.
pub fn max_open_positions_for(
    trader: &TraderConfig,
    balance: Option<f64>,
    sol_usd: Option<f64>,
) -> usize {
    if !trader.dynamic_size_enabled {
        return trader.max_open_positions;
    }
    slot_cap(balance, sol_usd, trader)
}

/// Permits the global position semaphore holds: `max_open_positions` when dynamic sizing
/// is off, else room for the third slot so it can open without a restart.
pub fn semaphore_capacity(trader: &TraderConfig) -> usize {
    if !trader.dynamic_size_enabled {
        return trader.max_open_positions;
    }
    trader
        .max_open_positions
        .max(trader.third_slot_max_positions)
}

/// [`semaphore_capacity`] of the live config.
pub fn get_semaphore_capacity() -> usize {
    with_config(|cfg| semaphore_capacity(&cfg.trader))
}

/// FORK: F3 (T10b). Whether `in_use` permits (counting the one just taken) fit the slot
/// cap: always true when dynamic sizing is off (upstream), else
/// `in_use <= max_open_positions_for(..)`.
pub fn permit_within_cap(
    trader: &TraderConfig,
    in_use: usize,
    balance: Option<f64>,
    sol_usd: Option<f64>,
) -> bool {
    if !trader.dynamic_size_enabled {
        return true;
    }
    in_use <= max_open_positions_for(trader, balance, sol_usd)
}

/// [`permit_within_cap`] with the cached balance and SOL/USD price of the live config.
pub fn permit_within_slot_cap(in_use: usize) -> bool {
    if !is_dynamic_size_enabled() {
        return true;
    }
    let balance = cached_sol_balance();
    let sol_usd = crate::apis::native_price::known_native_price();
    with_config(|cfg| permit_within_cap(&cfg.trader, in_use, balance, sol_usd))
}

/// `None` while dynamic sizing is off; otherwise the dynamic size of the cached balance.
pub fn dynamic_trade_size_now() -> Option<anyhow::Result<f64>> {
    if !is_dynamic_size_enabled() {
        return None;
    }
    let balance = cached_sol_balance();
    Some(with_config(|cfg| trade_size_for(&cfg.trader, balance)))
}

/// Get the entry check concurrency limit
pub fn get_entry_check_concurrency() -> usize {
    with_config(|cfg| cfg.trader.entry_check_concurrency)
}

/// Check if trader is enabled
pub fn is_trader_enabled() -> bool {
    with_config(|cfg| cfg.trader.enabled)
}

/// The DCA ladder: one step per automatic add, in the order the adds fire
pub fn get_dca_levels() -> Vec<DcaLevel> {
    with_config(|cfg| cfg.trader.dca_levels())
}

/// Get position close cooldown in minutes
pub fn get_position_close_cooldown_minutes() -> u64 {
    with_config(|cfg| cfg.trader.position_close_cooldown_minutes as u64)
}

/// Get sell (exit) concurrency limit
pub fn get_sell_concurrency() -> usize {
    with_config(|cfg| cfg.trader.sell_concurrency)
}

// ==================== ENTRY/EXIT MONITOR CONTROL ====================

/// Check if entry monitor is enabled
/// Combines master switch (trader.enabled) AND entry-specific flag
pub fn is_entry_monitor_enabled() -> bool {
    with_config(|cfg| cfg.trader.enabled && cfg.trader.entry_monitor_enabled)
}

/// Check if exit monitor is enabled
/// Combines master switch (trader.enabled) AND exit-specific flag
pub fn is_exit_monitor_enabled() -> bool {
    with_config(|cfg| cfg.trader.enabled && cfg.trader.exit_monitor_enabled)
}

/// Check if only entry monitor is enabled (for UI display)
pub fn is_entry_monitor_enabled_standalone() -> bool {
    with_config(|cfg| cfg.trader.entry_monitor_enabled)
}

/// Check if only exit monitor is enabled (for UI display)
pub fn is_exit_monitor_enabled_standalone() -> bool {
    with_config(|cfg| cfg.trader.exit_monitor_enabled)
}

// ==================== LOSS LIMIT CONFIGURATION ====================

/// Check if loss limit protection is enabled
pub fn is_loss_limit_enabled() -> bool {
    with_config(|cfg| cfg.trader.loss_limit_enabled)
}

/// Get loss limit threshold in SOL
pub fn get_loss_limit_native() -> f64 {
    with_config(|cfg| cfg.trader.loss_limit_sol)
}

/// Get loss limit period in hours
pub fn get_loss_limit_period_hours() -> u64 {
    with_config(|cfg| cfg.trader.loss_limit_period_hours)
}

/// Check if loss limit should auto-resume after period reset
pub fn is_loss_limit_auto_resume() -> bool {
    with_config(|cfg| cfg.trader.loss_limit_auto_resume)
}

// FORK: F3 dynamic sizing + third slot
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn on() -> TraderConfig {
        TraderConfig {
            dynamic_size_enabled: true,
            ..TraderConfig::default()
        }
    }

    #[test]
    fn dynamic_size_off_uses_trade_size_sol() {
        let off = TraderConfig::default();
        assert_eq!(trade_size_for(&off, Some(5.0)).unwrap(), off.trade_size_sol);
        assert_eq!(trade_size_for(&off, None).unwrap(), off.trade_size_sol);
    }

    #[test]
    fn dynamic_size_on_uses_balance() {
        let size = trade_size_for(&on(), Some(0.5)).unwrap();
        assert!((size - 0.05).abs() < 1e-12, "got {size}");
    }

    #[test]
    fn dynamic_size_on_unknown_balance_err() {
        assert!(trade_size_for(&on(), None).is_err());
        assert!(trade_size_for(&on(), Some(f64::NAN)).is_err());
    }

    #[test]
    fn capacity_check_allows_third_slot_above_40() {
        // 0.25 SOL * $160 = $40 exactly
        assert_eq!(max_open_positions_for(&on(), Some(0.25), Some(160.0)), 3);
        assert_eq!(max_open_positions_for(&on(), Some(0.25), Some(159.99)), 2);
        assert_eq!(max_open_positions_for(&on(), Some(0.25), None), 2);
    }

    #[test]
    fn slot_cap_off_ignores_balance() {
        let off = TraderConfig::default();
        assert_eq!(max_open_positions_for(&off, Some(100.0), Some(200.0)), 2);
    }

    #[test]
    fn semaphore_sized_for_third_slot() {
        assert_eq!(semaphore_capacity(&on()), 3);
        let mut bigger_base = on();
        bigger_base.max_open_positions = 5;
        assert_eq!(semaphore_capacity(&bigger_base), 5);
    }

    #[test]
    fn semaphore_off_is_upstream() {
        let mut off = TraderConfig::default();
        assert_eq!(semaphore_capacity(&off), 2);
        off.max_open_positions = 7;
        assert_eq!(semaphore_capacity(&off), 7);
    }

    #[test]
    fn permit_check_off_always_allows() {
        assert!(permit_within_cap(&TraderConfig::default(), 99, None, None));
    }

    #[test]
    fn permit_check_refuses_third_below_40() {
        assert!(!permit_within_cap(&on(), 3, Some(0.25), Some(159.99)));
        assert!(permit_within_cap(&on(), 2, Some(0.25), Some(159.99)));
    }

    #[test]
    fn permit_check_allows_third_at_40() {
        assert!(permit_within_cap(&on(), 3, Some(0.25), Some(160.0)));
        assert!(!permit_within_cap(&on(), 4, Some(0.25), Some(160.0)));
    }

    #[test]
    fn permit_check_unknown_price_is_two_slots() {
        assert!(!permit_within_cap(&on(), 3, Some(10.0), None));
        assert!(permit_within_cap(&on(), 2, Some(10.0), None));
    }

    #[test]
    fn fresh_balance_rejects_stale_and_future() {
        let now = Utc::now();
        assert_eq!(fresh_balance(1.0, now, now), Some(1.0));
        assert_eq!(
            fresh_balance(1.0, now - Duration::seconds(300), now),
            Some(1.0)
        );
        assert_eq!(fresh_balance(1.0, now - Duration::seconds(301), now), None);
        assert_eq!(
            fresh_balance(1.0, now + Duration::seconds(60), now),
            Some(1.0)
        );
        assert_eq!(fresh_balance(1.0, now + Duration::seconds(61), now), None);
    }
}
