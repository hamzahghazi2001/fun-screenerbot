// Copyright (c) 2024-2026 ScreenerBot (screenerbot.io)
// SPDX-License-Identifier: BUSL-1.1
// FORK: F2

//! Breakout retest condition — detects price breaking above resistance and holding the retest level.
//!
//! Candle counts are stored candles, not wall-clock buckets (no-trade buckets are never stored),
//! the same as every other candle condition. Closed candles only; the forming candle is excluded explicitly.

use crate::ohlcvs::{Candle, Timeframe};
use crate::strategies::conditions::{
    get_candles_for_timeframe, get_param_bool, get_param_f64_in_range, get_param_string_optional,
    validate_timeframe_param, ConditionEvaluator,
};
use crate::strategies::types::{Condition, ConditionData, EvaluationContext, StrategyType};
use crate::strategies::{Error, Result};
use async_trait::async_trait;
use serde_json::json;

/// Validated parameters of a breakout retest condition.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub timeframe: Option<String>,
    pub resistance_lookback: usize,
    pub retest_window: usize,
    pub retest_tolerance_pct: f64,
    pub max_candle_change_pct: f64,
    pub require_higher_low: bool,
}

fn read_params(condition: &Condition) -> Result<Params> {
    validate_timeframe_param(condition)?;
    let timeframe = get_param_string_optional(condition, "timeframe");

    let lookback_f = get_param_f64_in_range(
        condition,
        "resistance_lookback",
        "resistance lookback",
        2.0,
        50.0,
    )?;
    if lookback_f.fract() != 0.0 {
        return Err(Error::InvalidConditionValue {
            field: "resistance lookback",
            value: lookback_f.to_string(),
        });
    }

    let window_f = get_param_f64_in_range(condition, "retest_window", "retest window", 1.0, 12.0)?;
    if window_f.fract() != 0.0 {
        return Err(Error::InvalidConditionValue {
            field: "retest window",
            value: window_f.to_string(),
        });
    }

    let retest_tolerance_pct = get_param_f64_in_range(
        condition,
        "retest_tolerance_pct",
        "retest tolerance",
        0.1,
        10.0,
    )?;

    let max_candle_change_pct = get_param_f64_in_range(
        condition,
        "max_candle_change_pct",
        "max candle change",
        1.0,
        100.0,
    )?;

    let require_higher_low = get_param_bool(condition, "require_higher_low")?;

    Ok(Params {
        timeframe,
        resistance_lookback: lookback_f as usize,
        retest_window: window_f as usize,
        retest_tolerance_pct,
        max_candle_change_pct,
        require_higher_low,
    })
}

fn check_candle_count(available: usize, params: &Params) -> Result<usize> {
    let required = params.resistance_lookback + params.retest_window + 1;
    if available < required {
        return Err(Error::InsufficientCandles {
            indicator: "breakout retest",
            available,
            required,
        });
    }
    Ok(required)
}

fn closed_candles(candles: &[Candle], bucket: i64, now: i64) -> Vec<Candle> {
    let mut closed: Vec<Candle> = candles
        .iter()
        .filter(|c| c.timestamp + bucket <= now)
        .cloned()
        .collect();
    closed.sort_by_key(|c| c.timestamp);
    closed
}

fn evaluate_series(s: &[Candle], p: &Params) -> Result<bool> {
    let l = p.resistance_lookback;
    let w = p.retest_window;
    let need = l + w + 1;
    if s.len() < need {
        return Err(Error::InsufficientCandles {
            indicator: "breakout retest",
            available: s.len(),
            required: need,
        });
    }
    let s = &s[s.len() - need..];

    for c in s {
        if !c.open.is_finite()
            || c.open <= 0.0
            || !c.high.is_finite()
            || c.high <= 0.0
            || !c.low.is_finite()
            || c.low <= 0.0
            || !c.close.is_finite()
            || c.close <= 0.0
        {
            return Err(Error::MissingContextData { data: "OHLCV data" });
        }
    }

    let last = need - 1;
    let mut breakout = None;

    for i in (l..=last).rev() {
        let level = s[i - l..i]
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        if s[i].close > level {
            breakout = Some((i, level));
            break;
        }
    }

    let (breakout_idx, level) = match breakout {
        Some(b) => b,
        None => return Ok(false),
    };

    if breakout_idx == last {
        return Ok(false);
    }

    let retest_candles = &s[breakout_idx + 1..=last];
    let tol = p.retest_tolerance_pct / 100.0;
    let band_low = level * (1.0 - tol);
    let band_high = level * (1.0 + tol);

    for c in &s[breakout_idx..=last] {
        let body_move = ((c.close - c.open).abs() / c.open) * 100.0;
        let range_move = ((c.high - c.low) / c.low) * 100.0;
        if body_move > p.max_candle_change_pct || range_move > p.max_candle_change_pct {
            return Ok(false);
        }
    }

    for c in retest_candles {
        if c.close < band_low {
            return Ok(false);
        }
    }

    let touched = retest_candles
        .iter()
        .any(|c| band_low <= c.low && c.low <= band_high);
    if !touched {
        return Ok(false);
    }

    if p.require_higher_low {
        let min_retest_low = retest_candles
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);
        let min_base_low = s[breakout_idx - l..=breakout_idx]
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);
        if min_retest_low <= min_base_low {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Breakout and retest entry condition.
pub struct BreakoutRetestCondition;

#[async_trait]
impl ConditionEvaluator for BreakoutRetestCondition {
    fn condition_type(&self) -> &'static str {
        "BreakoutRetest"
    }

    async fn evaluate(&self, condition: &Condition, context: &EvaluationContext) -> Result<bool> {
        let params = read_params(condition)?;
        let candles = get_candles_for_timeframe(context, params.timeframe.as_deref())?;
        let tf = params
            .timeframe
            .as_deref()
            .unwrap_or(&context.strategy_timeframe);
        let bucket = Timeframe::from_str(tf)
            .ok_or_else(|| Error::InvalidConditionValue {
                field: "timeframe",
                value: tf.to_owned(),
            })?
            .to_seconds();
        let now = context.evaluated_at.timestamp();
        let closed = closed_candles(&candles, bucket, now);
        let need = check_candle_count(closed.len(), &params)?;
        let s = &closed[closed.len() - need..];
        evaluate_series(s, &params)
    }

    fn validate(&self, condition: &Condition) -> Result<()> {
        read_params(condition).map(|_| ())
    }

    fn applies_to(&self) -> &'static [StrategyType] {
        &[StrategyType::Entry]
    }

    fn reads(&self) -> ConditionData {
        ConditionData::Candles
    }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "BreakoutRetest",
            "category": "Price Analysis",
            "icon": "icon-magnet",
            "origin": "strategy",
            "parameters": {
                "timeframe": {
                    "type": "enum",
                    "default": "5m",
                    "optional": true,
                    "options": [
                        { "value": "1m" },
                        { "value": "5m" },
                        { "value": "15m" },
                        { "value": "1h" },
                        { "value": "4h" },
                        { "value": "12h" },
                        { "value": "1d" }
                    ]
                },
                "resistance_lookback": {
                    "type": "number",
                    "unit": "candles",
                    "default": 24,
                    "min": 2,
                    "max": 50,
                    "step": 1
                },
                "retest_window": {
                    "type": "number",
                    "unit": "candles",
                    "default": 12,
                    "min": 1,
                    "max": 12,
                    "step": 1
                },
                "retest_tolerance_pct": {
                    "type": "percent",
                    "default": 1.5,
                    "min": 0.1,
                    "max": 10,
                    "step": 0.1
                },
                "max_candle_change_pct": {
                    "type": "percent",
                    "default": 25,
                    "min": 1,
                    "max": 100,
                    "step": 1
                },
                "require_higher_low": {
                    "type": "boolean",
                    "default": true
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn candle(ts: i64, o: f64, h: f64, l: f64, c: f64) -> Candle {
        Candle::new(ts, o, h, l, c, 1000.0)
    }

    fn default_params() -> Params {
        Params {
            timeframe: Some("5m".to_owned()),
            resistance_lookback: 24,
            retest_window: 12,
            retest_tolerance_pct: 1.5,
            max_candle_change_pct: 25.0,
            require_higher_low: true,
        }
    }

    fn base_series() -> Vec<Candle> {
        let mut candles = Vec::with_capacity(37);
        for i in 0..24 {
            candles.push(candle(i * 300, 99.0, 100.0, 98.0, 99.0));
        }
        candles.push(candle(24 * 300, 100.0, 104.0, 100.0, 103.0));
        for i in 25..=36 {
            candles.push(candle(i * 300, 101.0, 102.0, 100.5, 101.0));
        }
        candles
    }

    fn test_condition(params: &[(&str, serde_json::Value)]) -> Condition {
        let mut map = HashMap::new();
        for (name, val) in params {
            map.insert(
                (*name).to_owned(),
                crate::strategies::types::Parameter {
                    value: val.clone(),
                    default: val.clone(),
                    constraints: None,
                },
            );
        }
        Condition {
            condition_type: "BreakoutRetest".to_owned(),
            parameters: map,
        }
    }

    #[test]
    fn breakout_retest_clean_retest_true() {
        let s = base_series();
        let p = default_params();
        assert!(evaluate_series(&s, &p).unwrap());
    }

    #[test]
    fn breakout_retest_short_series_err() {
        let p = default_params();
        let err = check_candle_count(36, &p).unwrap_err();
        assert!(matches!(
            err,
            Error::InsufficientCandles {
                indicator: "breakout retest",
                available: 36,
                required: 37,
            }
        ));
    }

    #[test]
    fn breakout_retest_non_finite_candle_err() {
        let mut s = base_series();
        s[10].close = f64::NAN;
        let p = default_params();
        let err = evaluate_series(&s, &p).unwrap_err();
        assert!(matches!(
            err,
            Error::MissingContextData { data: "OHLCV data" }
        ));
    }

    #[test]
    fn breakout_retest_breakout_on_last_candle_false() {
        let mut s = Vec::with_capacity(37);
        for i in 0..36 {
            s.push(candle(i * 300, 99.0, 100.0, 98.0, 99.0));
        }
        s.push(candle(36 * 300, 100.0, 104.0, 100.0, 103.0));
        let p = default_params();
        assert!(!evaluate_series(&s, &p).unwrap());
    }

    #[test]
    fn breakout_retest_vertical_candle_false() {
        let mut s = base_series();
        s[30].high = 130.0;
        let p = default_params();
        assert!(!evaluate_series(&s, &p).unwrap());
    }

    #[test]
    fn breakout_retest_no_touch_false() {
        let mut s = base_series();
        for c in &mut s[25..=36] {
            c.low = 101.6;
        }
        let p = default_params();
        assert!(!evaluate_series(&s, &p).unwrap());
    }

    #[test]
    fn breakout_retest_forming_candle_dropped() {
        let t = 10_000;
        let c1 = candle(t - 600, 10.0, 10.0, 10.0, 10.0);
        let c2 = candle(t - 300, 10.0, 10.0, 10.0, 10.0);
        let c3 = candle(t - 100, 10.0, 10.0, 10.0, 10.0);
        let unsorted = vec![c3, c1.clone(), c2.clone()];
        let result = closed_candles(&unsorted, 300, t);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].timestamp, t - 600);
        assert_eq!(result[1].timestamp, t - 300);
    }

    #[test]
    fn breakout_retest_params_out_of_range_err() {
        let valid_base = [
            ("timeframe", json!("5m")),
            ("resistance_lookback", json!(24)),
            ("retest_window", json!(12)),
            ("retest_tolerance_pct", json!(1.5)),
            ("max_candle_change_pct", json!(25)),
            ("require_higher_low", json!(true)),
        ];

        // resistance_lookback 51 (max 50)
        let mut p = valid_base.clone();
        p[1] = ("resistance_lookback", json!(51));
        assert!(matches!(
            read_params(&test_condition(&p)).unwrap_err(),
            Error::InvalidConditionValue {
                field: "resistance lookback",
                ..
            }
        ));

        // retest_window 0 (min 1)
        let mut p = valid_base.clone();
        p[2] = ("retest_window", json!(0));
        assert!(matches!(
            read_params(&test_condition(&p)).unwrap_err(),
            Error::InvalidConditionValue {
                field: "retest window",
                ..
            }
        ));

        // retest_window 2.5 (non-whole)
        let mut p = valid_base.clone();
        p[2] = ("retest_window", json!(2.5));
        assert!(matches!(
            read_params(&test_condition(&p)).unwrap_err(),
            Error::InvalidConditionValue {
                field: "retest window",
                ..
            }
        ));

        // retest_tolerance_pct 10.5 (max 10.0)
        let mut p = valid_base.clone();
        p[3] = ("retest_tolerance_pct", json!(10.5));
        assert!(matches!(
            read_params(&test_condition(&p)).unwrap_err(),
            Error::InvalidConditionValue {
                field: "retest tolerance",
                ..
            }
        ));

        // missing require_higher_low
        let without_rhl = &valid_base[..5];
        let err = read_params(&test_condition(without_rhl)).unwrap_err();
        assert!(matches!(
            err,
            Error::MissingConditionParameter { field } if field == "require_higher_low"
        ));
    }

    #[test]
    fn breakout_retest_close_exactly_at_resistance_not_breakout() {
        let p = default_params();
        let mut s1 = Vec::with_capacity(37);
        for i in 0..24 {
            s1.push(candle(i * 300, 99.0, 100.0, 98.0, 99.0));
        }
        s1.push(candle(24 * 300, 99.0, 100.0, 98.0, 100.0));
        for i in 25..=36 {
            s1.push(candle(i * 300, 99.0, 100.0, 98.0, 99.0));
        }
        assert!(!evaluate_series(&s1, &p).unwrap());

        let mut s2 = Vec::with_capacity(37);
        for i in 0..24 {
            s2.push(candle(i * 300, 99.0, 100.0, 98.0, 99.0));
        }
        s2.push(candle(24 * 300, 99.0, 100.01, 99.0, 100.01));
        for i in 25..=36 {
            s2.push(candle(i * 300, 100.0, 100.01, 99.9, 100.0));
        }
        assert!(evaluate_series(&s2, &p).unwrap());
    }

    #[test]
    fn breakout_retest_forming_candle_excluded() {
        let mut all = base_series();
        let bucket = 300;
        let now = 37 * bucket;
        all.push(candle(37 * bucket, 101.0, 102.0, 49.0, 50.0));
        let closed = closed_candles(&all, bucket, now);
        assert_eq!(closed.len(), 37);
        let p = default_params();
        assert!(evaluate_series(&closed, &p).unwrap());
    }

    #[test]
    fn breakout_retest_requires_higher_low() {
        let mut s = base_series();
        s[30] = candle(30 * 300, 101.0, 102.0, 97.9, 101.0);

        let mut p = default_params();
        p.require_higher_low = true;
        assert!(!evaluate_series(&s, &p).unwrap());

        p.require_higher_low = false;
        assert!(evaluate_series(&s, &p).unwrap());

        s[30] = candle(30 * 300, 101.0, 102.0, 98.0, 101.0);
        p.require_higher_low = true;
        assert!(!evaluate_series(&s, &p).unwrap());
    }

    #[test]
    fn breakout_retest_close_below_band_resets() {
        let p = default_params();

        // Candle 28 close = 98.4 (below band_low 98.5) -> false
        let mut s = base_series();
        s[28].close = 98.4;
        assert!(!evaluate_series(&s, &p).unwrap());

        // Fresh breakout at candle 31 (close 105 > level 104)
        s[31] = candle(31 * 300, 101.0, 106.0, 101.0, 105.0);
        for (offset, c) in s[32..=36].iter_mut().enumerate() {
            let i = (32 + offset) as i64;
            *c = candle(i * 300, 104.5, 105.0, 104.0, 104.5);
        }
        assert!(evaluate_series(&s, &p).unwrap());

        // Candle 28 close = 98.5 (exactly on band_low) on base series -> true
        let mut s2 = base_series();
        s2[28].close = 98.5;
        assert!(evaluate_series(&s2, &p).unwrap());
    }
}
