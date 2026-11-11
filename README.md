# risk-england-sim

Monte Carlo simulator for **Risk: The Battle for Medieval England** (English Heritage edition, Winning Moves 2023), built to test house rules before trying them on friends.

Bots play thousands of full games per second-ish; experiments compare rule variants by game length, seat fairness and which strategies win.

```sh
cargo test --release                                  # board, combat maths, engine sanity
cargo run --release -- all                            # every experiment, rulebook rules
cargo run --release -- cards --house --games 50000    # one experiment, house rules
```

Experiments: `pace`, `dice`, `cards`, `powers`, `styles`, `all`.
Flags: `--games N` (default 20000), `--seed S`, `--threads T`, `--house` (base config = the group's house rules).

Standard library only (written in a sandbox without crates.io). Adding `rand`, `rayon`, `serde`, `clap` is fine.

## Layout

| File | What it holds |
|---|---|
| `src/board.rs` | 42 territories (region, mountain, card stars), 99 connections (land / river / sea), region bonuses, trade and territory charts |
| `src/cards.rs` | 10 Mission cards, 10 Power cards |
| `src/combat.rs` | Dice rules, including the exact "defender sees the attack roll" table |
| `src/game.rs` | Engine: one full game, `Config` switches for rule variants |
| `src/bot.rs` | Bot personalities (parameter sets, not AI) |
| `src/sim.rs` | Parallel runner, strategies rotated through seats |
| `src/main.rs` | CLI experiments |

## Rules model

### Rulebook (default `Config`)
- **Setup**: deal all 42 territory cards (uneven split: last seats get the extra), 1 infantry per card star (deck: 18 one-star, 24 two-star, 66 stars). Reshuffle the cards into the draw pile. 2 Missions and 2 face-down Powers per player. Cavalry on the 5th century.
- **Turn**: place troops, attack, 1 manoeuvre, draw cards, advance cavalry.
- **New troops**: 3 every turn + territory chart (12-14 => +1 ... 40-42 => +10) + region bonuses (North 5, Midlands 7, East Anglia 3, South East 5, West Country 3) + mission bonus earned last turn + card trades.
- **Card trades**: troops by total stars traded: 2→2, 3→4, 4→7, 5→10, 6→13, 7→17, 8→21, 9→25, 10→30. Bots always trade the largest set up to 10 stars.
- **Cards drawn**: 1 per territory conquered that turn.
- **Combat**: attacker 1 die per committed troop (max 3), defender 1 per troop (max 2), ties to defender.
- **Manoeuvre**: one move along a chain of own territories (borders or sea lines). Crossing a river: max 5 troops, and they stop on the far side. Entering a mountain territory: they stop there. Attacks are not affected by rivers or mountains.
- **Timeline**: cavalry moves 1 space after every player's turn. Leaving the 7th, 8th, 10th and 13th century needs the printed even/odd roll (modelled as 50%). Reaching the 15th ends the game; most territories wins (ties split).
- **Elimination**: the eliminator takes the victim's territory cards (classic Risk; the rulebook is silent).

### House interpretations (agreed with the owner)
- Rivers sit on **borders**, not territories. River crossings: Leeds–Beeston, Leeds–Bolsover, Leeds–Conisbrough, Donnington–Portchester, Donnington–Farnham, Farnham–Jewel Tower, Jewel Tower–Dover, Colchester–Dover. Mountains: Carlisle, Brougham, Cliffords, Beeston, Wall, St Briavels, Old Sarum.
- Sea routes: Whalley–St Briavels, Barnard–Bolingbroke, Bolingbroke–Dover, Dover–Portchester, Portchester–Carisbrooke, Tintagel–Scilly.
- Mission "hold Jewel Tower for 3 turns" = 3 turns in total, any player's turns count.
- Power "Heavy snowfall": only the flipping player's North territories.
- Power losses never take a territory below 1 troop. Fixed losses (lose 2 / lose 3) must come from a territory that can take the full loss if one exists, else the largest.

### House rules (switches in `Config`)
- `Timeline::HousePace`: cavalry moves 1 space after every full round; each player rolls a d6 after their turn and a 6 moves it 1 extra space. No gates.
- `DiceRule::DefenderSees`: attacker rolls first; defender sees it, then chooses 1 or 2 dice (bots use the exact best choice).
- `CardDraw::OnePerTurn`: classic Risk card draw, for comparison.
- `Timeline::Off`: the rulebook's "no timeline" variation.
- `EventMode::Deck` / `EventMode::Force(ev)`: century events (`src/events.rs`, text in `docs/house-rules.html`). Every century the cavalry lands on rolls one of its 3 events; it lasts until the cavalry moves. Pact events are no-ops until pacts exist.

### Bot simplifications (known gaps)
- Missions are scored passively (bots get the bonus when they happen to meet one; they only mildly favour mission territories).
- Defenders use at most one keepable Power per battle, only when outnumbered. Artillery is used before attacking a territory with 5+ troops.
- One manoeuvre: largest interior stack toward the nearest most threatened frontier.
- Unit pieces (infantry 1 / cavalry 5 / artillery 10) are just troop counts.
- 2-player mode (3 neutral armies) not implemented.

## First results (20,000 games per row, seed 1)

**Timeline pace** (4 balanced bots, rulebook otherwise):

| Timeline | Rounds | Turns per player | Seat 1 / 2 / 3 / 4 win share |
|---|---|---|---|
| Official | 3.9 | 3.5 | 34% / 27% / 22% / 17% |
| House pace | 6.3 | 5.9 | 25% / 26% / 25% / 25% |
| No timeline | 20.0 | 16.0 | 32% / 27% / 22% / 19% |

The official timeline ends mid-round, so earlier seats get more turns: seat 1 wins twice as often as seat 4. The house pace advances on round boundaries and removes the seat bias almost completely, while giving ~70% more turns.

**Dice rule** (house pace): `DefenderSees` barely changes outcomes for these bots (battles won 85.7% → 84.9%), because they only attack at 1.5:1 or better, where the rule matters little. It mostly bites in even fights, which these bots avoid.

**Card trading** (house rules, trade threshold in stars): trade at 3 → 29.6%, at 6 → 29.9%, at 8 → 24.8%, at 10 (hoard) → 15.6%. Hoarding loses: with 1 card per capture, cards are the main troop source (~85 troops per player per game, vs ~30 from the base 3 + bonuses), and early troops snowball into more captures and more cards.

**Power cards**: always flipping wins 30.7% vs never flipping 19.3%. Loss cards are cheap because the player chooses where they hit (plague on a 2-troop territory costs 1 troop), while Artillery and the defensive cards are strong.

**Play styles**: aggressive 56%, balanced 30%, hoarder 13%, cautious 2%. Attack early and often is strongly favoured under card-per-capture.

Win shares carry ±0.6 percentage points (95%).

## Century events: engagement report

`cargo run --release --example event_report -- 20000 [threads] [name filter]` (4 players: balanced x2, aggressive, hoarder; bots keep a rear guard of 2). Env overrides: `DEFECT=n` troops placed on defecting territories (default 1), `REVOLT=n` territories in Peasants' Revolt (default 3), `FAMINE_HALF=0|1` (default 1).

Findings that shaped the deck:
- A century lasts only ~0.6 rounds (2-3 turns), and the 14th usually only half a round before the game ends. Effects that build up over turns barely act; immediate effects and final-count rules do.
- The game snowballs: from the 8th century on, the current leader wins 68-84% of games.
- Troop-only losses don't change who leads; the winner is decided by territories. Events that move territories (defection) or change the final count matter most.
- v1 deck (events hitting everyone): comebacks 23.1% → 24.6%. Reworked deck (events leaning against the leader, defection, Great Famine as a final-count rule): comebacks 23.1% → 36.3%, lead changes 1.58 → 1.93, final margin 21.7 → 15.9 territories, seats still ~25% each.
- Strongest: Great Famine (leader wins 69% vs 84%), Peasants' Revolt (76% vs 84%), One kingdom and Domesday survey (+3.8 pts comebacks), Petty kingdoms (+3.5).
- Flavor (tactics only, per bots): Roman roads, Plague of Justinian, Age of saints, Offa's Dyke, Hard winter, Norman castles, Great floods, Supply problems, Black Death. Bots don't exploit movement rules, so real effect may be larger.
- Defection tuning (owner felt 3 territories x 3 troops = 9 free troops was too much): 3 territories x 1 troop keeps most of the effect (leader wins 77.4% vs 76.4% with 3 troops, normally 84.3%), 2 x 2 is milder (79.5%). Default: 1 troop.
- Great Famine tuning (owner felt "1-troop territories don't count" was too harsh): counting them as half gives leader wins 80.4% (vs 68.9% harsh, 84.3% normal). Default: half. Bots don't reinforce thin land when the famine is announced; humans would, so the real effect is smaller still.
- With these defaults the full deck gives comebacks 23.1% → 33.7%, final margin 21.7 → 16.4 territories.

## Player-facing pages

`docs/house-rules.html` (house rules with a rollable century event deck, 33 events) and `docs/board-graph.html` (interactive board graph) are the pages the group uses at the table. Keep them in sync with the engine when rules change.

## Roadmap
1. **Alliance pacts**: stake 1 card each for 1-2 turns, 2 each for 3-4 turns, max 2 per player; honoured pact returns stakes plus bonus cards = turns − 1 (cap 3); one pact at a time; length counted in turns, min 1 turn each, renewal allowed; betrayer gets at most 1 card that turn, victim takes the stake pile, betrayer is an oathbreaker (no pacts) until the next century.
2. **Century events**: era-wide modifiers, 3 per century for the 5th-14th (the 15th ends the game), listed in `docs/house-rules.html`. They escalate: early boons, middle combat/movement changes, late punishments for big stacks and the leader. Stack thresholds were calibrated with `cargo run --release --example stacks_by_century -- 4 5000` (board state when each century begins). Note: with house pace, each century is reached only ~55-60% of the time (a 6 skips it), so ~6 events fire per game.
3. Bots that pursue missions, and a betrayal-aware diplomacy bot.
4. Sweep tools: grid over a parameter (e.g. card chart, timeline length), CSV output, charts.
