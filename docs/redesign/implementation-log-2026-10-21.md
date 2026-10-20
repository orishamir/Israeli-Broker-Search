# Implementation log, 2026-10-21 (Part 4, built unattended)

Judgement calls made while building `REDESIGN.md` Part 4 with nobody to ask. Each entry: what, why, where. Settled by Part 1, then the closest decision in Part 4, then today's behaviour.

## Orchestration

- **What:** P1 (engine, data, core copy) and P2 (questions card) ran first in parallel; P3 (answer card, path) and P5 (chart, where to buy, gain, horizon, facts) after P1 merged; P4 (phone) after P2 and P3; P6 review last. **Why:** §4.12's dependencies; P1 does its shapes and bindings itself rather than handing a stub, since the UI streams start only after it merges. **Where:** this round.
