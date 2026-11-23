# AGENTS.md — personal fork of ScreenerBot (BSL 1.1)
Rust 1.98.1 (rust-toolchain.toml), single crate, headless trading bot. This fork adds small,
default-off features for one user. Spec: docs/SPEC.md · Tasks: docs/TASKS.md · Status: docs/HANDOFF.md
Code is written by agy (Antigravity CLI), one task file at a time (tasks/TXX.md); Claude specifies and reviews.

## Never
- Run the screenerbot binary, `mcp serve`, or any #[ignore] test. Never set SB_TEST_MAINNET_SWAP / SB_TEST_WALLET.
- Read, create or print keys, wallets.db, any data-dir config.toml, or .env files.
- Edit src/chains/solana/swaps/revenue.rs, src/secure_storage/**, src/account/**,
  src/chains/solana/swaps/presend.rs or cost_guard/** unless the task names the file.
- Add a crate, network call or dependency without asking. Use plain `cargo test` across modules.

## Commands
cargo check --lib
cargo nextest run -E 'test(/<pattern>/)'
CARGO_BUILD_JOBS=2 cargo nextest run --all-targets --no-tests=fail   # before handing back
cargo fmt

## Fork rules
- Touch only the files the task file lists (max 4). If you need another file, stop and say why.
- New logic in new files: src/strategies/conditions/<name>.rs, src/fork/<name>.rs.
- Upstream files: only registration lines, fields appended at struct end, accessor bodies. Mark each `// FORK:`.
- Every new config field defaults to upstream behaviour (feature off).
- Every new file: BUSL header + `//!` description (tools/header_check.mjs).

## Checklists
New condition: impl ConditionEvaluator; `mod` + `pub use` + `registry.register` in conditions/mod.rs;
  strategies-condition-<kebab>-* keys in locales/en/strategies.ftl; entry in
  tools/tests/fixtures/dashboard/trader/strategies_condition_schemas.json; tests in tests/strategies_conditions.rs.
New config field: config_struct! line + field_metadata!; config-<section>-<field> (+ .hint) in locales/en/config.ftl.
New test file: `mod` line in tests/integration.rs. Other locales: leave them; English is the
  fallback source (locales/registry.toml), unless the task says otherwise.

## Code rules
Missing or stale data returns Err, never false or 0. NaN/zero guard on every float compare.
No unwrap() on money paths. Reach Solana code only via crate::chains::solana (tests/architecture_boundaries.rs).
Closed candles only; state explicitly whether the forming candle is excluded.

## Code Review Rules
Findings only, worst first, file:line, each with a failing scenario. Flag: edits to swaps/presend/cost_guard/
secure_storage/revenue; any default that changes live behaviour; candle-window off-by-one; Err vs false;
double-sell or oversell paths; unguarded float compares; edits outside the task's files.
