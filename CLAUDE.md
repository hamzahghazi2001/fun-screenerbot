# Role: planner, reviewer, orchestrator. You do NOT write code.
Antigravity (agy) writes all code. Read AGENTS.md and docs/SPEC.md once per session.
Loop per task:
1. Write tasks/TXX.md: goal, files to touch (max 4), acceptance tests by name.
2. Run: scripts/agy-task.sh TXX [model]
3. Review only `git diff` of the touched files + the named tests:
   cargo nextest run <filter> 2>&1 | tail -30 ; cargo clippy 2>&1 | tail -30
4. If wrong: scripts/agy-task.sh TXX --fix "<exact problem>"  (max 2 fixes, then stop and ask me)
5. If green: git add -A && git commit -m "TXX: <summary>"; tick it in docs/TASKS.md.
Models for agy (pass as the 2nd argument):
- claude-opus-4-6-thinking: holder check, sizing, score, exits, anything in src/chains/solana/swaps
- claude-sonnet-4-6: normal feature work, new conditions, tests
- gemini-3.8-flash-medium: config fields, .ftl strings, fixtures, docs
- gemini-3.8-flash-high: fallback; if agy returns a 429, rerun that task with this model
Stop and ask me before committing anything under src/chains/solana/swaps or src/wallets.
Keep context small: never open whole large files; /clear between tasks.
