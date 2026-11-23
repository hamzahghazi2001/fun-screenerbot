# Run ScreenerBot live, then patch five gaps

Use ScreenerBot as the engine and fork it rather than switch. No open-source project covers more of your strategy. Hummingbot plus Gateway is the only serious runner-up, and choosing it would mean building the screener, the RugCheck gate, the retest entry and the multi-step exits yourself. About two-thirds of your rules map straight onto ScreenerBot's `config.toml` keys and strategy rule trees. **Exit A can run exactly as written today** if you add one 48h exit strategy, so Day 1 goes live on exit A. Five rules need small code changes in a private fork: a holder gate that leaves out pool accounts and checks the top 10, a breakout-and-retest condition, 10% dynamic sizing with the third slot, the 0-100 score with a Telegram line, and a trail that applies only to exit B's last 25%. Each change is a new file plus a few registration lines. Claude writes the specs and reviews every diff, and Codex CLI writes the Rust on your MacBook. The bot runs headless under systemd on a 2 vCPU / 4 GB VPS. Its dashboard stays on loopback behind Tailscale with a password and TOTP, and auto-update is off so upstream can't overwrite your fork. Services cost about $0 if you spread RPC across four free tiers, which means correcting three wrong rate-limit defaults. Tooling is $40 for the month. The cost that grows with you is ScreenerBot's **hardcoded 0.5% fee on every swap**, which works out to about 0.75% of stake on a −50% stop round trip and 0.5% of every sale. The wallet key is encrypted with a machine-derived key and no passphrase, so the bot gets a burner wallet. The build takes two days and only the money paths get tested.

## ScreenerBot is the strongest base, and nothing open-source comes close

Only three relevant projects had commits in the two weeks before 23 Nov 2026: ScreenerBot itself (14 Nov), Hummingbot with Gateway (18 and 20 Nov) and Chainstack's pump.fun bot (17 Nov). The well-known warp-id sniper and its forks have had no commits since 2024. ScreenerBot is the only one that ships discovery, RugCheck holder filters, breakout and pullback conditions, a partial take-profit ladder of up to four steps, trailing bands, a position cap, per-strategy paper mode and Telegram control in one binary ([ScreenerBot README](https://github.com/farfary/ScreenerBot/blob/9cc0b74/README.md)).

| Candidate | Licence, last commit | What it gives you | What you would build |
|---|---|---|---|
| **ScreenerBot** | BSL 1.1, 14 Nov 2026 | Screener + filters + strategies + ladder + trail + Telegram + dashboard | 5 small patches (below) |
| Hummingbot + Gateway | Apache-2.0, 18/20 Nov 2026 | `PositionExecutor` triple barrier (one TP, SL, time limit, trailing stop), paper connector, Jupiter/Raydium/Meteora/Orca connectors, scrypt + AES-GCM keystore ([Hummingbot](https://github.com/hummingbot/hummingbot); [Gateway](https://github.com/hummingbot/gateway)) | Screener, RugCheck gate, retest signal, multi-step TP, sizing. No pump.fun/PumpSwap connector |
| Chainstack pumpfun-bonkfun-bot | Apache-2.0, 17 Nov 2026 | Launch sniper with `max_token_age` in seconds, a single TP/SL, a fixed buy and a plaintext `.env` key ([repo](https://github.com/chainstacklabs/pumpfun-bonkfun-bot)) | Almost everything. It targets the opposite end of a coin's life |
| outsmart / solana-agent-kit | MIT / Apache-2.0 | Execution libraries with no strategy engine ([outsmart](https://github.com/outsmartchad/solana-trading-cli); [agent-kit](https://github.com/sendaifun/solana-agent-kit)) | The whole bot |
| warp-id, Trench-tools | Stale 2024 / manual | New-pool sniper; manual browser-extension trading that promotes its own token ([warp-id](https://github.com/warp-id/solana-trading-bot); [Trench-tools](https://github.com/0xD3bt/Trench-tools)) | Not suitable |

**Verdict: keep ScreenerBot.** Hummingbot is the better choice only if owning Apache-licensed code or Gateway's stronger keystore matters more to you than two days of build time. Its single take-profit level alone means writing a custom controller for exit B. The search also turned up a hazard. Fake "Solana trading bot" repos are an active key-theft channel. SlowMist traced one whose obfuscated dependency uploaded wallet files, and the repo's stars and forks had probably been inflated by attacker-controlled accounts ([Cointelegraph](https://cointelegraph.com/news/solana-trading-bot-github-malware-scam)). JFrog found 25 malicious npm and PyPI packages aimed at people looking for MEV and trading bots ([JFrog](https://research.jfrog.com/post/solana-fakefix/)). The top results on GitHub's `solana-memecoin-sniper-bot` topic show the same keyword-stuffed pattern ([GitHub topic](https://github.com/topics/solana-memecoin-sniper-bot)). Install nothing from that genre on the trading box.

## Five add-ons earn a place; the rest duplicate ScreenerBot

ScreenerBot already includes most of what a bot stack usually bolts on. It load-balances across RPC providers with a circuit breaker ([rpc.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/rpc.rs)). It simulates every aggregator swap before signing and checks balance conservation through `cost_guard` ([cost_guard.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/swaps/cost_guard.rs)). It also exposes an `/api/health` route that skips auth, which a monitor can probe ([middleware.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/webserver/middleware.rs)). The add-ons that remain fill real gaps.

| Rank | Add-on | Gap it fills | Effort | Notes |
|---|---|---|---|---|
| 1 | **Four free RPC tiers** (Helius, Alchemy, dRPC, Chainstack) plus corrected limits | Load beyond Helius Free's 1M credits | Config | Optional: eRPC (Apache-2.0, active) gives one failover endpoint, but it caches only finalized Solana reads, not account polling, so it adds little ([eRPC](https://github.com/erpc/erpc)) |
| 2 | **Tailscale Free** | Phone and laptop access to a loopback-only dashboard | Low | The free Personal plan covers 6 users ([Tailscale](https://tailscale.com/docs/account/manage-plans/free-plans-discounts.md)). Never use Funnel. Headscale only adds a server to maintain |
| 3 | **Uptime Kuma** | Alerts when the bot or an RPC endpoint dies | Low (Docker) | MIT, active, Telegram notifier, image `louislam/uptime-kuma:2` ([Uptime Kuma](https://github.com/louislam/uptime-kuma)). Use a second Telegram bot, so alerts still arrive when ScreenerBot is down |
| 4 | **Solana Developer MCP, Jupiter Docs MCP, solana-dev-skill, jup-ag/agent-skills** | Current docs for coding agents | Very low | Read-only docs servers ([Solana MCP](https://mcp.solana.com/); [Jupiter MCP](https://developers.jup.ag/docs/ai/mcp.md); [solana-dev-skill](https://github.com/solana-foundation/solana-dev-skill); [agent-skills](https://github.com/jup-ag/agent-skills)). They help most when reviewing ScreenerBot's Jupiter router, which uses `/swap/v1` on `lite-api.jup.ag`, whose "deprecation date [is] TBA" ([jupiter.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/swaps/routers/jupiter.rs)) |
| 5 | **Solana Tracker Data API** (optional; GoPlus as a free alternative) | Bundler, sniper and insider data that RugCheck lacks | Medium (filter code) | `/tokens/{token}/bundlers`, `/insiders/chart/{token}`, first-buyers; pricing not stated ([Solana Tracker](https://docs.solanatracker.io/guides/sniper-detection.md)). GoPlus Free gives 150K CU a month; the next tier is $199 ([GoPlus](https://gopluslabs.io/security-api)). Add this after week one |

The rejects, and why: **solana-agent-kit** duplicates ScreenerBot's swaps and wants a raw secret key in an environment variable ([agent-kit](https://github.com/sendaifun/solana-agent-kit)). **Lighthouse** assertions would need changes to the swap builder, and the project has had no commits since April 2025 ([Lighthouse](https://github.com/Jac0xb/lighthouse)), while `cost_guard` already covers most of the same ground. **Prometheus/Grafana** would need a `/metrics` endpoint that ScreenerBot doesn't have. **TradeTally/TradeNote** import from stock brokers. For a trade journal, query a *copy* of ScreenerBot's SQLite database. OpenRouter isn't an add-on at all. It is already one of ScreenerBot's nine LLM providers ([llm.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/llm.rs)).

## Two-thirds of the strategy is configuration; five rules need code

Your strategy itself doesn't change. Pass/fail safety screen, 24-72h age, $15k liquidity, $50k volume, 5m breakout and retest, a 0-100 score, exits A or B with a −50% stop and a 48h cap, 10% of balance reinvested, two slots with a third above $40, and no averaging down. What changes is where each rule lives. Settings go in `data/config.toml` under the service user's app-data directory. Entry and exit strategies are stored in the strategies database and edited in the dashboard as AND/OR/NOT rule trees, each set to Live or Paper ([strategies/types.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/types.rs)). The real RPC and swap paths are `chains.solana.rpc.*` and `chains.solana.swaps.*`. The README's `[rpc]` and `[swaps]` names are legacy ones that the config migration rewrites ([migrate.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/migrate.rs)).

### The market filter and safety gate

```toml
[filtering]
age_enabled = true
min_token_age_minutes = 1440        # 24h floor; the 72h ceiling lives in the entry strategy

[filtering.dexscreener]
liquidity_enabled = true
min_liquidity_usd = 15000           # already the default
volume_enabled = true
min_volume_24h = 50000              # default 0

[filtering.rugcheck]
enabled = true                      # a token with no RugCheck report never passes
authority_checks_enabled = true
require_authorities_safe = true
allow_mint_authority = false
allow_freeze_authority = false
risk_level_enabled = true
block_danger_level = true
holder_distribution_enabled = true
max_top_holder_pct = 5              # counts pool/LP vaults today: see F1
max_top_3_holders_pct = 15          # interim stand-in for "top 10 < 20%"
min_unique_holders = 150            # > 0, so an unknown holder count rejects
insider_holder_checks_enabled = true
max_insider_total_pct = 10          # default 20
graph_insiders_enabled = true
transfer_fee_enabled = true
block_transfer_fee_tokens = true    # no Token-2022 transfer-fee coins
```

These keys and defaults come from the filtering schema and the RugCheck stage ([filtering.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/filtering.rs); [rugcheck.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/filtering/sources/rugcheck.rs)). Several defaults you never asked for are on: market cap ≥ $50k, 1h volume ≥ $3k, at least 40 transactions an hour, LP lock ≥ 50%, creator balance ≤ 10%, price-change bands (5m −15..+30%), a free on-chain scam-authority stage, and `require_logo_url`. They only narrow the funnel, so leave them on unless the Filtering page shows them rejecting setups you would have taken. That page lists the rejection reason for every token.

The safety gate has three real gaps. **First, the holder check takes the three largest `pct` values from RugCheck's `topHolders` without excluding pool, LP or AMM vault accounts.** The `owner` field is parsed but never used to exclude anything. **Second, there is no top-10 sum**, only top-1 and top-3. **Third, an empty `top_holders` list does not reject**, which breaks your rule that missing data means reject ([rugcheck.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/filtering/sources/rugcheck.rs)). A pool vault often holds far more than 5% of supply, so `max_top_holder_pct = 5` could reject nearly every coin. Whether RugCheck already strips vaults from `topHolders` is unverified. Run Explore Mode (discovery and filtering without a wallet) for an hour on Day 1 morning and read the rejection reasons. Then ship F1 either way, because the top-10 rule and the empty-list rule need code regardless.

### The entry rule tree

Build one ENTRY strategy with `timeframe: "5m"` and mode Live, as an AND of these conditions.

| Node | Condition and params | Purpose |
|---|---|---|
| 1 | `TokenAge {comparison: YOUNGER_THAN, time_value: 72, time_unit: HOURS}` | 72h ceiling. It measures time since on-chain token creation ([token_age.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/token_age.rs)) |
| 2 | **`BreakoutRetest`** (F2) `{timeframe: 5m, resistance_lookback: 24, retest_window: 12, retest_tolerance_pct: 1.5, max_candle_change_pct: 25, require_higher_low: true}` | Your exact entry |
| 3 | **`CandidateScore {min_score: N}`** (F4) | Weighted score, with safety already passed upstream |

Until F2 lands, the closest you can get in config is `PriceBreakout {lookback: 24, direction: UPWARD, confirmation: 0.5}` AND `NOT CandleSize {pattern: LARGE_BODY, threshold: 25}`. That is a breakout *chase*, not your retest. `PriceBreakout` compares the live price with the prior 24-candle high, and nothing remembers that a breakout happened earlier ([price_breakout.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/price_breakout.rs)). `PullbackFromHigh` can't go below a 1% minimum drop and measures from the swing high, not from the broken level ([pullback_from_high.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/pullback_from_high.rs)). `CandleSize` looks only at the last, possibly still-forming candle and also needs body/range ≥ threshold ([candle_size.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/candle_size.rs)). One property works in your favour: a condition that lacks data returns an error, the rule tree marks that leaf unknown, and no entry is taken ([engine.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/engine.rs)). Candle series older than three buckets are refused ([conditions/mod.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/mod.rs)). So "missing data = reject" already holds at the entry stage.

### Exits: A runs exactly today, B needs a runner-only trail

The built-in exit rules run in a fixed order, and the first one that fires wins: safety rules, optional LLM exit, stop loss, **trailing stop, then take profit**, time override, stagnation, and strategy exits last ([exit_rules.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/evaluators/exit_rules.rs)). Two details shape the config. **The time override only closes positions whose P&L is at or below a threshold, and the threshold can't be positive** ([exit_time.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/evaluators/exit_time.rs); [policy.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/policy.rs)), so it never closes a winner. An EXIT strategy signal, on the other hand, **sells the whole position**, as a unit test in the repo asserts ([strategies.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/evaluators/strategies.rs)). Note also that the trailing stop is **on by default** (activation 25%, distance 12%), and exit A must switch it off ([trader.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/trader.rs)).

```toml
[trader]                                # Exit A, exactly
enabled = true                          # master switch, default false
stop_loss_enabled = true
stop_loss_threshold_pct = 50
stop_loss_allow_partial = false
take_profit_enabled = true
take_profit_levels_pct = [900.0]        # 10x = +900% from average entry
take_profit_sell_pct = [90.0]           # of the position as opened; 10% runner remains
trailing_stop_enabled = false           # default is true
break_even_enabled = false
time_override_enabled = true            # backstop: closes losers at 48h
time_override_duration = 48
time_override_unit = "hours"
time_override_loss_threshold_percent = 0
dca_enabled = false
```

Add one EXIT strategy in Live mode, bound to your entry strategy through `entry_strategy_ids`, with the single condition `PositionHoldingTime {hours: 48, comparison: GREATER_EQUAL}` ([position_holding_time.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/position_holding_time.rs)). That sells everything left at 48h, winners included. The take-profit ladder allows up to four ascending steps of 1-1000% each, with sell shares of the original position that sum to at most 100% ([exit_take_profit.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/evaluators/exit_take_profit.rs)), so 10x fits.

Exit B fits the ladder (`[100.0, 400.0]` with `[50.0, 25.0]`), but not the trail. `trailing_stop_activation_pct` is capped at 100%, and because the trail runs *before* take profit, any retracement after 2x sells **everything left**, including the share meant for 5x. A trail that guards only the last 25% after 5x has no config path. Exits A and B also can't run side by side, because stop, ladder, trail and time override are global `trader.*` settings, and only copy-trading tasks get per-task overrides ([policy.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/policy.rs)). So the recommendation is: **go live on exit A, and add F5's `trailing_stop_runner_only` flag on Day 2**. That flag skips the trail until every ladder step has filled, then arms it at the last rung's price with your chosen distance. After F5, switching between A and B is a settings change. A per-entry-strategy exit policy, which would let A and B run at the same time, is a later patch in `resolve_exit_policy()`.

### Sizing, slots and never averaging down

Automatic sizing reads a fixed `trader.trade_size_sol` (default 0.005, minimum 0.001, maximum 10) through `get_trade_size_native()`. The slot count reads `trader.max_open_positions` (default 2) through `get_max_open_positions()` ([trader/config.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/config.rs)). The live capacity check reads config on every entry ([limits.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/safety/limits.rs)). The global position semaphore, however, is a `OnceLock` sized at startup and only reconciled when the positions worker starts ([state.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/positions/state.rs)). Lowering the slot count works live, but **raising it to 3 needs a restart**. F3 fixes both problems. It reads the balance from a cached SOL balance, sizes each entry at 10% of it (clamped between a floor and a ceiling), and sizes the semaphore at 3 while the capacity check applies 2 or 3 depending on a $40 threshold. Until F3 ships, set `trade_size_sol` by hand from the dashboard. "Never average down" is already config: `dca_enabled = false`, and the bot never enters a token it already holds ([limits.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/safety/limits.rs)). Keep the defaults `position_close_cooldown_minutes = 15` and `loss_blacklist_threshold_pct = -15`.

Paper trading stays optional. Live is blocked only when setup is incomplete (or the app is in Explore Mode) or the emergency stop is on ([controller.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/controller.rs)). Paper readiness is labelled "Advisory evidence ... before arming live" ([paper/readiness.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/paper/readiness.rs)), so it gates nothing. To run a shadow, clone your entry strategy in Paper mode. It uses its own paper cash and slots.

### Every forced compromise, and the call on each

| Rule | What ScreenerBot does | Recommended choice |
|---|---|---|
| Pair age 24-72h | Floor in filtering, ceiling only through `TokenAge`. Both measure **token** age, not pair age | **Config workaround.** For pump.fun graduates the pool is younger than the mint, so a few coins pass early. Accept it |
| 48h exit for winners | Time override closes only P&L ≤ 0 | **Config workaround**: an EXIT strategy with `PositionHoldingTime ≥ 48`, plus the time override as a backstop |
| Top-10 ordinary < 20%, largest < 5% | Top-1 and top-3 only, pools counted, empty list passes | **Code: F1**, the first Codex task on Day 1 |
| Insiders < 10% | Sum of RugCheck-flagged top holders | Config (`max_insider_total_pct = 10`) |
| Breakout + retest, no >25% verticals | Live-price breakout with no memory; last-candle size only | **Code: F2.** Chase approximation only until it lands |
| Exit B trail on the last 25% after 5x | Activation capped at 100%; trail sells everything | **Code: F5.** Run A until then |
| A and B side by side | One global policy | **Config**: pick one globally. Per-strategy override deferred |
| 10% of balance, reinvested | Fixed SOL size | **Code: F3**; adjust by hand until then |
| 3rd slot above $40 | Fixed cap; semaphore sized at startup | **Code: F3** (USD threshold from the bot's SOL/USD price; missing price → 2 slots) |
| Weighted 0-100 score | None | **Code: F4** |
| "Genuine buyers" | Buy/sell counts per window only, no distinct wallets ([types.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/types.rs)) | **Compromise inside F4**: counts plus buy ratio. Distinct-wallet data from Solana Tracker later |
| AI can only lower the score | LLM stages can gate entries and even sell first, at exit priority 3 ([exit.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/trader/evaluators/exit.rs)) | **Config**: every `llm_analysis.*` flag off. F4's AI penalty reads a cached verdict if one exists and only subtracts; with none it is 0 |

### The score

F4 adds a `CandidateScore` condition with a `min_score` parameter. A pure function `score(inputs) -> ScoreBreakdown` lives in a fork-owned `src/fork/score.rs`. It takes chart quality from the F2 retest geometry, buyers from `market_window()` buy/sell counts and their ratio, and liquidity from `market_data.liquidity_sol`. It subtracts a capped AI penalty and clamps the result at 0 or above. The safety gate is the filtering pipeline itself, because only passed tokens are ever evaluated. The Telegram "position opened" message is built in `msg_position_opened(...)`, which already has an `ai_reasoning` field ([messages.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/telegram/messages.rs)). F4 adds its own score line there rather than relying on `include_ai_reasoning`.

## Codex writes the fork, Claude specifies and reviews it

### Fork shape and conflict budget

Make the fork a **private mirror**: clone the repo and push it to a new private repo, so your strategy code isn't public. Add upstream as a remote and keep a `fork/main` branch, which is an upstream `v*` tag plus your patch commits, with one branch per feature. Upstream's conventions are strict and enforced by tests, which helps agents if they are told about them. A new condition is one file implementing `ConditionEvaluator` plus three registration lines in `conditions/mod.rs` ([conditions/mod.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/mod.rs)). Unit tests also fail unless the English `.ftl` keys exist and the recorded schema fixture `tools/tests/fixtures/dashboard/trader/strategies_condition_schemas.json` lists the condition ([catalog.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/strategies/conditions/catalog.rs)). A new config field is one `config_struct!` line plus a `config-<section>-<field>` key in `locales/en/config.ftl`, again enforced by a test ([metadata_catalog_tests.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/metadata_catalog_tests.rs)). Tests must run under nextest, because "under plain `cargo test` every test shares one process and that state collides" ([nextest.toml](https://github.com/farfary/ScreenerBot/blob/9cc0b74/.config/nextest.toml)). New test files need a `mod` line in `tests/integration.rs`.

| Task | Files | Estimate |
|---|---|---|
| F0 AGENTS.md, baseline green run | new `AGENTS.md`, `docs/` | 2-3 h, mostly waiting on the build |
| F1 HolderGate | `filtering/sources/rugcheck.rs`, `RugCheckFilters` fields, `config.ftl` | 1-2 h |
| F2 BreakoutRetest | new `conditions/breakout_retest.rs`, registry, `strategies.ftl`, fixture, tests | 3-4 h |
| F3 Dynamic sizing + 3rd slot | `trader/config.rs` accessors, `TraderConfig` fields, new `src/fork/balance.rs`, semaphore init | 3-5 h |
| F4 CandidateScore + Telegram | new `conditions/candidate_score.rs`, `src/fork/score.rs`, `messages.rs` | 4-6 h |
| F5 Runner-only trail | `trader/policy.rs`, `exit_rules.rs`, one field | 2-3 h (my estimate; money path) |

The estimates for F0-F4 come from the code review notes; F5's is mine. Put new fields at the end of their structs and new ftl keys at the end of their files, and mark every upstream-file edit `// FORK:`. That keeps rebases onto new upstream tags mechanical, and `git rerere` reuses past resolutions. Leave `src/chains/solana/swaps/revenue.rs` alone. The 0.5% fee is how upstream gets paid, and the file states it "must never be removed or weakened" ([revenue.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/swaps/revenue.rs)). The BSL 1.1 licence allows modification and personal non-commercial use. Keep `LICENSE` and the per-file BUSL headers, don't sell signals or binaries, and don't reuse the ScreenerBot name ([LICENSE](https://github.com/farfary/ScreenerBot/blob/9cc0b74/LICENSE)). The licence has no Change Date. This is a reading of the text, not legal advice.

### Build machine and where Codex runs

Rust is pinned to **1.98.1** in `rust-toolchain.toml` ([rust-toolchain.toml](https://github.com/farfary/ScreenerBot/blob/9cc0b74/rust-toolchain.toml)). The README's "1.89" is out of date. The build is heavy. `target/` "can grow to 25 GB or more", release builds take 8-15 minutes on 8 cores ([BUILD.md](https://github.com/farfary/ScreenerBot/blob/9cc0b74/BUILD.md)), and upstream CI caps parallelism at 2 jobs because a test build "peaks near 5 GB per rustc process" ([ci.yml](https://github.com/farfary/ScreenerBot/blob/9cc0b74/.github/workflows/ci.yml)). A 4 GB VPS can run the bot but can't build it. Your M4 MacBook Air with 16 GB can run `cargo check` and nextest natively with `CARGO_BUILD_JOBS=2`, as long as it has about 40 GB of free disk. Build the **Linux** binary in GitHub Actions on your private repo. Trim upstream's `build.yml` (six 20-40 minute jobs on `v*` tags ([build.yml](https://github.com/farfary/ScreenerBot/blob/9cc0b74/.github/workflows/build.yml))) down to the single Linux headless job on `ubuntu-22.04`, which keeps glibc below the installer's 2.34+ requirement on the VPS. Expect 20-40 minutes per build. Check your private-repo Actions minutes first. The fallback is a `linux/arm64` Docker build on the Mac with an arm64 VPS, which the installer supports.

Run **Codex CLI locally** for every task. Fetch crates and nextest once with network on, then work with `CARGO_NET_OFFLINE=true` and agent network off:

```toml
# ~/.codex/config.toml
sandbox_mode = "workspace-write"
approval_policy = "on-request"
[sandbox_workspace_write]
network_access = false
```

The older `sandbox_mode` keys can't be mixed with the newer beta `[permissions]` profiles ([Codex permissions](https://learn.chatgpt.com/codex/permissions)). Codex cloud is a poor fit here. Its universal image reportedly ships Rust 1.95.0 ([third-party write-up](https://codex.danielvaughan.com/2026/05/31/codex-cloud-environments-setup-scripts-caching-secrets-codex-universal/)), no CPU or RAM limits are documented, and the agent phase is offline by default ([Codex cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environment)). A Solana-SDK build there could run out of memory. Use it, if at all, for pure-logic test tasks, with a setup script that installs 1.98.1 and runs `cargo fetch`. Codex reads AGENTS.md from the repo root down, up to 32 KiB, and has to be restarted to reload it ([Codex AGENTS.md](https://learn.chatgpt.com/codex/guides/agents-md)). Claude Code reads AGENTS.md when no CLAUDE.md exists ([InfoWorld](https://infoworld.com/article/4224410/claude-code-now-also-accepts-instructions-in-openais-agents-md-format.html)), so one file serves both tools.

### Plans and limits

| Plan | Price | Relevant limit |
|---|---|---|
| ChatGPT Plus (Codex) | $20 | Messages per 5 hours vary 5-10x by model; local and cloud share the allowance ([Codex pricing](https://learn.chatgpt.com/docs/pricing)) |
| Codex Pro | $100+ | "No five-hour limit", weekly caps only ([Codex pricing](https://learn.chatgpt.com/docs/pricing)) |
| Claude Pro | $20 monthly | Claude Code included; 5-hour and weekly limits shared across chat and Claude Code ([Claude pricing](https://claude.com/pricing)) |
| Claude Max | $100 | 5x Pro ([Claude pricing](https://claude.com/pricing)) |

Start on Plus and Pro, $40 in total. Rust plus the fixture and i18n bookkeeping make for fewer, longer Codex sessions than the earlier TypeScript plan. Use the strongest Codex model for F3 and F5 (money paths) and a cheaper one for F0 and the ftl and fixture edits. Move to Codex Pro only if Day 1's afternoon wave hits the cap. Check `/status` before each wave.

### Draft AGENTS.md for the fork

```markdown
# AGENTS.md — personal fork of ScreenerBot (BSL 1.1)
Rust 1.98.1 (rust-toolchain.toml), single crate, headless trading bot. This fork adds small,
default-off features for one user. Spec: docs/SPEC.md · Tasks: docs/TASKS.md · Status: docs/HANDOFF.md

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
- New logic in new files: src/strategies/conditions/<name>.rs, src/fork/<name>.rs.
- Upstream files: only registration lines, fields appended at struct end, accessor bodies. Mark each `// FORK:`.
- Every new config field defaults to upstream behaviour (feature off).
- Every new file: BUSL header + `//!` description (tools/header_check.mjs).

## Checklists
New condition: impl ConditionEvaluator; `mod` + `pub use` + `registry.register` in conditions/mod.rs;
  strategies-condition-<kebab>-* keys in locales/en/strategies.ftl; entry in
  tools/tests/fixtures/dashboard/trader/strategies_condition_schemas.json; tests in tests/strategies_conditions.rs.
New config field: config_struct! line + field_metadata!; config-<section>-<field> (+ .hint) in locales/en/config.ftl.
New test file: `mod` line in tests/integration.rs. Other locales: copy the English text, don't translate.

## Code rules
Missing or stale data returns Err, never false or 0. NaN/zero guard on every float compare.
No unwrap() on money paths. Reach Solana code only via crate::chains::solana (tests/architecture_boundaries.rs).
Closed candles only; state explicitly whether the forming candle is excluded.

## Code Review Rules
Findings only, worst first, file:line, each with a failing scenario. Flag: edits to swaps/presend/cost_guard/
secure_storage/revenue; any default that changes live behaviour; candle-window off-by-one; Err vs false;
double-sell or oversell paths; unguarded float compares; edits outside the task's files.
```

### Example prompts

```text
[Claude, spec, F2] You are writing docs/TASKS.md entry F2 for Codex. Read AGENTS.md,
src/strategies/conditions/{mod.rs,price_breakout.rs,pullback_from_high.rs,candle_size.rs,catalog.rs},
tests/strategies_conditions.rs and tests/common/mod.rs. Specify a BreakoutRetest entry condition:
params with ranges/defaults (timeframe 5m, resistance_lookback 24, retest_window 1-12 default 12,
retest_tolerance_pct 1.5, max_candle_change_pct 25, require_higher_low true); closed candles only;
algorithm in pseudocode (latest candle i whose close > max(high of the 24 before i) = level; every
later close >= level*(1-tol); at least one later low within ±tol of level; last swing low > prior
swing low; reject if any candle in the window moves > 25% open→close or low→high); which inputs
return Err vs false; 10 boundary tests (exactly 1.5%, exactly 25%, 23 vs 24 candles, stale series);
the exact file list per the AGENTS.md checklist. No code. List open questions.
```

```text
[Codex CLI, local] Implement F2 from docs/TASKS.md. Read AGENTS.md first and touch only the files F2
lists. Show the plan, then implement. Run `cargo nextest run -E 'test(/strategies_conditions|catalog/)'`,
then `CARGO_BUILD_JOBS=2 cargo nextest run --all-targets --no-tests=fail`. Report results and update
docs/HANDOFF.md.
```

```text
[Claude Code, review] Review `git diff fork/main...feat/runner-trail` against AGENTS.md Code Review
Rules and docs/TASKS.md#F5. Findings only, worst first, file:line, each with a failing scenario.
Specifically: can the trail fire before the last ladder step fills? Can a trail and a TP both sell
the same tokens in one tick? Does the flag default to upstream behaviour? Is the peak reset when
the runner arms? Does stop loss still take precedence?
```

```text
[Codex, second opinion] codex exec --sandbox read-only "Review this branch's diff against
AGENTS.md Code Review Rules. Findings only."
```

```text
[Claude, rebase] Here is `git diff vOLD..vNEW -- <files we patch>` and `git log fork/main ^vOLD -p`.
List each conflict, the minimal resolution, and any upstream behaviour change that alters our
config or the meaning of our patches.
```

## A hardened VPS runs the bot, and every service is wired to free tiers

### The VPS beats the MacBook

A MacBook Air sleeps, travels with you between Dubai and Dallas, and holds your other keys and accounts. Since the wallet key can be decrypted by anything running as the bot's user on the same machine, the bot should live on a box that does nothing else. The installer recommends **2+ cores, 4 GB RAM, 5 GB free disk and glibc 2.34+** (Ubuntu 22.04+, Debian 12+), and warns below those thresholds ([install.sh](https://github.com/farfary/ScreenerBot/blob/9cc0b74/install.sh)). Pick 2 vCPU / 4 GB / 40 GB SSD running Ubuntu 24.04 in a US or EU region close to your RPC providers. VPS pricing wasn't part of this research, so compare providers.

### Install, then swap in the fork binary

```bash
# 1. Harden first: SSH keys only, PermitRootLogin no, unattended-upgrades on.
curl -fsSL https://screenerbot.io/install.sh -o install.sh && less install.sh   # read before running
sudo bash install.sh                        # verifies SHA-256 against screenerbot.io AND GitHub
sudo screenerbot-manager auto-update off    # the 6-hourly timer stays off
sudo screenerbot-manager password           # long random dashboard password
# 2. Tailscale (tailnet-only HTTPS to the loopback dashboard; never Funnel)
sudo tailscale up && sudo tailscale serve --bg http://127.0.0.1:8080
sudo ufw default deny incoming && sudo ufw allow in on tailscale0 && sudo ufw enable   # after SSH over tailnet works
# 3. Fork deploy (built in GitHub Actions); never run `screenerbot-manager update` on the fork
sudo systemctl stop screenerbot
sudo cp /opt/screenerbot/screenerbot /opt/screenerbot/screenerbot.prev
sudo install -m 755 ./screenerbot-fork /opt/screenerbot/screenerbot
sudo systemctl start screenerbot && sudo screenerbot-manager logs -f
```

The installer creates a `screenerbot` system user with a nologin shell and a systemd unit with `Restart=always`, `UMask=0077`, `NoNewPrivileges`, `PrivateTmp` and `ProtectSystem=full`. Its `ExecStart` points at `/opt/screenerbot/screenerbot`, so dropping in the fork binary is a file copy. Wallets, settings and databases live in `~/.local/share/ScreenerBot` under that user. `backup` and `restore` cover all three ([install.sh](https://github.com/farfary/ScreenerBot/blob/9cc0b74/install.sh)). Turn auto-update off in the app as well. Its defaults (`auto_check`, `auto_download` and `auto_install` all true) would swap your fork for upstream ([updates.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/updates.rs)). Telegram also has an `/update` command ([telegram/commands/mod.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/telegram/commands/mod.rs)), which you never use. Changes to RPC endpoints take effect only on the next launch ([AGENT_CONNECTIONS.md](https://github.com/farfary/ScreenerBot/blob/9cc0b74/AGENT_CONNECTIONS.md)), so restart after editing them. A restart rehydrates open positions and reconciles the semaphore, but exits aren't monitored for the few seconds it takes, so deploy between trades when you can.

Run Uptime Kuma in Docker with host networking so it can reach the loopback-only `127.0.0.1:8080/api/health`. Ports published with Docker's `-p` flag bypass ufw, whereas host networking leaves ufw's deny-incoming in force. Reach Kuma's UI through a second `tailscale serve` port. Add an HTTP monitor on `/api/health`, one monitor per RPC URL, and a separate Telegram bot for its alerts. Kuma on the same VPS can't report the whole VPS going down. A silent Telegram (ScreenerBot sends startup and shutdown notices by default) is your signal for that, or run Kuma on a second free box.

### Service and API map

| Service | Key? | Where it goes | Limit to respect |
|---|---|---|---|
| Helius Free | Key in URL | `chains.solana.rpc.urls` | 10 RPS, 1M credits/mo, 1 sendTx/s ([Helius](https://www.helius.dev/pricing.md)) |
| Alchemy Free | Key in URL | same | 30M CU/mo at 15 RPS ([Chainstack 2026](https://chainstack.com/best-solana-rpc-providers-in-2026/), a competitor's figures) |
| dRPC Free | Key in URL | same | ~100 RPS; monthly quota disputed (≈210M CU vs 50M CU) ([Chainstack 2026](https://chainstack.com/best-solana-rpc-providers-in-2026/)) |
| Chainstack Free | Key in URL | same | 3M requests at 5 RPS ([Chainstack 2026](https://chainstack.com/best-solana-rpc-providers-in-2026/)) |
| RugCheck | None | Built in | ~60/min ([rugcheck api](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/apis/rugcheck/mod.rs)) |
| DEX Screener | None | `filtering.dexscreener.*` | 60/min on discovery endpoints ([DEX Screener](https://docs.dexscreener.com/api/reference)) |
| GeckoTerminal | None | `ohlcv.sources.geckoterminal` (filtering off by default) | 30/min ([ohlcv.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/ohlcv.rs)) |
| Jupiter | Free key from portal.jup.ag | `chains.solana.swaps.jupiter.api_key` (moves from lite-api to api.jup.ag; fees unchanged) ([jupiter.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/swaps/routers/jupiter.rs)) | Free key: 1 RPS, 60 RPM ([Jupiter](https://developers.jup.ag/docs/portal/rate-limits.md)) |
| OpenRouter | Key | `llm.enabled=true`, `llm.default_provider="openrouter"`, `llm.providers.openrouter.{enabled,api_key,model}`, `rate_limit_per_minute=20` | Free models: 20 RPM, 50/day, or 1,000/day after a one-time $10 ([OpenRouter](https://openrouter.ai/docs/api/reference/limits.md)) |
| Telegram | BotFather token | `telegram.{enabled,bot_token,chat_id}`, `commands_require_2fa=true` | ~1 msg/s per chat ([Telegram](https://core.telegram.org/bots/faq.md)) |
| Tailscale, Uptime Kuma | Account / none | VPS, phone, Mac | Free |
| Solana and Jupiter docs MCPs | None | Your Mac's Claude Code and Codex: `claude mcp add --scope user --transport http jupiter https://developers.jup.ag/docs/mcp`; `codex mcp add solana-mcp --url https://mcp.solana.com/mcp` | Docs only. The responses are third-party content |
| Solana Tracker (optional) | API key | `ohlcv.sources.solana_tracker` for candles; a future filter source for bundlers and insiders | 30/min for candles; pricing not stated |

You don't need a screenerbot.io account. Every data-server caller falls back to a direct provider unless you choose `chains.solana.access = "service"` ([data_server/mod.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/data_server/mod.rs)). One sizing detail matters, though. Without the account's candle service, candles come from GeckoTerminal at 30 requests a minute, so the default `ohlcv.max_monitored_tokens = 300` can't all refresh within a 5-minute bar (my arithmetic). Set it to about 100. Stale series fail safe as "no entry", and the dashboard shows which conditions return unknown.

### RPC: spread the load and fix three wrong defaults

The research notes disagree about load. One reads the 500 ms fetcher loop as about 2 calls a second, or 5.2M a month. The code shows the fetcher only sends *due* accounts. Watched pools without a position refresh once per heartbeat (half of `pools.price_cache_ttl_secs`, so 15 s by default) in batches of 50 accounts per `getMultipleAccounts` call. Pools with an open position refresh at least every 5 s ([fetcher_types.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/pools/fetcher_types.rs)). The code-based reading wins. Assuming about 3 accounts per pool, 30 passed tokens cost roughly 0.13 RPS and 200 cost about 0.8 RPS. With wallet polling and swaps on top, a realistic 0.4-0.8 RPS comes to **1.0-2.1M calls a month** (my estimates), which is over Helius Free's 1M on its own and comfortable across four providers.

The three corrections. The default `helius_rate_limit = 50` is five times Helius Free's 10 RPS. Alchemy and dRPC **share** `quicknode_rate_limit` (default 25), which is above Alchemy's 15 ([rate_limiter/mod.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/rpc/rate_limiter/mod.rs)). Chainstack isn't in the detection list, so it falls under `default_rate_limit = 10`, against its 5 ([detection.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/rpc/provider/detection.rs)). Each mismatch leads to 429 errors, which trip the circuit breaker after 5 failures.

```toml
[chains.solana.rpc]
urls = ["<helius-url>", "<alchemy-url>", "<drpc-url>", "<chainstack-url>"]   # drop the public default
selection_strategy = "round_robin"    # spreads credits evenly; "adaptive" may drain the fastest tier
helius_rate_limit = 8
quicknode_rate_limit = 12             # applies to Alchemy and dRPC
default_rate_limit = 4                # Chainstack (detected as Unknown)
stats_enabled = true                  # real numbers on the Services page

[pools]
max_watched_tokens = 300              # default 2000; the cap would be ~8 RPS
[wallet]
watch_enabled = false                 # copy trading unused
```

The selector values `adaptive`, `round_robin` and `priority` are defined in [rpc/types.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/rpc/types.rs).

### Security settings, as design inputs

**The key is obfuscated at rest, not protected.** It is AES-256-GCM under `blake3(machine_id, APP_SALT)` with no user passphrase ([encryption.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/secure_storage/encryption.rs)), so root or the service user can decrypt it. Bloom lost about $953,000 from 48 bot wallets in 94 seconds in October 2026, each drain a transfer signed with the wallet's own key ([Cointelegraph](https://cointelegraph.com/news/bloom-bot-hackers-953k-faster-popcorn)). That is why the design uses a **fresh burner** holding only the bankroll plus roughly 0.02-0.05 SOL for fees and token-account rent (an estimate), sweeps profits out by hand, and keeps the burner's seed offline. A backup restored on another server probably can't decrypt the wallet, because the key comes from the machine id. A passphrase layer (Argon2id mixed with the machine key) is a 4-6 hour fork task, deferred because every headless restart would then need the passphrase.

The rest of the hardening is settings. In `[webserver]`, keep `host = "127.0.0.1"` and turn on `auth_enabled` and `auth_totp_enabled`. The password hash is fast BLAKE3 ([password.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/secure_storage/password.rs)), so the password must be long and random. The server refuses a non-loopback bind without auth anyway ([server.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/webserver/server.rs)). Telegram commands need the same TOTP ([telegram.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/telegram.rs)): log in with `/login` and stay signed in for 30 minutes. Your phone controls are `/status`, `/positions`, `/balance`, `/pause_entries`, `/resume_entries` and `/force_stop`, plus inline Sell. Skip inline DCA. In `[account]`, keep `auto_wallet_signin = false` and `use_gateway_rpc = false`, and leave `referral.code = ""` ([account.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/account.rs)). In `[agent_control]`, the in-app assistant's `trading`, `config` and `system` policies **all default to "allow"** ([agent_control.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/agent_control.rs)). Set `trading = "deny"`, and `config` and `system` to `"ask_user"`, so the OpenRouter assistant can read and post a scheduled Telegram summary but can't trade. ScreenerBot's own MCP server is stdio-only and accepts only localhost, and **new pairings start at full access** ([AGENT_CONNECTIONS.md](https://github.com/farfary/ScreenerBot/blob/9cc0b74/AGENT_CONNECTIONS.md)). An agent on your Mac can't reach it, and running one on the VPS puts a shell-capable agent next to a key it can derive. Skip it for the build. If you add it later, pair with Analysis and Portfolio set to Allow, Config and System to Ask, and Trading Off. Configure no secrets in Codex, keep its agent internet off, and have Claude review any `Cargo.lock` diff for new crates.

## Costs stay near zero except the 0.5% fee on every swap

| Item | Monthly |
|---|---|
| VPS, 2 vCPU / 4 GB | Provider-dependent (not priced here) |
| RPC across four free tiers | $0. Helius Developer is $49 for 10M credits if you outgrow them ([Helius](https://www.helius.dev/pricing.md)) |
| RugCheck, DEX Screener, GeckoTerminal, Jupiter key, Telegram, Tailscale, Uptime Kuma, docs MCPs | $0 |
| OpenRouter | $0, or $10 once for 1,000 requests a day |
| Solana Tracker / GoPlus (optional) | Not stated / $0 free tier, $199 next tier |
| GitHub Actions builds | Within the free private-repo minutes (check your quota) |
| Build tools, month one | ChatGPT Plus $20 + Claude Pro $20 = **$40**; Codex Pro $100 if limits bite |
| **Per swap** | **0.5% platform fee** on every router, every direction ([revenue.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/chains/solana/swaps/revenue.rs)), plus network fee, a 50,000 micro-lamport/CU priority fee ([swaps.rs](https://github.com/farfary/ScreenerBot/blob/9cc0b74/src/config/schemas/swaps.rs)), DEX fee and slippage |

The fee works out to 0.5% of the buy plus 0.5% of every sale's proceeds. On a $2 position (10% of $20) stopped at −50%, that is $0.010 + $0.005, or **0.75% of stake**. On an exit-A winner it is 0.5% of $18 at 10x, which hardly matters. Compared with the earlier from-scratch design, which paid Jupiter's 10 bps on buys, nothing on `/build` exits, and a ≥ 0.001 SOL Helius Sender tip on every max-path stop, ScreenerBot is **cheaper on stop sales below 0.2 SOL** (0.5% × 0.2 SOL = 0.001 SOL) and dearer on every buy and take-profit (my arithmetic). The default priority fee of 50,000 micro-lamports per CU is 0.05 lamports per CU, so at about 300k CU per swap the network cost stays near 0.00002 SOL per transaction (an estimate). Liquidity impact matters more than either fee as positions grow. In a $15k constant-product pool, a $50 buy loses about 0.91% to fee plus impact and $500 loses 6.47% ([Raydium](https://docs.raydium.io/algorithms/constant-product)).

## Two days, hour by hour

The Codex windows open at about 08:30 on Day 1 and 08:00 on Day 2, and reset roughly every 5 hours. Claude's windows are offset so reviews land between Codex waves. The reset times shown in each tool are the authoritative ones.

**Day 1: config live on exit A, F1 and F2 shipped**

| Time | Codex | Claude | You |
|---|---|---|---|
| 07:00-08:30 | | claude.ai: turn this report into `docs/SPEC.md`, `docs/TASKS.md` (F0-F5) and AGENTS.md | Buy the VPS, harden SSH, Tailscale on VPS + phone + Mac. Create the burner. Get keys: Helius, Alchemy, dRPC, Chainstack, Jupiter, OpenRouter. Make two Telegram bots |
| 08:30-09:30 | **F0** local: commit AGENTS.md, `cargo fetch`, nextest, warm `cargo build`, baseline `nextest run --all-targets` | | Run install.sh, auto-update off, password + TOTP. Explore Mode with the filter TOML. Private mirror repo; trimmed Linux build workflow |
| 09:30-10:30 | | Claude Code: write F1 and F2 specs from the real files | Read the Filtering page rejection reasons: does `max_top_holder_pct=5` reject everything? Feed the answer into the F1 spec |
| 10:30-13:00 | **F1** and **F2** in two git worktrees | | Uptime Kuma, Telegram `setup`/`test`, RPC TOML + restart, OpenRouter assistant (read-only policies) |
| 13:00-14:00 | | Review F1 and F2 (findings only) | Lunch |
| 14:00-15:00 | Fix-ups; merge to `fork/main` | | CI builds the Linux binary |
| 15:00-16:00 | | | Deploy the fork binary. Import the burner (Full mode). Exit A TOML + 48h exit strategy. Entry strategy (TokenAge + BreakoutRetest) in Live. `trade_size_sol` = 10% of balance by hand. `max_open_positions = 2` |
| 16:00-17:00 | | | Essential tests 4-6 (below): tiny manual buy and sell, kill switch, restart with a position open |
| 17:00 | | | **`trader.enabled = true`: live** |
| 18:30-21:00 | **F3** sizing + third slot (strongest model) | Write F4 and F5 specs at 18:00 | Watch the first signals on Telegram |
| 21:00-22:00 | | Review F3 | Merge; hold the deploy until Day 2 |

**Day 2: F3-F5 live, option to switch to exit B**

| Time | Codex | Claude | You |
|---|---|---|---|
| 08:00-11:00 | **F4** score + Telegram line and **F5** runner-only trail, in parallel worktrees | | Check overnight trades and RPC stats; adjust `max_watched_tokens` |
| 11:00-12:30 | | Review F4; **fresh-session review of F3 + F5** (money paths) | |
| 12:30-14:00 | Fix-ups; full nextest | | CI build |
| 14:00-15:00 | | | Deploy between trades. Turn on `dynamic_size_enabled` (10%, floor 0.001, threshold $40 → 3 slots). Add `CandidateScore {min_score}` to the entry tree |
| 15:00-16:00 | | | Choose A or B. For B: `[100,400]`/`[50,25]`, `trailing_stop_runner_only = true`, trail distance of your choice |
| 16:00-17:30 | Optional: paper-mode clone of the entry strategy as a shadow | Rebase runbook in HANDOFF.md | `screenerbot-manager backup` copied off-box; seed stored offline; check Kuma alerts fire |
| 17:30-20:00 | Buffer | Buffer | Live |

If you want trades before F1 and F2 land, you can arm the config-only chase entry at about 11:00 on Day 1. It isn't your retest rule, and the holder gate still has F1's gaps.

## Seven tests protect the money paths

| Test | Catches |
|---|---|
| 1. Baseline `cargo nextest run --all-targets` green before any change | A broken fork base hiding behind your patches |
| 2. F2 boundary tests: exactly 1.5%, exactly 25%, 23 vs 24 candles, forming candle excluded, stale series → Err | A wrong entry on every signal |
| 3. F1, F3, F5 unit tests: pool account excluded, top-10 exactly 20%, empty list → reject; size floor, ceiling and slot flip at $40, missing price → 2 slots; trail silent until the last rung, stop still first | Oversized trades, a leaky safety gate, sells that empty the ladder |
| 4. One tiny live round trip from the dashboard | Router, fee, P&L record and Telegram alert all working on mainnet |
| 5. `/login` with TOTP, then `/pause_entries` and `/force_stop` from the phone | A kill switch that doesn't respond |
| 6. Restart with a position open | Positions rehydrate, exits resume, semaphore counts correctly |
| 7. Fresh-session Claude review of F3 and F5 before they go live | Logic errors in the code that moves money |

## Conclusion

Switching from a custom build to ScreenerBot changes the risk profile more than the feature list. The earlier design concentrated its engineering in a narrow keyless watcher plus one allowlisted signer. ScreenerBot brings a mature screener and exit engine but one process holding a key it can derive itself. The burner's balance therefore becomes the main security control, and sweeping profits by hand is part of the strategy, not housekeeping. The fork stays cheap to maintain only if it stays thin: five features, all in new files and all default-off, so each upstream release is a rebase and a test run rather than a merge project. The economics also flip compared with the earlier design. At today's position sizes the 0.5% fee costs less than the old Sender tips on stops. Once reinvestment pushes single sales past about 0.2 SOL, it becomes the largest cost per trade, and that is the point to review the exit ladder's number of sells.

Not financial advice: this is engineering research for your own bot; trade at your own discretion.
