//! Terrain in combat: battle odds and whole-game effects of candidate rules.
//!   cargo run --release --example terrain_report -- [games]
use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::*;
use risk_england_sim::events::EventMode;
use risk_england_sim::game::*;
use risk_england_sim::rng::Rng;

fn variants() -> Vec<(&'static str, TerrainCombat)> {
    let none = TerrainCombat::default();
    vec![
        ("no terrain combat (rulebook)", none),
        (
            "mountain: defender +1 on top die",
            TerrainCombat {
                mountain_def_plus: 1,
                ..none
            },
        ),
        (
            "mountain: attacker loses 1 climbing",
            TerrainCombat {
                mountain_climb_cost: 1,
                ..none
            },
        ),
        (
            "river: attacker max 2 dice",
            TerrainCombat {
                river_att_max: 2,
                ..none
            },
        ),
        (
            "river: defender +1 on top die",
            TerrainCombat {
                river_def_plus: 1,
                ..none
            },
        ),
        (
            "mountain +1 def, river max 2 dice",
            TerrainCombat {
                mountain_def_plus: 1,
                river_att_max: 2,
                ..none
            },
        ),
        (
            "climb 1, river max 2 dice",
            TerrainCombat {
                mountain_climb_cost: 1,
                river_att_max: 2,
                ..none
            },
        ),
        (
            "climb 1, ford 1 (lose 1 at either)",
            TerrainCombat {
                mountain_climb_cost: 1,
                river_cross_cost: 1,
                ..none
            },
        ),
    ]
}

fn battle(rng: &mut Rng, table: &DefenderTable, a0: u32, d0: u32, m: Mods, cost: u32) -> bool {
    let mut a = a0.saturating_sub(cost);
    let mut d = d0;
    while a > 0 && d > 0 {
        let (x, y) = roll_mod(rng, a.min(3), d, DiceRule::DefenderOneByOne, table, m);
        a -= x.min(a);
        d -= y.min(d);
    }
    d == 0
}

fn main() {
    let games: u64 = std::env::args()
        .nth(1)
        .map(|x| x.parse().unwrap())
        .unwrap_or(20_000);
    let table = DefenderTable::build();
    let fights = [
        (3u32, 1u32),
        (5, 2),
        (6, 3),
        (10, 5),
        (10, 10),
        (15, 10),
        (20, 10),
    ];

    println!("== Attacker wins, committed attackers v defenders, fought to the end (one-by-one dice, 40,000 each)");
    println!("   (mountain rules shown for a mountain target, river rules for a river crossing)");
    print!("{:<38}", "");
    for (a, d) in fights {
        print!("{:>9}", format!("{a} v {d}"));
    }
    println!();
    for (name, t) in variants() {
        let m = Mods {
            def_plus: t.mountain_def_plus + t.river_def_plus,
            att_max: if t.river_att_max > 0 {
                t.river_att_max
            } else {
                3
            },
            ..Mods::default()
        };
        print!("{name:<38}");
        for (a, d) in fights {
            let mut rng = Rng::new(9);
            let n = 40_000;
            let cost = t.mountain_climb_cost.max(t.river_cross_cost);
            let w = (0..n)
                .filter(|_| battle(&mut rng, &table, a, d, m, cost))
                .count();
            print!("{:>8.1}%", 100.0 * w as f64 / n as f64);
        }
        println!();
    }

    println!("\n== Whole games: house rules + events + pacts, 4 players ({games} games each)");
    println!("{:<38} attacks into  plain/mtn/river | conquest rate plain/mtn/river | rounds | winner terr | comebacks", "");
    let strats = [
        Strategy::LOYAL,
        Strategy::OPPORTUNIST,
        Strategy::AGGRESSIVE,
        Strategy::HOARDER,
    ];
    for (name, t) in variants() {
        let cfg = Config {
            timeline: Timeline::HouseCircle,
            last_century_round: true,
            dice: DiceRule::DefenderOneByOne,
            events: EventMode::Deck,
            pacts: Some(PactRules::default()),
            terrain: t,
            ..Config::default()
        };
        let threads = 2u64;
        let per = games.div_ceil(threads);
        let parts: Vec<[f64; 10]> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..threads)
                .map(|th| {
                    sc.spawn(move || {
                        let sh = Shared::new();
                        let mut acc = [0f64; 10];
                        for g in th * per..((th + 1) * per).min(games) {
                            let rot = (g % 4) as usize;
                            let seat: Vec<Strategy> =
                                (0..4).map(|s| strats[(s + rot) % 4]).collect();
                            let r =
                                Game::new(cfg, &sh, &seat, g.wrapping_mul(0x9E37_79B9) + 3).play();
                            for k in 0..3 {
                                acc[k] += r.terrain_attacks[k] as f64;
                                acc[3 + k] += r.terrain_wins[k] as f64;
                            }
                            acc[6] += r.rounds as f64;
                            acc[7] += *r.territories.iter().max().unwrap() as f64;
                            let mid = r.snapshots[r.snapshots.len() / 2].leader;
                            if r.winners.len() == 1 && r.winners[0] != mid {
                                acc[8] += 1.0;
                            }
                            acc[9] += 1.0;
                        }
                        acc
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut a = [0f64; 10];
        for p in parts {
            for i in 0..10 {
                a[i] += p[i];
            }
        }
        let g = a[9];
        println!(
            "{name:<38} {:>6.1} /{:>5.1} /{:>5.1}          | {:>5.1}% / {:>5.1}% / {:>5.1}%           | {:>5.1}  | {:>6.1}      | {:>5.1}%",
            a[0] / g, a[1] / g, a[2] / g,
            100.0 * a[3] / a[0], 100.0 * a[4] / a[1].max(1.0), 100.0 * a[5] / a[2].max(1.0),
            a[6] / g, a[7] / g, 100.0 * a[8] / g
        );
    }
}
