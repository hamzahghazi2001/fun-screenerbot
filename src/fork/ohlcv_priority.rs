// Copyright (c) 2024-2026 ScreenerBot (screenerbot.io)
// SPDX-License-Identifier: BUSL-1.1

//! OHLCV passed-first ordering and priority helpers.

use std::collections::HashSet;

use crate::ohlcvs::{Priority, Timeframe};

static NATIVE_5M: [Timeframe; 1] = [Timeframe::Minute5];

/// Mints to fetch this cycle. `enabled == false`: `mints` unchanged (same order, all kept).
/// `enabled == true`: mints in `held` (sorted), then mints in `passed` and not in `held`
/// (sorted); every other mint is dropped.
pub fn fetch_order(
    mints: Vec<String>,
    held: &HashSet<String>,
    passed: &HashSet<String>,
    enabled: bool,
) -> Vec<String> {
    if !enabled {
        return mints;
    }

    let mut held_mints = Vec::new();
    let mut passed_mints = Vec::new();

    for mint in mints {
        if held.contains(&mint) {
            held_mints.push(mint);
        } else if passed.contains(&mint) {
            passed_mints.push(mint);
        }
    }

    held_mints.sort();
    passed_mints.sort();
    held_mints.extend(passed_mints);
    held_mints
}

/// Whether `mint` may be candle-fetched at all. `enabled == false`: always true.
/// `enabled == true`: true only for a held or passed mint.
pub fn may_fetch(
    mint: &str,
    held: &HashSet<String>,
    passed: &HashSet<String>,
    enabled: bool,
) -> bool {
    if !enabled {
        true
    } else {
        held.contains(mint) || passed.contains(mint)
    }
}

/// Priority a fetch uses: `stored.max(Priority::High)` when `enabled && passed`, else `stored`.
pub fn effective_priority(stored: Priority, passed: bool, enabled: bool) -> Priority {
    if enabled && passed {
        stored.max(Priority::High)
    } else {
        stored
    }
}

/// Native aggregated timeframes to refresh: `Some(&[Timeframe::Minute5])` when
/// `enabled && passed && stored != Priority::Critical`, else `None` (= every timeframe, upstream).
pub fn native_timeframes(
    stored: Priority,
    passed: bool,
    enabled: bool,
) -> Option<&'static [Timeframe]> {
    if enabled && passed && stored != Priority::Critical {
        Some(&NATIVE_5M)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ohlcv_passed_first_off_keeps_order_and_all_tokens() {
        let mints = vec![
            "p2".to_string(),
            "x".to_string(),
            "h1".to_string(),
            "p1".to_string(),
        ];
        let held = HashSet::from(["h1".to_string()]);
        let passed = HashSet::from(["p1".to_string(), "p2".to_string()]);
        let result = fetch_order(mints.clone(), &held, &passed, false);
        assert_eq!(result, mints);
    }

    #[test]
    fn ohlcv_fetch_order_held_then_passed() {
        let mints = vec![
            "p2".to_string(),
            "x".to_string(),
            "h1".to_string(),
            "p1".to_string(),
        ];
        let held = HashSet::from(["h1".to_string()]);
        let passed = HashSet::from(["p1".to_string(), "p2".to_string(), "h1".to_string()]);
        let result = fetch_order(mints, &held, &passed, true);
        assert_eq!(result, vec!["h1", "p1", "p2"]);
    }

    #[test]
    fn ohlcv_fetch_order_skips_unpassed_unheld() {
        let mints = vec!["unpassed_unheld".to_string(), "held1".to_string()];
        let held = HashSet::from(["held1".to_string()]);
        let passed = HashSet::new();
        let result = fetch_order(mints, &held, &passed, true);
        assert_eq!(result, vec!["held1"]);
        assert!(!may_fetch("unpassed_unheld", &held, &passed, true));
        assert!(may_fetch("unpassed_unheld", &held, &passed, false));
    }

    #[test]
    fn ohlcv_fetch_order_held_not_passed_kept() {
        let mints = vec!["p1".to_string(), "h_unpassed".to_string()];
        let held = HashSet::from(["h_unpassed".to_string()]);
        let passed = HashSet::from(["p1".to_string()]);
        let result = fetch_order(mints, &held, &passed, true);
        assert_eq!(result, vec!["h_unpassed", "p1"]);
    }

    #[test]
    fn ohlcv_passed_token_gets_high_priority() {
        assert_eq!(
            effective_priority(Priority::Low, true, true),
            Priority::High
        );
        assert_eq!(
            effective_priority(Priority::Medium, true, true),
            Priority::High
        );
        assert_eq!(
            effective_priority(Priority::High, true, true),
            Priority::High
        );
    }

    #[test]
    fn ohlcv_critical_stays_critical() {
        assert_eq!(
            effective_priority(Priority::Critical, true, true),
            Priority::Critical
        );
    }

    #[test]
    fn ohlcv_off_priority_unchanged() {
        assert_eq!(
            effective_priority(Priority::Low, true, false),
            Priority::Low
        );
        assert_eq!(
            effective_priority(Priority::Low, false, true),
            Priority::Low
        );
        assert_eq!(
            effective_priority(Priority::Medium, false, true),
            Priority::Medium
        );
    }

    #[test]
    fn ohlcv_passed_native_timeframes_only_5m() {
        assert_eq!(
            native_timeframes(Priority::Low, true, true),
            Some(&[Timeframe::Minute5] as &[Timeframe])
        );
        assert_eq!(
            native_timeframes(Priority::High, true, true),
            Some(&[Timeframe::Minute5] as &[Timeframe])
        );
    }

    #[test]
    fn ohlcv_held_and_off_fetch_all_timeframes() {
        assert_eq!(native_timeframes(Priority::Critical, true, true), None);
        assert_eq!(native_timeframes(Priority::Low, true, false), None);
        assert_eq!(native_timeframes(Priority::Low, false, true), None);
        assert_eq!(native_timeframes(Priority::Critical, false, false), None);
    }
}
