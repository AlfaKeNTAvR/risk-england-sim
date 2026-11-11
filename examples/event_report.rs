//! Engagement report for century events.
//!   cargo run --release --example event_report -- [games] [threads]
//! Compares no events, the full deck, and each event forced on its own,
//! using human-like bots (rear guard of 2) and the group's house rules.
use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::DiceRule;
use risk_england_sim::events::{EventMode, ALL};
use risk_england_sim::game::*;

const STRATS: [Strategy; 4] = [Strategy::BALANCED, Strategy::BALANCED, Strategy::AGGRESSIVE, Strategy::HOARDER];

#[derive(Default, Clone)]
struct Acc {
    games: f64,
    rounds: f64,
    lead_changes: f64,
    comebacks: f64,
    margin: f64,
    seat_wins: [f64; 4],
    // per space: [landings, toppled, leader wins, leader share change, troops lost at once, largest stack]
    space: [[f64; 6]; 10],
    // for forced events: same, only where the event fired
    fired: [f64; 6],
}

fn add(a: &mut [f64; 6], b: &[f64; 6]) {
    for i in 0..6 {
        a[i] += b[i];
    }
}

fn run(mode: EventMode, games: u64, threads: u64) -> Acc {
    let defect: u32 = std::env::var("DEFECT").ok().and_then(|v| v.parse().ok()).unwrap_or(Config::default().defect_troops);
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<usize>().ok());
    let cfg = Config {
        timeline: Timeline::HousePace,
        dice: DiceRule::DefenderOneByOne,
        events: mode,
        defect_troops: defect,
        revolt_count: env("REVOLT").unwrap_or(Config::default().revolt_count),
        famine_half: env("FAMINE_HALF").map_or(Config::default().famine_half, |v| v == 1),
        ..Config::default()
    };
    let per = games.div_ceil(threads);
    let parts: Vec<Acc> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                sc.spawn(move || {
                    let sh = Shared::new();
                    let mut a = Acc::default();
                    for g in t * per..((t + 1) * per).min(games) {
                        let rot = (g % 4) as usize;
                        let strats: Vec<Strategy> = (0..4).map(|s| STRATS[(s + rot) % 4]).collect();
                        let r = Game::new(cfg, &sh, &strats, g.wrapping_mul(0x9E37_79B9) + 11).play();
                        a.games += 1.0;
                        a.rounds += r.rounds as f64;
                        let share = 1.0 / r.winners.len() as f64;
                        for &w in &r.winners {
                            a.seat_wins[w] += share;
                        }
                        let mut leaders: Vec<usize> = r.snapshots.iter().map(|s| s.leader).collect();
                        if r.winners.len() == 1 {
                            leaders.push(r.winners[0]);
                        }
                        a.lead_changes += leaders.windows(2).filter(|w| w[0] != w[1]).count() as f64;
                        let mid = r.snapshots[r.snapshots.len() / 2].leader;
                        if r.winners.len() == 1 && r.winners[0] != mid {
                            a.comebacks += 1.0;
                        }
                        let mut t = r.territories.clone();
                        t.sort_unstable_by(|x, y| y.cmp(x));
                        a.margin += (t[0] - t[1]) as f64;
                        for (i, c) in r.centuries.iter().enumerate() {
                            if c.space >= 10 {
                                continue;
                            }
                            let next = r.centuries.get(i + 1).map(|n| &n.territories).unwrap_or(&r.territories);
                            let next_leader = (0..4).max_by_key(|&p| (next[p], std::cmp::Reverse(p))).unwrap();
                            let toppled = (next_leader != c.leader) as u8 as f64;
                            let wins = if r.winners.contains(&c.leader) { share } else { 0.0 };
                            let d_share = next[c.leader] as f64 / 42.0 - c.leader_share;
                            let row = [1.0, toppled, wins, d_share, c.troops_lost_at_once as f64, c.largest_stack as f64];
                            add(&mut a.space[c.space], &row);
                            if c.event.is_some() {
                                if let EventMode::Force(_) = mode {
                                    add(&mut a.fired, &row);
                                }
                            }
                        }
                    }
                    a
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut tot = Acc::default();
    for p in parts {
        tot.games += p.games;
        tot.rounds += p.rounds;
        tot.lead_changes += p.lead_changes;
        tot.comebacks += p.comebacks;
        tot.margin += p.margin;
        for i in 0..4 {
            tot.seat_wins[i] += p.seat_wins[i];
        }
        for s in 0..10 {
            add(&mut tot.space[s], &p.space[s]);
        }
        add(&mut tot.fired, &p.fired);
    }
    tot
}

fn game_line(label: &str, a: &Acc) {
    let g = a.games;
    let seats: Vec<String> = a.seat_wins.iter().map(|w| format!("{:.0}", 100.0 * w / g)).collect();
    println!(
        "{label:<22} rounds {:4.1} | lead changes {:4.2} | comebacks {:4.1}% | final margin {:4.1} terr | seats {}%",
        a.rounds / g,
        a.lead_changes / g,
        100.0 * a.comebacks / g,
        a.margin / g,
        seats.join("/")
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let games: u64 = args.next().map(|x| x.parse().unwrap()).unwrap_or(20_000);
    let threads: u64 = args.next().map(|x| x.parse().unwrap()).unwrap_or(2);
    let only: Option<String> = args.next().map(|x| x.to_lowercase());
    println!("{games} games per row, 4 players (balanced x2, aggressive, hoarder), house pace + defender-sees dice\n");

    let base = run(EventMode::Off, games, threads);
    let deck = run(EventMode::Deck, games, threads);
    println!("== Whole game");
    game_line("no events", &base);
    game_line("full event deck", &deck);

    println!("\n== Baseline by century (no events): what a typical century looks like");
    println!("century  landed  largest stack  leader toppled  leader goes on to win");
    for s in 0..10 {
        let r = &base.space[s];
        if r[0] == 0.0 {
            continue;
        }
        println!(
            "{:>4}th   {:>5.0}%  {:>13.1}  {:>13.1}%  {:>20.1}%",
            s + 5,
            100.0 * r[0] / games as f64,
            r[5] / r[0],
            100.0 * r[1] / r[0],
            100.0 * r[2] / r[0]
        );
    }

    println!("\n== Each event on its own (vs the same century with no event)");
    println!("{:<22} cent  toppled (base)   leader wins (base)  leader share  lost at once | game: lead chg  comebacks  margin", "event");
    for e in ALL {
        if only.as_ref().is_some_and(|o| !e.name().to_lowercase().contains(o.as_str())) {
            continue;
        }
        let a = run(EventMode::Force(e), games, threads);
        let f = &a.fired;
        let b = &base.space[e.space()];
        if f[0] == 0.0 {
            continue;
        }
        let g = a.games;
        println!(
            "{:<22} {:>3}th  {:>5.1}% ({:>4.1}%)    {:>5.1}% ({:>4.1}%)      {:>+6.2} pp   {:>9.1}   |   {:>+6.2}     {:>+5.1} pp   {:>+5.1}{}",
            e.name(),
            e.space() + 5,
            100.0 * f[1] / f[0],
            100.0 * b[1] / b[0],
            100.0 * f[2] / f[0],
            100.0 * b[2] / b[0],
            100.0 * (f[3] / f[0] - b[3] / b[0]),
            f[4] / f[0],
            a.lead_changes / g - base.lead_changes / base.games,
            100.0 * (a.comebacks / g - base.comebacks / base.games),
            a.margin / g - base.margin / base.games,
            if e.simulated() { "" } else { "  (pacts not simulated)" }
        );
    }
}
