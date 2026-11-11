//! Typical board state when each century begins, under the house rules.
//! Used to calibrate century-event thresholds:
//!   cargo run --release --example stacks_by_century -- [players] [games]
use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::DiceRule;
use risk_england_sim::game::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let players: usize = args.next().map(|x| x.parse().unwrap()).unwrap_or(4);
    let games: u64 = args.next().map(|x| x.parse().unwrap()).unwrap_or(20000);
    let pool = [Strategy::BALANCED, Strategy::BALANCED, Strategy::AGGRESSIVE, Strategy::HOARDER, Strategy::BALANCED];
    let strats: Vec<Strategy> = pool[..players].to_vec();
    let cfg = Config { timeline: Timeline::HousePace, dice: DiceRule::DefenderOneByOne,
        terrain: TerrainCombat { mountain_climb_cost: 1, river_cross_cost: 1, ..TerrainCombat::default() }, ..Config::default() };
    let sh = Shared::new();
    // per century: count, sum of (largest, 3rd largest, 10th largest, median, leader terr, leader troops, hand stars), share of games reaching it
    let mut acc = vec![[0f64; 8]; 11];
    for g in 0..games {
        let r = Game::new(cfg, &sh, &strats, g * 7919 + 1).play();
        let mut seen = [false; 11];
        for s in &r.snapshots {
            let c = s.century.min(10);
            if seen[c] {
                continue; // first round of each century only
            }
            seen[c] = true;
            let a = &mut acc[c];
            a[0] += 1.0;
            a[1] += s.stacks[0] as f64;
            a[2] += s.stacks[2] as f64;
            a[3] += s.stacks[9] as f64;
            a[4] += s.stacks[21] as f64;
            a[5] += s.leader_territories as f64;
            a[6] += s.leader_troops as f64;
            a[7] += s.hand_stars;
        }
    }
    println!("{players} players, {games} games, house rules. Averages when a century begins:");
    println!("century  reached  largest  3rd  10th  median  leader-terr  leader-troops  hand-stars");
    for (c, a) in acc.iter().enumerate() {
        if a[0] == 0.0 {
            continue;
        }
        let k = a[0];
        println!(
            "{:>4}th  {:>6.0}%  {:>7.1}  {:>4.1}  {:>4.1}  {:>6.1}  {:>11.1}  {:>13.1}  {:>10.1}",
            c + 5,
            100.0 * k / games as f64,
            a[1] / k, a[2] / k, a[3] / k, a[4] / k, a[5] / k, a[6] / k, a[7] / k
        );
    }
}
