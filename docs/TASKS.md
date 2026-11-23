# TASKS — fork build plan (from docs/SPEC.md "Two days, hour by hour")

Loop per CLAUDE.md: write `tasks/TXX.md` from the real files → `scripts/agy-task.sh TXX <model>` →
review `git diff` + named tests → max 2 `--fix` → commit `TXX: <summary>` and tick here.
Each task is one feature, ≤ 4 files, and must leave `cargo nextest run --all-targets` green.
All new config fields default to upstream behaviour. 🔒 = stop and ask before committing.

Models: **opus** = claude-opus-4-6-thinking · **sonnet** = claude-sonnet-4-6 · **flash** = gemini-3.8-flash-medium

## Day 1 — holder gate, breakout-retest, live on exit A

### F0 Baseline
- [ ] **T00 Baseline green run** · flash
  - Goal: `cargo fetch`, warm `cargo build`, record the baseline `CARGO_BUILD_JOBS=2 cargo nextest run --all-targets --no-tests=fail` (pass/fail counts, any pre-existing failures) in docs/HANDOFF.md. No code changes.
  - Files: `docs/HANDOFF.md`
  - Tests: full suite (SPEC test 1). Any failure here blocks F1 until understood.

### F1 HolderGate (first feature)
Gate before T03 spec: read Explore Mode rejection reasons — does `max_top_holder_pct = 5` reject nearly everything (i.e. RugCheck leaves pool vaults in `topHolders`)? Put the answer and a sample `topHolders` payload into tasks/T03.md.

- [x] **T01 HolderGate config fields** · flash
  - Goal: append to `RugCheckFilters`: `holder_gate_enabled: bool = false`, `max_top_10_holders_pct: f64 = 20.0` (used only when the gate is on). Gate on ⇒ pool/LP/AMM holders excluded, top-1/3/10 computed on ordinary holders, empty `top_holders` rejects.
  - Files: `src/config/schemas/filtering.rs`, `locales/en/config.ftl`
  - Tests: `metadata_catalog_tests`, the RugCheck defaults test in `filtering.rs` (extend it to assert the two new defaults).

- [x] **T02 HolderGate rejection reasons** · sonnet
  - Goal: add `RugcheckTop10HoldersTooHigh` and `RugcheckTopHoldersMissing` to `FilterRejectionReason`, mapped to source RugCheck, with English labels.
  - Files: `src/filtering/sources/rejection.rs`, `src/filtering/sources/mod.rs`, `locales/en/filtering.ftl`
  - Tests: `cargo nextest run -E 'test(/rejection|filtering_sources/)'`

- [ ] **T03 Pool-holder classifier (pure)** · opus
  - Goal: new `src/fork/holder_gate.rs`: `ordinary_holders(&[TokenHolder], &PoolContext) -> Vec<&TokenHolder>` excluding holders whose `owner`/`address` is a known AMM/LP authority or a pool vault of the token; `top_n_sum(.., n)` with the NaN rules of `three_largest_pcts`. Creates the `src/fork` module.
  - Files: `src/fork/holder_gate.rs`, `src/fork/mod.rs`, `src/lib.rs` (one `// FORK:` mod line)
  - Tests (inline `#[cfg(test)]`): `pool_vault_excluded`, `amm_authority_owner_excluded`, `ordinary_holder_kept`, `nan_pct_dropped`, `top10_sum_with_fewer_than_10_holders`.

- [ ] **T04 Wire HolderGate into RugCheck stage** · opus
  - Goal: in `check_holder_distribution`, when `holder_gate_enabled`: empty list → `RugcheckTopHoldersMissing`; top-1/top-3 on ordinary holders; top-10 sum `>` `max_top_10_holders_pct` → `RugcheckTop10HoldersTooHigh`. Gate off ⇒ byte-for-byte upstream behaviour.
  - Files: `src/filtering/sources/rugcheck.rs`, `tests/filtering_sources.rs`
  - Tests: `holder_gate_excludes_pool_account`, `holder_gate_top10_exactly_20_passes`, `holder_gate_top10_above_20_rejects`, `holder_gate_empty_list_rejects`, `holder_gate_off_matches_upstream` (SPEC test 3, F1 part).

### F2 BreakoutRetest
- [ ] **T05 BreakoutRetest condition** · sonnet
  - Goal: new ENTRY condition per SPEC §"The entry rule tree" + F2 prompt: params `timeframe 5m, resistance_lookback 24, retest_window 1-12 (12), retest_tolerance_pct 1.5, max_candle_change_pct 25, require_higher_low true`; closed candles only; pseudocode and Err-vs-false table go in tasks/T05.md.
  - Files: `src/strategies/conditions/breakout_retest.rs`, `src/strategies/conditions/mod.rs`, `locales/en/strategies.ftl`, `tools/tests/fixtures/dashboard/trader/strategies_condition_schemas.json`
  - Tests: `cargo nextest run -E 'test(/catalog|condition_schema/)'`

- [ ] **T06 BreakoutRetest boundary tests** · sonnet
  - Goal: 10 boundary tests; fix the condition only if one fails.
  - Files: `tests/strategies_conditions.rs`, `src/strategies/conditions/breakout_retest.rs`
  - Tests: `breakout_retest_tolerance_exactly_1_5_pct`, `..._candle_change_exactly_25_pct`, `..._23_candles_is_err`, `..._24_candles_ok`, `..._forming_candle_excluded`, `..._stale_series_is_err`, `..._no_retest_touch_false`, `..._close_below_level_false`, `..._higher_low_required`, `..._retest_window_expired_false` (SPEC test 2).

### Build + deploy prep
- [ ] **T07 Trim CI to the Linux headless build** · flash
  - Goal: reduce `build.yml` to the single Linux headless job on `ubuntu-22.04` (glibc < 2.34), triggered on `v*` tags and `workflow_dispatch`, uploading the binary as an artifact.
  - Files: `.github/workflows/build.yml`
  - Tests: `actionlint` if installed, else YAML parse; no Rust change.

Manual gate (you): merge F1+F2 to `fork/main`, CI build, deploy, exit-A TOML + 48h EXIT strategy, entry tree TokenAge + BreakoutRetest, SPEC tests 4-6, then `trader.enabled = true`.

### F3 Dynamic sizing + third slot (money path)
- [ ] **T08 Sizing config fields** · flash
  - Goal: append to `TraderConfig`: `dynamic_size_enabled = false`, `dynamic_size_pct = 10.0`, `dynamic_size_floor_sol = 0.001`, `dynamic_size_ceiling_sol = 10.0`, `third_slot_threshold_usd = 40.0`, `third_slot_max_positions = 3`.
  - Files: `src/config/schemas/trader.rs`, `locales/en/config.ftl`
  - Tests: `metadata_catalog_tests`, trader defaults test.

- [ ] **T09 Sizing + slot math (pure)** · opus
  - Goal: `src/fork/balance.rs`: `dynamic_trade_size(balance_sol, &cfg) -> Result<f64>` (10%, clamped floor..ceiling, Err on NaN/≤0/unknown balance) and `slot_cap(balance_sol, sol_usd: Option<f64>, &cfg) -> usize` (≥ $40 → 3, missing/NaN price → base cap).
  - Files: `src/fork/balance.rs`, `src/fork/mod.rs`
  - Tests (inline): `size_floor_applies`, `size_ceiling_applies`, `size_nan_balance_err`, `slot_flip_exactly_40_usd`, `slot_below_40_usd_is_2`, `missing_price_gives_2_slots`.

- [ ] **T10 Wire sizing into trader + semaphore** 🔒 if the balance cache lives under `src/wallets` · opus
  - Goal: `get_trade_size_native()` / `get_max_open_positions()` use T09 when enabled, reading a cached SOL balance (no new RPC); capacity check uses `slot_cap`; `GLOBAL_POSITION_SEMAPHORE` sized at `max(max_open_positions, third_slot_max_positions)` when enabled so 3 slots need no restart.
  - Files: `src/trader/config.rs`, `src/trader/safety/limits.rs`, `src/positions/state.rs`, `tests/trading.rs`
  - Tests: `dynamic_size_off_uses_trade_size_sol`, `dynamic_size_on_uses_balance`, `capacity_check_allows_third_slot_above_40`, `semaphore_sized_for_third_slot` (SPEC test 3, F3 part). Then fresh-session review (SPEC test 7).

## Day 2 — score, runner-only trail, option to switch to exit B

### F4 CandidateScore + Telegram
- [ ] **T11 Score function (pure)** · opus
  - Goal: `src/fork/score.rs`: `score(&ScoreInputs) -> ScoreBreakdown` (chart quality from retest geometry, buyers from buy/sell counts + ratio, liquidity from `liquidity_sol`, minus capped AI penalty, clamped 0..=100). Missing input → Err.
  - Files: `src/fork/score.rs`, `src/fork/mod.rs`
  - Tests (inline): `score_clamped_0_100`, `ai_penalty_only_subtracts`, `no_ai_verdict_penalty_zero`, `missing_input_err`, `nan_inputs_err`.

- [ ] **T12 CandidateScore condition** · opus
  - Goal: ENTRY condition `CandidateScore {min_score}` calling T11; true iff score ≥ min_score.
  - Files: `src/strategies/conditions/candidate_score.rs`, `src/strategies/conditions/mod.rs`, `locales/en/strategies.ftl`, `tools/tests/fixtures/dashboard/trader/strategies_condition_schemas.json`
  - Tests: `cargo nextest run -E 'test(/catalog|condition_schema/)'`

- [ ] **T13 CandidateScore tests** · sonnet
  - Files: `tests/strategies_conditions.rs`
  - Tests: `candidate_score_exactly_min_passes`, `candidate_score_below_min_false`, `candidate_score_missing_data_err`, `candidate_score_ai_cannot_raise`.

- [ ] **T14 Score line on "position opened"** · sonnet
  - Goal: `msg_position_opened` gets an optional score breakdown and prints one `Score NN/100 (chart a · buyers b · liq c · ai −d)` line; absent → no line.
  - Files: `src/telegram/messages.rs`, `src/telegram/notifier.rs` (+ the trader call site if it must pass the score; name it in tasks/T14.md, ≤ 4 total)
  - Tests: `position_opened_with_score_line`, `position_opened_without_score_unchanged`.

### F5 Runner-only trail (money path)
- [ ] **T15 Runner-only trail field** · flash
  - Goal: append `trailing_stop_runner_only: bool = false` to `TraderConfig`.
  - Files: `src/config/schemas/trader.rs`, `locales/en/config.ftl`
  - Tests: `metadata_catalog_tests`, trader defaults test.

- [ ] **T16 Runner-only trail logic** · opus
  - Goal: when on, the trail is skipped until every ladder step has filled, then arms at the last rung's fill price with peak reset; stop loss keeps precedence; trail and TP can never sell the same tokens in one tick.
  - Files: `src/trader/policy.rs`, `src/trader/evaluators/exit_rules.rs`, `tests/trader_exits.rs`
  - Tests: `runner_trail_silent_before_last_rung`, `runner_trail_arms_at_last_rung_with_peak_reset`, `runner_trail_stop_loss_still_first`, `runner_trail_no_double_sell_same_tick`, `runner_trail_off_matches_upstream` (SPEC test 3, F5 part). Then fresh-session review of T10 + T16 (SPEC test 7).

Manual gate (you): CI build, deploy between trades, `dynamic_size_enabled = true`, add `CandidateScore` to the entry tree, choose exit A or B (`[100,400]`/`[50,25]` + `trailing_stop_runner_only = true`).

### Wrap-up
- [ ] **T17 Rebase runbook** · flash
  - Goal: HANDOFF.md section: rebasing `fork/main` onto a new upstream `v*` tag (`git rerere`, `// FORK:` grep, full nextest, fixture/ftl re-check), plus deploy/rollback steps from SPEC.
  - Files: `docs/HANDOFF.md`
  - Tests: none (docs).

## Later (not in the 2 days)
Per-entry-strategy exit policy in `resolve_exit_policy()` (A and B side by side) · Solana Tracker bundler/insider filter · passphrase layer on the wallet key · paper-mode shadow clone of the entry strategy (dashboard, no code).
