use risk_england_sim::bot::Strategy;
use risk_england_sim::combat::DiceRule;
use risk_england_sim::game::*;
use risk_england_sim::sim::{margin, run, Summary};

struct Args {
    experiment: String,
    games: u64,
    seed: u64,
    threads: usize,
    house: bool,
}

fn parse() -> Args {
    let mut a = Args {
        experiment: "all".into(),
        games: 20_000,
        seed: 1,
        threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        house: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "--games" => a.games = it.next().unwrap().parse().unwrap(),
            "--seed" => a.seed = it.next().unwrap().parse().unwrap(),
            "--threads" => a.threads = it.next().unwrap().parse().unwrap(),
            "--house" => a.house = true,
            "-h" | "--help" => {
                println!("usage: risk-england-sim [pace|dice|cards|powers|styles|all] [--games N] [--seed S] [--threads T] [--house]");
                println!("  --house  use the group's house rules (house pace + defender-sees dice) as the base config");
                std::process::exit(0);
            }
            e => a.experiment = e.to_string(),
        }
    }
    a
}

fn pct(x: f64) -> String {
    format!("{:5.1}%", x * 100.0)
}

fn header(title: &str, a: &Args, base: &Config) {
    println!("\n=== {title} ===");
    println!(
        "{} games per row, seed {}, base: timeline {:?}, dice {:?}, cards {:?}",
        a.games, a.seed, base.timeline, base.dice, base.card_draw
    );
}

fn game_line(label: &str, s: &Summary) {
    println!(
        "{label:<22} rounds {:5.1} | turns/player {:5.1} | ends: timeline {} conquest {} limit {} | winner holds {:4.1} terr | battles {:5.1}, won {}",
        s.rounds,
        s.turns_per_player,
        pct(s.by_timeline),
        pct(s.by_conquest),
        pct(s.by_limit),
        s.winner_territories,
        s.battles,
        pct(s.battle_win_rate)
    );
}

fn seats_line(s: &Summary) {
    let seats: Vec<String> = s.seat_wins.iter().enumerate().map(|(i, w)| format!("seat {}: {}", i + 1, pct(*w))).collect();
    println!("{:<22} {}", "  win share by seat", seats.join("  "));
}

fn strategy_table(strats: &[Strategy], s: &Summary) {
    // merge slots with the same name
    let mut names: Vec<&str> = Vec::new();
    for st in strats {
        if !names.contains(&st.name) {
            names.push(st.name);
        }
    }
    for name in names {
        let idx: Vec<usize> = (0..strats.len()).filter(|&i| strats[i].name == name).collect();
        let k = idx.len() as f64;
        let w: f64 = idx.iter().map(|&i| s.strategy_wins[i]).sum::<f64>() / k;
        let ct: f64 = idx.iter().map(|&i| s.card_troops[i]).sum::<f64>() / k;
        let mt: f64 = idx.iter().map(|&i| s.mission_troops[i]).sum::<f64>() / k;
        let gf: f64 = idx.iter().map(|&i| s.good_flips[i]).sum::<f64>() / k;
        let bf: f64 = idx.iter().map(|&i| s.bad_flips[i]).sum::<f64>() / k;
        println!(
            "  {name:<14} win {} ±{:.1} | troops from cards {:5.1} | from missions {:4.1} | flips good {:.2} bad {:.2}",
            pct(w),
            margin(w, s.games) * 100.0,
            ct,
            mt,
            gf,
            bf
        );
    }
    println!("  (fair share with {} players = {})", strats.len(), pct(1.0 / strats.len() as f64));
}

fn base(a: &Args) -> Config {
    if a.house {
        Config { timeline: Timeline::HousePace, dice: DiceRule::DefenderOneByOne, ..Config::default() }
    } else {
        Config::default()
    }
}

fn pace(a: &Args) {
    let b = base(a);
    header("Timeline pace (4 balanced bots)", a, &b);
    let strats = [Strategy::BALANCED; 4];
    for (label, tl) in [("official", Timeline::Official), ("house pace", Timeline::HousePace), ("no timeline", Timeline::Off)] {
        let s = run(Config { timeline: tl, ..b }, &strats, a.games, a.seed, a.threads);
        game_line(label, &s);
        seats_line(&s);
    }
}

fn dice(a: &Args) {
    let b = base(a);
    header("Dice rule (4 balanced bots)", a, &b);
    let strats = [Strategy::BALANCED; 4];
    for (label, d) in [("classic", DiceRule::Classic), ("defender sees", DiceRule::DefenderSees)] {
        let s = run(Config { dice: d, ..b }, &strats, a.games, a.seed, a.threads);
        game_line(label, &s);
        seats_line(&s);
    }
}

fn cards(a: &Args) {
    let b = base(a);
    header("Card trading: when to cash in", a, &b);
    let strats = [Strategy::TRADE_AT_3, Strategy::BALANCED, Strategy::TRADE_AT_8, Strategy::HOARDER];
    for (label, cd) in [("1 card per capture", CardDraw::PerCapture), ("max 1 card per turn", CardDraw::OnePerTurn)] {
        let s = run(Config { card_draw: cd, ..b }, &strats, a.games, a.seed, a.threads);
        println!("-- {label} (balanced trades at 6, hoarder at 10)");
        game_line("", &s);
        strategy_table(&strats, &s);
    }
}

fn powers(a: &Args) {
    let b = base(a);
    header("Power cards: flip or not", a, &b);
    let strats = [Strategy::FLIPPER, Strategy::NEVER_FLIP, Strategy::FLIPPER, Strategy::NEVER_FLIP];
    let s = run(b, &strats, a.games, a.seed, a.threads);
    game_line("", &s);
    strategy_table(&strats, &s);
}

fn styles(a: &Args) {
    let b = base(a);
    header("Play styles", a, &b);
    let strats = [Strategy::AGGRESSIVE, Strategy::BALANCED, Strategy::CAUTIOUS, Strategy::HOARDER];
    let s = run(b, &strats, a.games, a.seed, a.threads);
    game_line("", &s);
    strategy_table(&strats, &s);
}

fn main() {
    let a = parse();
    let t0 = std::time::Instant::now();
    match a.experiment.as_str() {
        "pace" => pace(&a),
        "dice" => dice(&a),
        "cards" => cards(&a),
        "powers" => powers(&a),
        "styles" => styles(&a),
        "all" => {
            pace(&a);
            dice(&a);
            cards(&a);
            powers(&a);
            styles(&a);
        }
        e => {
            eprintln!("unknown experiment `{e}`; try --help");
            std::process::exit(2);
        }
    }
    eprintln!("\ndone in {:.1}s on {} threads", t0.elapsed().as_secs_f64(), a.threads);
}
