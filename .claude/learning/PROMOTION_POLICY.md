# Promotion Policy

## Occurrence Gate
- `occurrences >= 2` AND `confidence` in (high, medium) → eligible for promotion
- `occurrences == 1` AND `confidence == high` AND catastrophic cost if missed → may promote with justification
- Otherwise: remain candidate

## Decision Matrix
| Destination | Condition | Action |
|---|---|---|
| rule | High-confidence, cross-project, recurrence likely | Add to `.claude/rules/` or `.claude/skills/` |
| skill | Process-oriented, repeatable workflow | Add to `.claude/skills/` |
| hook | Needs to intercept specific tool calls | Add to `.claude/hooks/` |
| memory | Project-specific, low recurrence risk | Add to auto-memory via `animus_remember` |
| note | Explains external system or pattern | Add to `notes/topics/` |

## Promotion Log
- 2026-07-24: rule candidate #1 (duplicate lookup tables) — 1 occurrence, promoted on high confidence + catastrophic TTK cost
- 2026-07-24: rule candidate #2 (loose test assertions) — 1 occurrence, promoted on high confidence + safety impact
