//! Pact balance report.
//!   cargo run --release --example pact_report -- [games] [threads]
//! House rules + full century event deck, 4 players, seats rotated.
use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::DiceRule;
use risk_england_sim::events::EventMode;
use risk_england_sim::game::*;
use risk_england_sim::sim::margin;

#[derive(Default, Clone)]
struct Acc {
    games: f64,
    wins: Vec<f64>,
    signed: Vec<f64>,
    kept: Vec<f64>,
    betrayals: Vec<f64>,
    betrayed: Vec<f64>,
    pact_cards: Vec<f64>,
    rounds: f64,
    lead_changes: f64,
    comebacks: f64,
    margin: f64,
}

fn run(cfg: Config, strats: &[Strategy; 4], games: u64, threads: u64) -> Acc {
    let per = games.div_ceil(threads);
    let parts: Vec<Acc> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                sc.spawn(move || {
                    let sh = Shared::new();
                    let mut a = Acc { wins: vec![0.0; 4], signed: vec![0.0; 4], kept: vec![0.0; 4], betrayals: vec![0.0; 4], betrayed: vec![0.0; 4], pact_cards: vec![0.0; 4], ..Default::default() };
                    for g in t * per..((t + 1) * per).min(games) {
                        let rot = (g % 4) as usize;
                        let slot: Vec<usize> = (0..4).map(|s| (s + rot) % 4).collect();
                        let seat: Vec<Strategy> = slot.iter().map(|&i| strats[i]).collect();
                        let r = Game::new(cfg, &sh, &seat, g.wrapping_mul(0x9E37_79B9) + 5).play();
                        a.games += 1.0;
                        a.rounds += r.rounds as f64;
                        let share = 1.0 / r.winners.len() as f64;
                        for &w in &r.winners {
                            a.wins[slot[w]] += share;
                        }
                        for (s, ps) in r.players.iter().enumerate() {
                            let i = slot[s];
                            a.signed[i] += ps.pacts_signed as f64;
                            a.kept[i] += ps.pacts_kept as f64;
                            a.betrayals[i] += ps.betrayals as f64;
                            a.betrayed[i] += ps.betrayed as f64;
                            a.pact_cards[i] += ps.pact_cards as f64;
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
                        let mut tt = r.territories.clone();
                        tt.sort_unstable_by(|x, y| y.cmp(x));
                        a.margin += (tt[0] - tt[1]) as f64;
                    }
                    a
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut t = Acc { wins: vec![0.0; 4], signed: vec![0.0; 4], kept: vec![0.0; 4], betrayals: vec![0.0; 4], betrayed: vec![0.0; 4], pact_cards: vec![0.0; 4], ..Default::default() };
    for p in parts {
        t.games += p.games;
        t.rounds += p.rounds;
        t.lead_changes += p.lead_changes;
        t.comebacks += p.comebacks;
        t.margin += p.margin;
        for i in 0..4 {
            t.wins[i] += p.wins[i];
            t.signed[i] += p.signed[i];
            t.kept[i] += p.kept[i];
            t.betrayals[i] += p.betrayals[i];
            t.betrayed[i] += p.betrayed[i];
            t.pact_cards[i] += p.pact_cards[i];
        }
    }
    t
}

fn game_line(label: &str, a: &Acc) {
    let g = a.games;
    println!(
        "{label:<34} rounds {:4.1} | lead changes {:4.2} | comebacks {:4.1}% | margin {:4.1}",
        a.rounds / g,
        a.lead_changes / g,
        100.0 * a.comebacks / g,
        a.margin / g
    );
}

fn table(strats: &[Strategy; 4], a: &Acc) {
    let g = a.games;
    let mut names: Vec<&str> = Vec::new();
    for s in strats {
        if !names.contains(&s.name) {
            names.push(s.name);
        }
    }
    for name in names {
        let idx: Vec<usize> = (0..4).filter(|&i| strats[i].name == name).collect();
        let k = idx.len() as f64;
        let f = |v: &Vec<f64>| idx.iter().map(|&i| v[i]).sum::<f64>() / k / g;
        let w = f(&a.wins);
        println!(
            "    {name:<12} win {:5.1}% ±{:.1} | pacts signed {:4.2} kept {:4.2} | betrays {:4.2} betrayed {:4.2} | pact cards {:4.2}",
            100.0 * w,
            100.0 * margin(w, a.games as u64),
            f(&a.signed),
            f(&a.kept),
            f(&a.betrayals),
            f(&a.betrayed),
            f(&a.pact_cards)
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let games: u64 = args.next().map(|x| x.parse().unwrap()).unwrap_or(20_000);
    let threads: u64 = args.next().map(|x| x.parse().unwrap()).unwrap_or(2);
    let base = Config { timeline: Timeline::HousePace, dice: DiceRule::DefenderSees, events: EventMode::Deck, ..Config::default() };
    let rules = PactRules::default();
    let on = Config { pacts: Some(rules), ..base };
    println!("{games} games per row; house pace, defender-sees dice, full century deck; 4 players, seats rotated\n");

    println!("== Does diplomacy change the game?");
    let a = run(base, &[Strategy::NO_PACTS; 4], games, threads);
    game_line("no pacts", &a);
    let a = run(on, &[Strategy::LOYAL; 4], games, threads);
    game_line("pacts, everyone loyal", &a);
    table(&[Strategy::LOYAL; 4], &a);
    let a = run(on, &[Strategy::OPPORTUNIST; 4], games, threads);
    game_line("pacts, everyone opportunist", &a);
    table(&[Strategy::OPPORTUNIST; 4], &a);

    let mix = [Strategy::LOYAL, Strategy::OPPORTUNIST, Strategy::BACKSTABBER, Strategy::NO_PACTS];
    println!("\n== Mixed table: who wins? (fair share 25%)");
    let variants: Vec<(&str, PactRules)> = vec![
        ("balanced rules (default)", rules),
        ("first draft: victim takes all, cap 1", PactRules { on_betrayal: StakeOnBetrayal::VictimTakesAll, betray_card_cap: Some(1), ..rules }),
        ("no bonus cards", PactRules { bonus_per_turn: 0, ..rules }),
        ("bonus 2 per turn (cap 6)", PactRules { bonus_per_turn: 2, bonus_cap: 6, ..rules }),
        ("betrayer capped at 1 card", PactRules { betray_card_cap: Some(1), ..rules }),
        ("no oathbreaker", PactRules { oathbreaker: false, ..rules }),
        ("victim takes both stakes", PactRules { on_betrayal: StakeOnBetrayal::VictimTakesAll, ..rules }),
        ("both stakes discarded", PactRules { on_betrayal: StakeOnBetrayal::AllDiscarded, ..rules }),
        ("bigger stakes 2/3", PactRules { stake_short: 2, stake_long: 3, ..rules }),
    ];
    for (label, r) in variants {
        let a = run(Config { pacts: Some(r), ..base }, &mix, games, threads);
        game_line(label, &a);
        table(&mix, &a);
    }

    println!("\n== Pact length: short (2 turns) vs long (4 turns)");
    let lens = [Strategy::LOYAL, Strategy::LOYAL_LONG, Strategy::LOYAL, Strategy::LOYAL_LONG];
    for (label, r) in [("bonus = length - 1 (cap 3)", rules), ("bonus = 2 x (length - 1) (cap 6)", PactRules { bonus_per_turn: 2, bonus_cap: 6, ..rules })] {
        let a = run(Config { pacts: Some(r), ..base }, &lens, games, threads);
        game_line(label, &a);
        table(&lens, &a);
    }
}
