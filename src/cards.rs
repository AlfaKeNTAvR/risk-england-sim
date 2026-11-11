//! Mission cards and Power cards, transcribed from the physical decks.

#[derive(Clone, Copy, Debug)]
pub enum MissionGoal {
    /// Own every listed territory.
    Hold(&'static [&'static str]),
    /// Own at least this many complete regions.
    Regions(usize),
    /// Own both ports of any one sea route.
    AnySeaRoute,
    /// House interpretation: hold Jewel Tower for 3 turns in total, where any
    /// player's turn counts.
    HoldJewelTower { turns: u32 },
}

pub struct Mission {
    pub text: &'static str,
    pub troops: u32,
    pub goal: MissionGoal,
}

use MissionGoal::*;

pub static MISSIONS: [Mission; 10] = [
    Mission { text: "Capture the port territories of Dover, Barnard and St Briavels", troops: 5, goal: Hold(&["Dover", "Barnard", "St Briavels"]) },
    Mission { text: "Capture the smallest territory, Piel", troops: 2, goal: Hold(&["Piel"]) },
    Mission { text: "Capture 2 regions", troops: 6, goal: Regions(2) },
    Mission { text: "Capture the south coast: Dover, Battle, Portchester, Sherborne, Okehampton, Tintagel", troops: 5, goal: Hold(&["Dover", "Battle", "Portchester", "Sherborne", "Okehampton", "Tintagel"]) },
    Mission { text: "Capture Dunstanburgh and Carlisle", troops: 2, goal: Hold(&["Dunstanburgh", "Carlisle"]) },
    Mission { text: "Capture the port territories of Whalley, Portchester and Tintagel", troops: 5, goal: Hold(&["Whalley", "Portchester", "Tintagel"]) },
    Mission { text: "Capture the tri-border territories: Bosworth, Bushmede, High Wycombe", troops: 3, goal: Hold(&["Bosworth", "Bushmede", "High Wycombe"]) },
    Mission { text: "Capture and hold Jewel Tower for 3 turns", troops: 2, goal: HoldJewelTower { turns: 3 } },
    Mission { text: "Capture the island strongholds of Scilly and Carisbrooke", troops: 5, goal: Hold(&["Scilly", "Carisbrooke"]) },
    Mission { text: "Capture both ports of a sea route", troops: 3, goal: AnySeaRoute },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Power {
    /// Add 5 troops to a territory. Play immediately.
    Reinforcements,
    /// Keep. When attacked, destroy 2 attacking troops before the dice roll.
    WeaponStash,
    /// Keep. Add 3 troops to a territory under attack.
    Trainees,
    /// Keep. Destroy all but 1 troop in an opponent's territory.
    Artillery,
    /// Keep. +4 troops to a territory under attack, and remove up to 4 troops
    /// from the attacking territory.
    EnemyDefect,
    /// Lose 3 troops from each of your territories in the North (house rule:
    /// only the player who flips it).
    Snowfall,
    /// Lose all but 1 troop from one of your territories.
    Plague,
    /// Lose all but 1 troop from one of your territories.
    Starvation,
    /// Lose 3 troops from a territory.
    TiredTroops,
    /// Lose 2 troops from a territory.
    TroopsDefect,
}

pub static POWER_DECK: [Power; 10] = [
    Power::Reinforcements,
    Power::WeaponStash,
    Power::Trainees,
    Power::Artillery,
    Power::EnemyDefect,
    Power::Snowfall,
    Power::Plague,
    Power::Starvation,
    Power::TiredTroops,
    Power::TroopsDefect,
];

impl Power {
    /// Cards that are kept face up after flipping and played later.
    pub fn is_keepable(self) -> bool {
        matches!(self, Power::WeaponStash | Power::Trainees | Power::Artillery | Power::EnemyDefect)
    }
}
