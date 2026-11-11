//! Monte Carlo runner: plays many games in parallel and aggregates results.
//! Strategies rotate through the seats so seat order does not bias them.

use crate::bot::Strategy;
use crate::game::*;

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub games: u64,
    /// Win share per strategy slot (ties split evenly).
    pub strategy_wins: Vec<f64>,
    /// Win share per seat (turn order).
    pub seat_wins: Vec<f64>,
    pub rounds: f64,
    pub turns: f64,
    pub turns_per_player: f64,
    pub by_timeline: f64,
    pub by_conquest: f64,
    pub by_limit: f64,
    pub battles: f64,
    pub battle_win_rate: f64,
    pub winner_territories: f64,
    pub card_troops: Vec<f64>,
    pub mission_troops: Vec<f64>,
    pub good_flips: Vec<f64>,
    pub bad_flips: Vec<f64>,
    pub ties: f64,
}

fn run_chunk(cfg: Config, strats: &[Strategy], first: u64, count: u64, seed: u64) -> Summary {
    let sh = Shared::new();
    let n = strats.len();
    let mut s = Summary {
        strategy_wins: vec![0.0; n],
        seat_wins: vec![0.0; n],
        card_troops: vec![0.0; n],
        mission_troops: vec![0.0; n],
        good_flips: vec![0.0; n],
        bad_flips: vec![0.0; n],
        ..Default::default()
    };
    let mut won_battles = 0u64;
    let mut all_battles = 0u64;
    for g in first..first + count {
        let rot = (g % n as u64) as usize;
        // seat -> strategy slot
        let slot: Vec<usize> = (0..n).map(|seat| (seat + rot) % n).collect();
        let seat_strats: Vec<Strategy> = slot.iter().map(|&i| strats[i]).collect();
        let game_seed = seed ^ g.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let r = Game::new(cfg, &sh, &seat_strats, game_seed).play();
        s.games += 1;
        let share = 1.0 / r.winners.len() as f64;
        if r.winners.len() > 1 {
            s.ties += 1.0;
        }
        for &w in &r.winners {
            s.seat_wins[w] += share;
            s.strategy_wins[slot[w]] += share;
        }
        s.rounds += r.rounds as f64;
        s.turns += r.turns as f64;
        s.turns_per_player += r.turns as f64 / n as f64;
        match r.ended_by {
            EndReason::Timeline => s.by_timeline += 1.0,
            EndReason::Conquest => s.by_conquest += 1.0,
            EndReason::RoundLimit => s.by_limit += 1.0,
        }
        s.battles += r.battles as f64;
        all_battles += r.battles as u64;
        won_battles += r.battles_won as u64;
        s.winner_territories += *r.territories.iter().max().unwrap() as f64;
        for (seat, ps) in r.players.iter().enumerate() {
            let i = slot[seat];
            s.card_troops[i] += ps.card_troops as f64;
            s.mission_troops[i] += ps.mission_troops as f64;
            s.good_flips[i] += ps.good_flips as f64;
            s.bad_flips[i] += ps.bad_flips as f64;
        }
    }
    s.battle_win_rate = won_battles as f64; // raw count, normalised in merge
    s.battles = all_battles as f64;
    s
}

pub fn run(cfg: Config, strats: &[Strategy], games: u64, seed: u64, threads: usize) -> Summary {
    let threads = threads.max(1) as u64;
    let per = games.div_ceil(threads);
    let parts: Vec<Summary> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .filter_map(|t| {
                let first = t * per;
                if first >= games {
                    return None;
                }
                let count = per.min(games - first);
                Some(scope.spawn(move || run_chunk(cfg, strats, first, count, seed)))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let n = strats.len();
    let mut total = Summary {
        strategy_wins: vec![0.0; n],
        seat_wins: vec![0.0; n],
        card_troops: vec![0.0; n],
        mission_troops: vec![0.0; n],
        good_flips: vec![0.0; n],
        bad_flips: vec![0.0; n],
        ..Default::default()
    };
    let mut won = 0.0;
    for p in parts {
        total.games += p.games;
        for i in 0..n {
            total.strategy_wins[i] += p.strategy_wins[i];
            total.seat_wins[i] += p.seat_wins[i];
            total.card_troops[i] += p.card_troops[i];
            total.mission_troops[i] += p.mission_troops[i];
            total.good_flips[i] += p.good_flips[i];
            total.bad_flips[i] += p.bad_flips[i];
        }
        total.rounds += p.rounds;
        total.turns += p.turns;
        total.turns_per_player += p.turns_per_player;
        total.by_timeline += p.by_timeline;
        total.by_conquest += p.by_conquest;
        total.by_limit += p.by_limit;
        total.battles += p.battles;
        won += p.battle_win_rate;
        total.winner_territories += p.winner_territories;
        total.ties += p.ties;
    }
    let g = total.games as f64;
    total.battle_win_rate = if total.battles > 0.0 { won / total.battles } else { 0.0 };
    for v in [&mut total.rounds, &mut total.turns, &mut total.turns_per_player, &mut total.battles, &mut total.winner_territories] {
        *v /= g;
    }
    for v in [&mut total.by_timeline, &mut total.by_conquest, &mut total.by_limit, &mut total.ties] {
        *v /= g;
    }
    // Each strategy slot sits in every seat equally often, so it plays every game.
    for i in 0..n {
        total.strategy_wins[i] /= g;
        total.seat_wins[i] /= g;
        total.card_troops[i] /= g;
        total.mission_troops[i] /= g;
        total.good_flips[i] /= g;
        total.bad_flips[i] /= g;
    }
    total
}

/// 95% margin of error for a win share p over `games` games.
pub fn margin(p: f64, games: u64) -> f64 {
    1.96 * (p * (1.0 - p) / games as f64).sqrt()
}
