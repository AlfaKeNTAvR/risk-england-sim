//! Game engine: one full game of Risk: The Battle for Medieval England,
//! 3-5 players, with rulebook rules plus house-rule switches in `Config`.

use crate::board::*;
use crate::bot::*;
use crate::cards::*;
use crate::combat::*;
use crate::events::*;
use crate::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Timeline {
    /// Rulebook: the golden cavalry moves 1 space after every player's turn.
    /// Leaving a gated space needs the right even/odd roll (50%).
    Official,
    /// House rule: 1 space after every full round, plus each player rolls a
    /// d6 after their turn and a 6 moves it 1 extra space. No gates.
    HousePace,
    /// Rulebook variation: no timeline, play until one army holds everything
    /// (capped by `max_rounds`).
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardDraw {
    /// Rulebook: draw 1 territory card for every territory conquered.
    PerCapture,
    /// Classic Risk: at most 1 card per turn.
    OnePerTurn,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub timeline: Timeline,
    pub dice: DiceRule,
    pub card_draw: CardDraw,
    pub powers: bool,
    pub missions: bool,
    pub max_rounds: u32,
    /// House rule: century events.
    pub events: EventMode,
}

impl Default for Config {
    /// The rulebook as printed.
    fn default() -> Self {
        Config {
            timeline: Timeline::Official,
            dice: DiceRule::Classic,
            card_draw: CardDraw::PerCapture,
            powers: true,
            missions: true,
            max_rounds: 300,
            events: EventMode::Off,
        }
    }
}

/// Timeline spaces 0..=10 = 5th..15th century. A dice gate sits between
/// 7th-8th, 8th-9th, 10th-11th and 13th-14th: leaving these spaces needs a roll.
const GATED_SPACES: [usize; 4] = [2, 3, 5, 8];
pub const LAST_SPACE: usize = 10;

/// Data shared read-only by every game on a thread.
pub struct Shared {
    pub graph: Graph,
    pub table: DefenderTable,
    pub jewel_tower: usize,
    pub mission_ids: Vec<Vec<usize>>, // territory ids for Hold missions
    /// Ends of sea routes.
    pub ports: [bool; N_TERR],
}

impl Shared {
    pub fn new() -> Self {
        let mission_ids = MISSIONS
            .iter()
            .map(|m| match m.goal {
                MissionGoal::Hold(names) => names.iter().map(|n| id(n)).collect(),
                _ => Vec::new(),
            })
            .collect();
        let graph = Graph::build();
        let mut ports = [false; N_TERR];
        for &(a, b) in &graph.sea_routes {
            ports[a] = true;
            ports[b] = true;
        }
        Shared { graph, table: DefenderTable::build(), jewel_tower: id("Jewel Tower"), mission_ids, ports }
    }
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndReason {
    Timeline,
    Conquest,
    RoundLimit,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerStats {
    pub card_troops: u32,
    pub trades: u32,
    pub mission_troops: u32,
    pub good_flips: u32,
    pub bad_flips: u32,
    pub captures: u32,
}

#[derive(Clone, Debug)]
pub struct GameResult {
    /// Seats with the most territories at the end (more than one on a tie).
    pub winners: Vec<usize>,
    pub territories: Vec<usize>,
    pub rounds: u32,
    pub turns: u32,
    pub ended_by: EndReason,
    pub battles: u32,
    pub battles_won: u32,
    pub players: Vec<PlayerStats>,
    /// Board state at the start of every round, for calibrating events.
    pub snapshots: Vec<Snapshot>,
    /// Every century the cavalry landed on, in order.
    pub centuries: Vec<CenturyLog>,
}

/// Board state at the start of a round.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Timeline space (0 = 5th century ... 10 = 15th).
    pub century: usize,
    pub round: u32,
    /// All stacks, sorted descending.
    pub stacks: Vec<u32>,
    /// Territories held by the leading player.
    pub leader_territories: usize,
    /// Total troops of the largest army.
    pub leader_troops: u32,
    /// Average stars in hand per living player.
    pub hand_stars: f64,
    /// Player with the most territories (lowest seat on a tie).
    pub leader: usize,
    pub territories: Vec<usize>,
}

/// A century the cavalry landed on, and what happened there.
#[derive(Clone, Debug)]
pub struct CenturyLog {
    pub space: usize,
    pub event: Option<Ev>,
    pub round: u32,
    /// Leader (most territories) when the century began.
    pub leader: usize,
    /// Leader's share of all territories when the century began.
    pub leader_share: f64,
    /// Troops removed by the event's immediate effect.
    pub troops_lost_at_once: u32,
    /// Territories per player when the century began (after the event).
    pub territories: Vec<usize>,
    /// Largest stack when the century began (before the event).
    pub largest_stack: u32,
}

struct Player {
    strat: Strategy,
    hand: Vec<usize>,
    face_down: Vec<Power>,
    kept: Vec<Power>,
    missions: Vec<usize>,
    pending: u32,
    alive: bool,
    stats: PlayerStats,
}

pub struct Game<'a> {
    cfg: Config,
    sh: &'a Shared,
    rng: Rng,
    owner: [usize; N_TERR],
    troops: [u32; N_TERR],
    players: Vec<Player>,
    deck: Vec<usize>,
    discard: Vec<usize>,
    cavalry: usize,
    jt_holder: usize,
    jt_turns: u32,
    battles: u32,
    battles_won: u32,
    event: Option<Ev>,
    round: u32,
    centuries: Vec<CenturyLog>,
}

impl<'a> Game<'a> {
    pub fn new(cfg: Config, sh: &'a Shared, strategies: &[Strategy], seed: u64) -> Self {
        let n = strategies.len();
        assert!((3..=5).contains(&n), "3-5 players supported");
        let mut rng = Rng::new(seed);
        let mut players: Vec<Player> = strategies
            .iter()
            .map(|&s| Player {
                strat: s,
                hand: Vec::new(),
                face_down: Vec::new(),
                kept: Vec::new(),
                missions: Vec::new(),
                pending: 0,
                alive: true,
                stats: PlayerStats::default(),
            })
            .collect();

        // Deal all 42 territory cards; with an uneven split the last seats get the extra cards.
        let mut cards: Vec<usize> = (0..N_TERR).collect();
        rng.shuffle(&mut cards);
        let base = N_TERR / n;
        let extra = N_TERR % n;
        let mut owner = [0usize; N_TERR];
        let mut troops = [0u32; N_TERR];
        let mut it = cards.into_iter();
        for (p, _) in players.iter().enumerate() {
            let count = base + usize::from(p >= n - extra);
            for c in it.by_ref().take(count) {
                owner[c] = p;
                troops[c] = TERRITORIES[c].stars as u32;
            }
        }
        let mut deck: Vec<usize> = (0..N_TERR).collect();
        rng.shuffle(&mut deck);

        if cfg.missions {
            let mut m: Vec<usize> = (0..MISSIONS.len()).collect();
            rng.shuffle(&mut m);
            for (p, pl) in players.iter_mut().enumerate() {
                pl.missions = m[p * 2..p * 2 + 2].to_vec();
            }
        }
        if cfg.powers {
            let mut d = POWER_DECK.to_vec();
            rng.shuffle(&mut d);
            for (p, pl) in players.iter_mut().enumerate() {
                pl.face_down = d[p * 2..p * 2 + 2].to_vec();
            }
        }
        let jt = sh.jewel_tower;
        Game {
            cfg,
            sh,
            rng,
            owner,
            troops,
            players,
            deck,
            discard: Vec::new(),
            cavalry: 0,
            jt_holder: owner[jt],
            jt_turns: 0,
            battles: 0,
            battles_won: 0,
            event: None,
            round: 0,
            centuries: Vec::new(),
        }
    }

    fn ev(&self, e: Ev) -> bool {
        self.event == Some(e)
    }

    fn leader(&self) -> usize {
        let n = self.players.len();
        (0..n).max_by_key(|&p| (self.count(p), std::cmp::Reverse(p))).unwrap()
    }

    fn weakest(&self) -> Option<usize> {
        let n = self.players.len();
        (0..n).filter(|&p| self.players[p].alive).min_by_key(|&p| (self.count(p), p))
    }

    /// Lose troops on one territory, never below 1. Returns troops removed.
    fn shrink(&mut self, t: usize, to: u32) -> u32 {
        let to = to.max(1);
        if self.troops[t] > to {
            let lost = self.troops[t] - to;
            self.troops[t] = to;
            lost
        } else {
            0
        }
    }

    /// The cavalry has landed on a new century: roll its event and apply any
    /// immediate effect.
    #[allow(clippy::needless_range_loop)] // indexes several per-territory arrays
    fn begin_century(&mut self) {
        let space = self.cavalry;
        self.event = match self.cfg.events {
            EventMode::Off => None,
            EventMode::Deck => {
                let slot = (self.rng.d6() as usize - 1) / 2;
                Ev::of_space(space, slot)
            }
            EventMode::Force(e) => (e.space() == space).then_some(e),
        };
        let leader = self.leader();
        let leader_share = self.count(leader) as f64 / N_TERR as f64;
        let largest_stack = *self.troops.iter().max().unwrap();
        let mut lost = 0;
        if let Some(e) = self.event {
            for t in 0..N_TERR {
                let x = self.troops[t];
                lost += match e {
                    Ev::PlagueOfJustinian if x >= th::PLAGUE_6TH => self.shrink(t, x - 1),
                    Ev::VikingRaids if self.sh.ports[t] => self.shrink(t, x.saturating_sub(2)),
                    Ev::HardWinter if x >= th::WINTER_10TH => self.shrink(t, x - 2),
                    Ev::HarryingOfTheNorth if TERRITORIES[t].region == Region::North => self.shrink(t, x - x / 2),
                    Ev::CampFever if x >= th::FEVER_13TH => self.shrink(t, x - x / 3),
                    Ev::BlackDeath => self.shrink(t, x - x / 3),
                    _ => 0,
                };
            }
        }
        let territories = (0..self.players.len()).map(|p| self.count(p)).collect();
        self.centuries.push(CenturyLog {
            space,
            event: self.event,
            round: self.round,
            leader,
            leader_share,
            troops_lost_at_once: lost,
            territories,
            largest_stack,
        });
    }

    fn count(&self, p: usize) -> usize {
        self.owner.iter().filter(|&&o| o == p).count()
    }

    fn owns_region(&self, p: usize, r: usize) -> bool {
        self.sh.graph.region_members[r].iter().all(|&t| self.owner[t] == p)
    }

    fn enemy_neighbours(&self, t: usize) -> impl Iterator<Item = usize> + '_ {
        let o = self.owner[t];
        self.sh.graph.adj[t].iter().map(|&(n, _)| n).filter(move |&n| self.owner[n] != o)
    }

    fn is_frontier(&self, t: usize) -> bool {
        self.enemy_neighbours(t).next().is_some()
    }

    fn threat(&self, t: usize) -> f64 {
        let enemy: u32 = self.enemy_neighbours(t).map(|n| self.troops[n].saturating_sub(1)).sum();
        enemy as f64 - self.troops[t] as f64
    }

    /// How much player p wants enemy territory `t`.
    fn target_value(&self, p: usize, t: usize) -> f64 {
        let r = TERRITORIES[t].region.idx();
        let members = &self.sh.graph.region_members[r];
        let mine = members.iter().filter(|&&m| self.owner[m] == p).count() as f64;
        let mut v = 1.0 + 3.0 * mine / members.len() as f64;
        if mine as usize == members.len() - 1 {
            v += 2.0; // completes a region
        }
        if self.owns_region(self.owner[t], r) {
            v += 1.5; // breaks an enemy region bonus
        }
        let pl = &self.players[p];
        if pl.missions.iter().any(|&m| self.sh.mission_ids[m].contains(&t)) {
            v += 0.5;
        }
        v
    }

    /// Best (src, dst, score) attack for player p, ignoring the ratio threshold.
    fn best_attack(&self, p: usize) -> Option<(usize, usize, f64, f64)> {
        let mut best: Option<(usize, usize, f64, f64)> = None;
        for src in 0..N_TERR {
            if self.owner[src] != p || self.troops[src] < 2 {
                continue;
            }
            for &(dst, kind) in &self.sh.graph.adj[src] {
                if self.owner[dst] == p || (kind == EdgeKind::Sea && self.ev(Ev::VikingRaids)) {
                    continue;
                }
                let ratio = (self.troops[src] - 1) as f64 / self.troops[dst] as f64;
                let score = ratio * self.target_value(p, dst);
                if best.is_none_or(|b| score > b.2) {
                    best = Some((src, dst, score, ratio));
                }
            }
        }
        best
    }

    // ---------------------------------------------------------------- cards

    fn draw_card(&mut self, p: usize) {
        if self.deck.is_empty() {
            if self.discard.is_empty() {
                return;
            }
            std::mem::swap(&mut self.deck, &mut self.discard);
            self.rng.shuffle(&mut self.deck);
        }
        let c = self.deck.pop().unwrap();
        self.players[p].hand.push(c);
    }

    fn hand_stars(&self, p: usize) -> u32 {
        self.players[p].hand.iter().map(|&c| TERRITORIES[c].stars as u32).sum()
    }

    /// Trade cards according to the player's policy. Returns troops gained.
    fn trade_cards(&mut self, p: usize) -> u32 {
        let TradePolicy::AtStars(threshold) = self.players[p].strat.trade;
        let mut gained = 0;
        if self.ev(Ev::MagnaCarta) {
            return 0;
        }
        while self.hand_stars(p) >= threshold.max(2) {
            let hand = &self.players[p].hand;
            let ones = hand.iter().filter(|&&c| TERRITORIES[c].stars == 1).count() as u32;
            let twos = hand.len() as u32 - ones;
            let y = twos.min(5);
            let x = ones.min(10 - 2 * y);
            let stars = x + 2 * y;
            if stars < 2 {
                break;
            }
            let (mut need1, mut need2) = (x, y);
            let mut keep = Vec::new();
            for c in std::mem::take(&mut self.players[p].hand) {
                let s = TERRITORIES[c].stars;
                if s == 1 && need1 > 0 {
                    need1 -= 1;
                    self.discard.push(c);
                } else if s == 2 && need2 > 0 {
                    need2 -= 1;
                    self.discard.push(c);
                } else {
                    keep.push(c);
                }
            }
            self.players[p].hand = keep;
            let t = troops_for_stars(stars);
            gained += t;
            self.players[p].stats.trades += 1;
        }
        self.players[p].stats.card_troops += gained;
        gained
    }

    // --------------------------------------------------------------- powers

    fn wants_flip(&self, p: usize) -> bool {
        if !self.cfg.powers || self.players[p].face_down.is_empty() {
            return false;
        }
        match self.players[p].strat.flip {
            FlipPolicy::Never => false,
            FlipPolicy::Always => true,
            FlipPolicy::WhenBehind => {
                let alive = self.players.iter().filter(|pl| pl.alive).count();
                (self.count(p) as f64) < N_TERR as f64 / alive as f64
            }
        }
    }

    /// Fixed loss on one of p's territories. House rule: never below 1; pick a
    /// territory that can take the full loss if one exists (bot prefers the
    /// least exposed), otherwise the largest.
    fn lose_fixed(&mut self, p: usize, loss: u32) {
        let mut best: Option<(usize, (bool, bool, u32))> = None;
        for t in 0..N_TERR {
            if self.owner[t] != p {
                continue;
            }
            let key = (self.troops[t] > loss, !self.is_frontier(t), self.troops[t]);
            if best.is_none_or(|b| key > b.1) {
                best = Some((t, key));
            }
        }
        if let Some((t, _)) = best {
            self.troops[t] = self.troops[t].saturating_sub(loss).max(1);
        }
    }

    /// Flip one Power card during placement. Returns extra troops to place.
    fn flip_power(&mut self, p: usize) -> u32 {
        let card = self.players[p].face_down.pop().unwrap();
        let good = matches!(card, Power::Reinforcements) || card.is_keepable();
        if good {
            self.players[p].stats.good_flips += 1;
        } else {
            self.players[p].stats.bad_flips += 1;
        }
        if card.is_keepable() {
            self.players[p].kept.push(card);
            return 0;
        }
        match card {
            Power::Reinforcements => return 5,
            Power::Snowfall => {
                for &t in &self.sh.graph.region_members[Region::North.idx()] {
                    if self.owner[t] == p {
                        self.troops[t] = self.troops[t].saturating_sub(3).max(1);
                    }
                }
            }
            Power::Plague | Power::Starvation => {
                // Player chooses which territory: the bot loses as little as possible.
                let t = (0..N_TERR)
                    .filter(|&t| self.owner[t] == p && self.troops[t] > 1)
                    .min_by_key(|&t| self.troops[t]);
                if let Some(t) = t {
                    self.troops[t] = 1;
                }
            }
            Power::TiredTroops => self.lose_fixed(p, 3),
            Power::TroopsDefect => self.lose_fixed(p, 2),
            _ => unreachable!(),
        }
        0
    }

    fn take_kept(&mut self, p: usize, card: Power) -> bool {
        if let Some(i) = self.players[p].kept.iter().position(|&c| c == card) {
            self.players[p].kept.remove(i);
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------ placement

    fn place(&mut self, p: usize, n: u32) {
        if n == 0 {
            return;
        }
        let spear = self.best_attack(p).map(|b| b.0);
        let weak = (0..N_TERR)
            .filter(|&t| self.owner[t] == p && self.is_frontier(t))
            .max_by(|&a, &b| self.threat(a).partial_cmp(&self.threat(b)).unwrap());
        let spear = match spear.or(weak) {
            Some(s) => s,
            None => return,
        };
        // Top up thin frontier territories first (at most a third of new troops).
        let min = self.players[p].strat.frontier_min;
        let mut budget = n / 3;
        let mut n = n;
        if min > 0 {
            let mut thin: Vec<usize> = (0..N_TERR)
                .filter(|&t| self.owner[t] == p && self.troops[t] < min && self.is_frontier(t))
                .collect();
            thin.sort_by(|&a, &b| self.threat(b).partial_cmp(&self.threat(a)).unwrap());
            for t in thin {
                let add = (min - self.troops[t]).min(budget);
                self.troops[t] += add;
                budget -= add;
                n -= add;
                if budget == 0 {
                    break;
                }
            }
        }
        let share = self.players[p].strat.spearhead_share;
        let on_spear = ((n as f64) * share).round() as u32;
        self.troops[spear] += on_spear;
        let rest = n - on_spear;
        let def = weak.unwrap_or(spear);
        self.troops[def] += rest;
    }

    // --------------------------------------------------------------- attack

    fn eliminate(&mut self, victim: usize, by: usize) {
        self.players[victim].alive = false;
        let cards = std::mem::take(&mut self.players[victim].hand);
        self.players[by].hand.extend(cards);
        self.players[victim].face_down.clear();
        self.players[victim].kept.clear();
        self.players[victim].missions.clear();
    }

    /// Runs the attack phase. Returns the number of territories conquered.
    fn attack_phase(&mut self, p: usize) -> u32 {
        let strat = self.players[p].strat;
        let mut captures = 0;
        for _ in 0..200 {
            let Some((src, dst, _, ratio)) = self.best_attack(p) else { break };
            if ratio < strat.attack_ratio {
                break;
            }
            let defender = self.owner[dst];

            // Attacker: Artillery on a strong target before attacking.
            if self.cfg.powers && self.troops[dst] >= 5 && self.take_kept(p, Power::Artillery) {
                self.troops[dst] = 1;
            }

            // Commit troops: keep a guard back if src borders other enemies.
            let other_enemies = self.enemy_neighbours(src).any(|n| n != dst);
            let guard = if other_enemies { strat.rear_guard.max(1).max(self.troops[src] / 4) } else { 1 };
            if self.troops[src] <= guard {
                break;
            }
            let mut attackers = self.troops[src] - guard;
            self.troops[src] -= attackers;
            let mut defenders = self.troops[dst];

            // Defender: one keepable Power card per battle when the attack is dangerous.
            if self.cfg.powers && attackers > defenders {
                if self.take_kept(defender, Power::EnemyDefect) {
                    defenders += 4;
                    attackers = attackers.saturating_sub(4);
                } else if self.take_kept(defender, Power::WeaponStash) {
                    attackers = attackers.saturating_sub(2);
                } else if self.take_kept(defender, Power::Trainees) {
                    defenders += 3;
                }
            }

            self.battles += 1;
            let river = self.sh.graph.edge(src, dst) == Some(EdgeKind::River);
            let start_def = defenders;
            let mut m = Mods::default();
            if self.ev(Ev::ShieldWalls) {
                m.att_max = 2;
            }
            if self.ev(Ev::GreatFloods) && river {
                m.att_max = 1;
            }
            if self.ev(Ev::GreatHeathenArmy) {
                m.att_plus = 1;
            }
            if self.ev(Ev::AlfredsBurhs) && start_def >= th::BURHS_9TH {
                m.def_plus = 1;
            }
            if self.ev(Ev::NormanCastles) && start_def >= th::CASTLES_11TH {
                m.def_max = 3;
            }
            let no_withdraw = self.ev(Ev::Brunanburh);
            while attackers > 0 && defenders > 0 {
                if !no_withdraw && (attackers as f64) < strat.withdraw_ratio * defenders as f64 {
                    break;
                }
                let (al, dl) = roll_mod(&mut self.rng, attackers, defenders, self.cfg.dice, &self.sh.table, m);
                attackers -= al.min(attackers);
                defenders -= dl.min(defenders);
            }
            if defenders == 0 && attackers > 0 {
                self.battles_won += 1;
                captures += 1;
                self.players[p].stats.captures += 1;
                self.owner[dst] = p;
                self.troops[dst] = attackers;
                if self.count(defender) == 0 {
                    self.eliminate(defender, p);
                }
                if self.players.iter().filter(|pl| pl.alive).count() == 1 {
                    break;
                }
            } else {
                // Withdraw: survivors go back; defenders keep what is left.
                self.troops[src] += attackers;
                self.troops[dst] = defenders.max(1);
            }
        }
        captures
    }

    // ------------------------------------------------------------ manoeuvre

    /// One manoeuvre through own territories. Crossing a river: max 5 troops
    /// and they stop on the far side. Entering a mountain territory: stop there.
    fn manoeuvre(&mut self, p: usize) {
        let src = (0..N_TERR)
            .filter(|&t| self.owner[t] == p && self.troops[t] > 1 && !self.is_frontier(t))
            .max_by_key(|&t| self.troops[t]);
        let Some(src) = src else { return };
        // BFS to the nearest frontier territory (most threatened among the nearest).
        let mut prev = [usize::MAX; N_TERR];
        let mut dist = [u32::MAX; N_TERR];
        dist[src] = 0;
        let mut queue = std::collections::VecDeque::from([src]);
        let mut found: Option<usize> = None;
        let mut found_d = u32::MAX;
        while let Some(u) = queue.pop_front() {
            if dist[u] > found_d {
                break;
            }
            if u != src && self.is_frontier(u) {
                if found.is_none_or(|f| self.threat(u) > self.threat(f)) {
                    found = Some(u);
                    found_d = dist[u];
                }
                continue;
            }
            for &(v, kind) in &self.sh.graph.adj[u] {
                let blocked = (self.ev(Ev::HardWinter) && TERRITORIES[v].mountain)
                    || (self.ev(Ev::GreatFloods) && kind == EdgeKind::River);
                if self.owner[v] == p && dist[v] == u32::MAX && !blocked {
                    dist[v] = dist[u] + 1;
                    prev[v] = u;
                    queue.push_back(v);
                }
            }
        }
        let Some(dest) = found else { return };
        let mut path = vec![dest];
        while *path.last().unwrap() != src {
            let x = prev[*path.last().unwrap()];
            path.push(x);
        }
        path.reverse();
        let mut moving = self.troops[src] - 1;
        let mut at = src;
        for w in path.windows(2) {
            let (u, v) = (w[0], w[1]);
            at = v;
            if self.ev(Ev::RomanRoads) {
                continue; // roads: rivers and mountains don't stop the move
            }
            let river = self.sh.graph.edge(u, v) == Some(EdgeKind::River)
                || (self.ev(Ev::OffasDyke) && TERRITORIES[u].region != TERRITORIES[v].region);
            if river {
                moving = moving.min(5);
                break;
            }
            if TERRITORIES[v].mountain {
                break;
            }
        }
        self.troops[src] -= moving;
        self.troops[at] += moving;
    }

    // ------------------------------------------------------------- missions

    fn mission_done(&self, p: usize, m: usize) -> bool {
        match MISSIONS[m].goal {
            MissionGoal::Hold(_) => self.sh.mission_ids[m].iter().all(|&t| self.owner[t] == p),
            MissionGoal::Regions(k) => (0..N_REGIONS).filter(|&r| self.owns_region(p, r)).count() >= k,
            MissionGoal::AnySeaRoute => self.sh.graph.sea_routes.iter().any(|&(a, b)| self.owner[a] == p && self.owner[b] == p),
            MissionGoal::HoldJewelTower { turns } => self.jt_holder == p && self.jt_turns >= turns,
        }
    }

    fn check_missions(&mut self, p: usize) {
        let ms = self.players[p].missions.clone();
        for m in ms {
            if self.mission_done(p, m) {
                let bonus = MISSIONS[m].troops * if self.ev(Ev::AgeOfSaints) { 2 } else { 1 };
                self.players[p].pending += bonus;
                self.players[p].stats.mission_troops += bonus;
                self.players[p].missions.retain(|&x| x != m);
            }
        }
    }

    // --------------------------------------------------------------- turns

    fn take_turn(&mut self, p: usize) {
        let is_leader = self.leader() == p;
        // Start-of-turn century effects.
        if self.ev(Ev::SupplyProblems) {
            for t in 0..N_TERR {
                if self.owner[t] == p && self.troops[t] >= th::SUPPLY_12TH {
                    self.troops[t] -= 2;
                }
            }
        }
        if self.ev(Ev::PeasantsRevolt) && is_leader {
            if let Some(t) = (0..N_TERR).filter(|&t| self.owner[t] == p).max_by_key(|&t| self.troops[t]) {
                let x = self.troops[t];
                self.shrink(t, x.saturating_sub(3));
            }
        }
        let mut danegeld_troops = false;
        if self.ev(Ev::Danegeld) && is_leader {
            if let Some(w) = self.weakest().filter(|&w| w != p) {
                // Cheapest card goes to the weakest player; no cards = 2 troops instead.
                let hand = &self.players[p].hand;
                if let Some(i) = (0..hand.len()).min_by_key(|&i| TERRITORIES[hand[i]].stars) {
                    let c = self.players[p].hand.remove(i);
                    self.players[w].hand.push(c);
                } else {
                    self.players[w].pending += 2;
                    danegeld_troops = true;
                }
            }
        }

        // 1. Place troops
        let base: u32 = if self.ev(Ev::SaxonMigration) {
            5
        } else if self.ev(Ev::GreatFamine) {
            1
        } else {
            3
        };
        let mut new = base;
        if !(self.ev(Ev::TheAnarchy) && is_leader) {
            new += territory_bonus(self.count(p));
        }
        let region_mult = if self.ev(Ev::PettyKingdoms) || self.ev(Ev::OneKingdom) { 2 } else { 1 };
        for r in 0..N_REGIONS {
            if self.owns_region(p, r) {
                new += Region::ALL[r].bonus() * region_mult;
            }
            if self.ev(Ev::GoodHarvests) {
                let mine = self.sh.graph.region_members[r].iter().filter(|&&t| self.owner[t] == p).count();
                if mine >= 3 {
                    new += 1;
                }
            }
        }
        if self.ev(Ev::UnderdogKings) && self.weakest() == Some(p) {
            new += 3;
        }
        new += std::mem::take(&mut self.players[p].pending);
        new += self.trade_cards(p);
        if danegeld_troops {
            new = new.saturating_sub(2);
        }
        if self.wants_flip(p) {
            new += self.flip_power(p);
        }
        self.place(p, new);
        // 2. Attack
        let captures = self.attack_phase(p);
        if self.players.iter().filter(|pl| pl.alive).count() == 1 {
            return;
        }
        // 3. Manoeuvre
        self.manoeuvre(p);
        if self.ev(Ev::RomanRoads) {
            self.manoeuvre(p);
        }
        // 4. Draw cards
        let mut draws = match self.cfg.card_draw {
            CardDraw::PerCapture => captures,
            CardDraw::OnePerTurn => captures.min(1),
        };
        if self.ev(Ev::LandGrab) && captures > 0 {
            draws += 1;
        }
        if self.ev(Ev::DomesdaySurvey) {
            draws = draws.min(1);
        }
        for _ in 0..draws {
            self.draw_card(p);
        }
        // Jewel Tower hold counter: every player's turn counts.
        let jt_owner = self.owner[self.sh.jewel_tower];
        if jt_owner == self.jt_holder {
            self.jt_turns += 1;
        } else {
            self.jt_holder = jt_owner;
            self.jt_turns = 1;
        }
        if self.cfg.missions {
            self.check_missions(p);
        }
    }

    pub fn play(mut self) -> GameResult {
        let n = self.players.len();
        let mut turns = 0;
        let mut rounds = 0;
        let ended_by;
        let mut snapshots = Vec::new();
        self.begin_century();
        'game: loop {
            rounds += 1;
            self.round = rounds;
            snapshots.push(self.snapshot(rounds));
            for p in 0..n {
                if !self.players[p].alive {
                    continue;
                }
                self.take_turn(p);
                turns += 1;
                if self.players.iter().filter(|pl| pl.alive).count() == 1 {
                    ended_by = EndReason::Conquest;
                    break 'game;
                }
                // 5. Advance golden cavalry
                match self.cfg.timeline {
                    Timeline::Official => {
                        let gated = GATED_SPACES.contains(&self.cavalry);
                        if !gated || self.rng.d6().is_multiple_of(2) {
                            self.cavalry += 1;
                        }
                    }
                    Timeline::HousePace => {
                        if self.rng.d6() == 6 {
                            self.cavalry += 1;
                        }
                    }
                    Timeline::Off => {}
                }
                if self.cavalry >= LAST_SPACE {
                    ended_by = EndReason::Timeline;
                    break 'game;
                }
                if self.cavalry != self.centuries.last().map_or(usize::MAX, |c| c.space) {
                    self.begin_century();
                }
            }
            if self.cfg.timeline == Timeline::HousePace {
                self.cavalry += 1;
                if self.cavalry >= LAST_SPACE {
                    ended_by = EndReason::Timeline;
                    break 'game;
                }
                self.begin_century();
            }
            if rounds >= self.cfg.max_rounds {
                ended_by = EndReason::RoundLimit;
                break;
            }
        }
        let territories: Vec<usize> = (0..n).map(|p| self.count(p)).collect();
        let max = *territories.iter().max().unwrap();
        let winners = (0..n).filter(|&p| territories[p] == max).collect();
        GameResult {
            winners,
            territories,
            rounds,
            turns,
            ended_by,
            battles: self.battles,
            battles_won: self.battles_won,
            players: self.players.into_iter().map(|pl| pl.stats).collect(),
            snapshots,
            centuries: self.centuries,
        }
    }

    fn snapshot(&self, round: u32) -> Snapshot {
        let mut stacks: Vec<u32> = self.troops.to_vec();
        stacks.sort_unstable_by(|a, b| b.cmp(a));
        let n = self.players.len();
        let leader_territories = (0..n).map(|p| self.count(p)).max().unwrap_or(0);
        let leader_troops = (0..n)
            .map(|p| (0..N_TERR).filter(|&t| self.owner[t] == p).map(|t| self.troops[t]).sum::<u32>())
            .max()
            .unwrap_or(0);
        let alive: Vec<usize> = (0..n).filter(|&p| self.players[p].alive).collect();
        let hand_stars = alive.iter().map(|&p| self.hand_stars(p) as f64).sum::<f64>() / alive.len().max(1) as f64;
        let territories: Vec<usize> = (0..n).map(|p| self.count(p)).collect();
        Snapshot { century: self.cavalry, round, stacks, leader_territories, leader_troops, hand_stars, leader: self.leader(), territories }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn games_finish_and_conserve_board() {
        let sh = Shared::new();
        for seed in 0..200 {
            let cfg = Config { timeline: Timeline::Off, ..Config::default() };
            let strats = [Strategy::BALANCED, Strategy::AGGRESSIVE, Strategy::CAUTIOUS, Strategy::HOARDER];
            let r = Game::new(cfg, &sh, &strats, seed).play();
            assert_eq!(r.territories.iter().sum::<usize>(), 42);
            assert!(!r.winners.is_empty());
        }
    }

    #[test]
    fn deterministic() {
        let sh = Shared::new();
        let s = [Strategy::BALANCED; 4];
        let a = Game::new(Config::default(), &sh, &s, 7).play();
        let b = Game::new(Config::default(), &sh, &s, 7).play();
        assert_eq!(a.territories, b.territories);
        assert_eq!(a.turns, b.turns);
    }
}
