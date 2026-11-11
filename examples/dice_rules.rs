//! Compare defence dice rules: exact per-roll expectations, whole battles,
//! and whole games.  cargo run --release --example dice_rules -- [games]
use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::*;
use risk_england_sim::events::EventMode;
use risk_england_sim::game::*;
use risk_england_sim::rng::Rng;
use risk_england_sim::sim::run;

const RULES: [(&str, DiceRule); 3] = [
    ("classic (both roll together)", DiceRule::Classic),
    ("defender sees, picks 1 or 2", DiceRule::DefenderSees),
    ("defender sees, rolls one by one", DiceRule::DefenderOneByOne),
];

fn main() {
    let games: u64 = std::env::args().nth(1).map(|x| x.parse().unwrap()).unwrap_or(20_000);
    let table = DefenderTable::build();

    println!("== One roll, 3 attack dice vs 2 defenders (exact, all 6^5 rolls weighted)");
    // Monte Carlo with many samples is simpler than enumerating the sequential rule.
    for (name, rule) in RULES {
        let mut rng = Rng::new(42);
        let n = 4_000_000u64;
        let (mut al, mut dl) = (0u64, 0u64);
        for _ in 0..n {
            let (x, y) = roll_once(&mut rng, 3, 2, rule, &table);
            al += x as u64;
            dl += y as u64;
        }
        println!("{name:<34} attacker loses {:.3}, defender loses {:.3} per roll", al as f64 / n as f64, dl as f64 / n as f64);
    }

    println!("\n== Whole battles: attacker win chance, fought to the end (20,000 each)");
    let fights = [(4u32, 2u32), (6, 3), (10, 5), (11, 10), (16, 10), (21, 10)];
    print!("{:<34}", "");
    for (a, d) in fights {
        print!("{:>9}", format!("{a} v {d}"));
    }
    println!();
    for (name, rule) in RULES {
        print!("{name:<34}");
        for (a0, d0) in fights {
            let mut rng = Rng::new(7);
            let mut wins = 0;
            for _ in 0..20_000 {
                let (mut a, mut d) = (a0, d0);
                while a > 1 && d > 0 {
                    let (x, y) = roll_once(&mut rng, (a - 1).min(3), d, rule, &table);
                    a -= x;
                    d -= y;
                }
                if d == 0 {
                    wins += 1;
                }
            }
            print!("{:>8.1}%", 100.0 * wins as f64 / 20_000.0);
        }
        println!();
    }

    println!("\n== Whole games, house rules + events + pacts ({games} games each)");
    let strats = [Strategy::BALANCED, Strategy::BALANCED, Strategy::AGGRESSIVE, Strategy::CAUTIOUS];
    for (name, rule) in RULES {
        let cfg = Config {
            timeline: Timeline::HousePace,
            dice: rule,
            events: EventMode::Deck,
            pacts: Some(PactRules::default()),
            ..Config::default()
        };
        let s = run(cfg, &strats, games, 1, 2);
        let names = ["balanced", "balanced", "aggressive", "cautious"];
        let w: Vec<String> = names.iter().zip(&s.strategy_wins).skip(1).map(|(n, w)| format!("{n} {:.1}%", 100.0 * w)).collect();
        println!(
            "{name:<34} battles won {:.1}% | battles/game {:.0} | winner holds {:.1} terr | {}",
            100.0 * s.battle_win_rate,
            s.battles,
            s.winner_territories,
            w.join(", ")
        );
    }
}
