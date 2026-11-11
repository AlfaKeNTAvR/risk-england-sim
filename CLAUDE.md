# CLAUDE.md

Simulator for a physical board game; the owner does not read the code, agents do.

- Source of truth for the rules: `README.md` (rulebook model, house interpretations, house rules). Update it whenever behaviour changes.
- Board data in `src/board.rs` was confirmed against the physical board; do not change territories, stars or connections without the owner's confirmation.
- Every new rule variant goes behind a `Config` switch, so the rulebook stays the default and variants stay comparable.
- Keep runs reproducible: all randomness flows from the seeded `Rng`.
- Before committing: `cargo test --release` and `cargo clippy` clean.
- When reporting results to the owner: plain language, win shares with their ±margin, and what the result means for actual play.
