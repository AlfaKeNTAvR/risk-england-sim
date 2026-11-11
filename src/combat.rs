//! Dice combat. Attacker: 1 black die per committed troop (max 3).
//! Defender: 1 red die per troop (max 2). Pair highest to lowest; ties go to
//! the defender.

use crate::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiceRule {
    /// Rulebook: both sides roll at the same time, defender rolls min(2, troops).
    Classic,
    /// House rule: attacker rolls first; the defender sees the result and then
    /// chooses to roll 1 or 2 dice (here: the choice that maximises the
    /// defender's expected troop trade).
    DefenderSees,
    /// House variant: attacker rolls first; the defender rolls one die, looks,
    /// then decides whether to roll a second (bots use the exact best choice).
    DefenderOneByOne,
}

/// Losses (attacker, defender) from comparing sorted-descending dice.
pub fn compare(att: &[u8], def: &[u8]) -> (u32, u32) {
    let mut a_loss = 0;
    let mut d_loss = 0;
    for i in 0..att.len().min(def.len()) {
        if att[i] > def[i] {
            d_loss += 1;
        } else {
            a_loss += 1;
        }
    }
    (a_loss, d_loss)
}

fn sorted_desc(v: &mut [u8]) {
    v.sort_unstable_by(|a, b| b.cmp(a));
}

/// Lookup table for the house rule: index by sorted attack dice -> best number
/// of defence dice (1 or 2). Built by exact enumeration.
pub struct DefenderTable {
    best: Vec<u8>,
    /// One-by-one rule: roll the second die? Indexed by key(attack) * 7 + first die.
    second: Vec<bool>,
}

fn key(a: &[u8]) -> usize {
    let mut k = a.len() * 343;
    let mut m = 1;
    for &x in a {
        k += x as usize * m;
        m *= 7;
    }
    k
}

impl DefenderTable {
    pub fn get(&self, sorted_attack: &[u8]) -> u8 {
        self.best[key(sorted_attack)]
    }
    pub fn roll_second(&self, sorted_attack: &[u8], first: u8) -> bool {
        self.second[key(sorted_attack) * 7 + first as usize]
    }
}

impl DefenderTable {
    pub fn build() -> Self {
        let mut best = vec![0u8; 4 * 343];
        let mut second = vec![false; 4 * 343 * 7];
        for n in 1..=3usize {
            let mut idx = vec![1u8; n];
            loop {
                let mut a = idx.clone();
                sorted_desc(&mut a);
                if best[key(&a)] == 0 {
                    let mut ev1 = 0.0;
                    for d in 1..=6u8 {
                        let (x, y) = compare(&a, &[d]);
                        ev1 += x as f64 - y as f64;
                    }
                    ev1 /= 6.0;
                    let mut ev2 = 0.0;
                    for d1 in 1..=6u8 {
                        for d2 in 1..=6u8 {
                            let mut d = [d1, d2];
                            sorted_desc(&mut d);
                            let (x, y) = compare(&a, &d);
                            ev2 += x as f64 - y as f64;
                        }
                    }
                    ev2 /= 36.0;
                    best[key(&a)] = if ev2 > ev1 { 2 } else { 1 };
                    for d1 in 1..=6u8 {
                        let (x, y) = compare(&a, &[d1]);
                        let stop = x as f64 - y as f64;
                        let mut go = 0.0;
                        for d2 in 1..=6u8 {
                            let mut d = [d1, d2];
                            sorted_desc(&mut d);
                            let (x, y) = compare(&a, &d);
                            go += x as f64 - y as f64;
                        }
                        second[key(&a) * 7 + d1 as usize] = go / 6.0 > stop;
                    }
                }
                // increment odometer
                let mut i = 0;
                loop {
                    if i == n {
                        break;
                    }
                    if idx[i] < 6 {
                        idx[i] += 1;
                        break;
                    }
                    idx[i] = 1;
                    i += 1;
                }
                if i == n {
                    break;
                }
            }
        }
        DefenderTable { best, second }
    }
}

/// One exchange of dice. `attackers` = committed attacking troops,
/// `defenders` = troops in the defending territory.
/// Dice changes from century events.
#[derive(Clone, Copy, Debug)]
pub struct Mods {
    /// Most dice the attacker may roll (normally 3).
    pub att_max: u32,
    /// Added to the attacker's highest die.
    pub att_plus: u8,
    /// Added to the defender's highest die.
    pub def_plus: u8,
    /// Most dice the defender may roll (normally 2).
    pub def_max: u32,
}

impl Default for Mods {
    fn default() -> Self {
        Mods { att_max: 3, att_plus: 0, def_plus: 0, def_max: 2 }
    }
}

pub fn roll_once(
    rng: &mut Rng,
    attackers: u32,
    defenders: u32,
    rule: DiceRule,
    table: &DefenderTable,
) -> (u32, u32) {
    roll_mod(rng, attackers, defenders, rule, table, Mods::default())
}

pub fn roll_mod(
    rng: &mut Rng,
    attackers: u32,
    defenders: u32,
    rule: DiceRule,
    table: &DefenderTable,
    m: Mods,
) -> (u32, u32) {
    let na = attackers.min(m.att_max).clamp(1, 3) as usize;
    let mut a = [0u8; 3];
    for x in a.iter_mut().take(na) {
        *x = rng.d6();
    }
    let a = &mut a[..na];
    sorted_desc(a);
    let nd = if m.def_max >= 3 {
        defenders.min(3) as usize
    } else {
        match rule {
            DiceRule::Classic => defenders.min(2) as usize,
            DiceRule::DefenderSees => {
                if defenders >= 2 {
                    table.get(a) as usize
                } else {
                    1
                }
            }
            DiceRule::DefenderOneByOne => 0, // rolled below, one at a time
        }
    };
    let mut d = [0u8; 3];
    let nd = if nd == 0 {
        d[0] = rng.d6();
        if defenders >= 2 && table.roll_second(a, d[0]) {
            d[1] = rng.d6();
            2
        } else {
            1
        }
    } else {
        for x in d.iter_mut().take(nd) {
            *x = rng.d6();
        }
        nd
    };
    a[0] += m.att_plus; // after the defender's decisions, which see the plain dice
    let d = &mut d[..nd];
    sorted_desc(d);
    d[0] += m.def_plus;
    compare(a, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exact expected losses per 3v2 exchange, matching the numbers computed
    /// in the design chat: classic 0.921 / 1.079, defender-sees 0.750 / 0.750.
    #[test]
    fn exact_expectations() {
        let table = DefenderTable::build();
        let mut classic = (0.0, 0.0);
        let mut house = (0.0, 0.0);
        for a1 in 1..=6u8 {
            for a2 in 1..=6u8 {
                for a3 in 1..=6u8 {
                    let mut a = [a1, a2, a3];
                    sorted_desc(&mut a);
                    let k = table.get(&a);
                    for d1 in 1..=6u8 {
                        for d2 in 1..=6u8 {
                            let mut d = [d1, d2];
                            sorted_desc(&mut d);
                            let (x, y) = compare(&a, &d);
                            classic.0 += x as f64;
                            classic.1 += y as f64;
                            let (x, y) = if k == 2 { compare(&a, &d) } else { compare(&a, &[d1]) };
                            house.0 += x as f64;
                            house.1 += y as f64;
                        }
                    }
                }
            }
        }
        let n = 216.0 * 36.0;
        assert!((classic.0 / n - 0.9209).abs() < 1e-3);
        assert!((classic.1 / n - 1.0791).abs() < 1e-3);
        assert!((house.0 / n - 0.7504).abs() < 1e-3);
        assert!((house.1 / n - 0.7496).abs() < 1e-3);
    }
}
