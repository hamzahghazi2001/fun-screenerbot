# Role: planner, reviewer, orchestrator. Claude writes code only for the money tasks below.
Antigravity (agy) writes all other code. Read AGENTS.md and docs/SPEC.md once per session.
Loop per task:
1. Write tasks/TXX.md: goal, files to touch (max 4), acceptance tests by name.
2. Run: scripts/agy-task.sh TXX [model]
3. Review only `git diff` of the touched files + the named tests:
   cargo nextest run <filter> 2>&1 | tail -30 ; cargo clippy 2>&1 | tail -30
4. If wrong: scripts/agy-task.sh TXX --fix "<exact problem>"  (max 2 fixes, then stop and ask me)
5. If green: git add -A && git commit -m "TXX: <summary>"; tick it in docs/TASKS.md.

## Routing (agy's Claude models are quota-capped until about 2026-11-30; agy means Gemini only)
a) agy, Gemini only:
   - gemini-3.8-flash-medium: tasks that only add config fields, locale text (.ftl), condition
     registration, fixtures, the Telegram score line, CI or docs
   - gemini-3.8-flash-high: other logic tasks that are not Claude's (boundary/behaviour tests)
   scripts/agy-task.sh ends every prompt (new run and --fix) with the named-tests-only /
   no-BypassSandbox line; task files and --fix messages need not repeat it.
   The reviewer (Claude) runs the full suite before committing.
b) Claude writes the code itself, with Sonnet-level effort, only for the money logic:
   dynamic sizing calculation and its wiring (T09, T10), CandidateScore evaluation (T11, T12a),
   runner-only trail logic (T16). Model tag `claude` in docs/TASKS.md.
   Still write tasks/TXX.md first, keep to its file list, run the named tests and the full suite,
   then do the fresh-session review that docs/TASKS.md lists (SPEC test 7) before relying on it.
c) Claude never edits anything under src/chains/solana/swaps or src/wallets. If a task needs a change
   there, stop and ask me. Stop and ask me before committing anything under those paths.

Keep context small: never open whole large files; /clear between tasks.
