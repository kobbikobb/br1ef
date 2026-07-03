# br1ef — Agent Workflow

Every change goes through: **Plan → Challenge → Implement → Validate**

## The loop

```
┌──────────────────────────────────────────────────┐
│  1. Plan                                          │
│     Write or update the milestone in PLAN.md      │
│     Include clear DoD for each step               │
├──────────────────────────────────────────────────┤
│  2. Challenge                                     │
│     Read existing code + docs.                    │
│     Stress-test the plan before writing code:     │
│     • Does this contradict any prior decision?    │
│     • Does every new type connect to something?   │
│     • Are there edge cases not handled?           │
│     • Is there a simpler way?                     │
│     • Does the DoD actually prove completion?     │
├──────────────────────────────────────────────────┤
│  3. Implement                                     │
│     Write the code. Follow code style rules.      │
│     • No comments unless WHY is non-obvious       │
│     • Match existing patterns in the repo         │
│     • One conceptual change per commit            │
├──────────────────────────────────────────────────┤
│  4. Validate                                      │
│     • Build: cargo build                          │
│     • Test: cargo test                            │
│     • Lint: cargo clippy -- -D warnings           │
│     • Demo-able: can you run it and see it work?  │
│     • DoD: every checkbox in the milestone is ✓   │
└──────────────────────────────────────────────────┘
```

## Milestone DoD requirements

Every milestone step must define **how you know it's done**.

If you can't describe how to validate a step, the plan isn't complete.

## Rules

- **Never skip Challenge phase** — if you're about to implement, first challenge.
- **All changes go through pull requests** — create a branch, push, open a PR, get it reviewed and merged. Never push directly to main.
- **One concept per commit** — keep changes small and reviewable.
- **Update PLAN.md as you go** — check off items, note blockers.
- **Follow software engineering best practices** — write tests, handle errors, validate inputs, keep functions small, prefer clarity over cleverness.
- **Never commit PII** — no real email addresses, passwords, tokens, API keys, or personal data in source code. Use `.env` for secrets, `.env.example` for templates.
- **No comments unless WHY is non-obvious** — never restate code, never narrate plans.
- **Lean over clever** — less code is better than more abstraction.
- **Match existing patterns** — naming, file structure, imports, idioms.
