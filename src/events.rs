//! Century events: era-wide modifiers, 3 per century for the 5th-14th.
//! When the cavalry lands on a century, one of its 3 events is rolled
//! (die 1-2 / 3-4 / 5-6) and stays active until the cavalry moves again.
//! The player-facing text lives in docs/house-rules.html; keep both in sync.

/// Stack thresholds, calibrated with `examples/stacks_by_century.rs`.
pub mod th {
    pub const PLAGUE_6TH: u32 = 3;
    pub const BURHS_9TH: u32 = 4;
    pub const WINTER_10TH: u32 = 6;
    pub const CASTLES_11TH: u32 = 5;
    pub const SUPPLY_12TH: u32 = 8;
    pub const FEVER_13TH: u32 = 10;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ev {
    // 5th
    SaxonMigration,
    RomanRoads,
    LandGrab,
    // 6th
    PettyKingdoms,
    PlagueOfJustinian,
    AgeOfSaints,
    // 7th
    ShieldWalls,
    PeaceOfTheChurch,
    UnderdogKings,
    // 8th
    VikingRaids,
    OffasDyke,
    GoodHarvests,
    // 9th
    GreatHeathenArmy,
    Danegeld,
    AlfredsBurhs,
    // 10th
    OneKingdom,
    Brunanburh,
    HardWinter,
    // 11th
    NormanCastles,
    HarryingOfTheNorth,
    DomesdaySurvey,
    // 12th
    TheAnarchy,
    GreatFloods,
    SupplyProblems,
    // 13th
    MagnaCarta,
    BaronsWar,
    CampFever,
    // 14th
    BlackDeath,
    GreatFamine,
    PeasantsRevolt,
}

pub static ALL: [Ev; 30] = [
    Ev::SaxonMigration, Ev::RomanRoads, Ev::LandGrab,
    Ev::PettyKingdoms, Ev::PlagueOfJustinian, Ev::AgeOfSaints,
    Ev::ShieldWalls, Ev::PeaceOfTheChurch, Ev::UnderdogKings,
    Ev::VikingRaids, Ev::OffasDyke, Ev::GoodHarvests,
    Ev::GreatHeathenArmy, Ev::Danegeld, Ev::AlfredsBurhs,
    Ev::OneKingdom, Ev::Brunanburh, Ev::HardWinter,
    Ev::NormanCastles, Ev::HarryingOfTheNorth, Ev::DomesdaySurvey,
    Ev::TheAnarchy, Ev::GreatFloods, Ev::SupplyProblems,
    Ev::MagnaCarta, Ev::BaronsWar, Ev::CampFever,
    Ev::BlackDeath, Ev::GreatFamine, Ev::PeasantsRevolt,
];

impl Ev {
    pub fn index(self) -> usize {
        ALL.iter().position(|&e| e == self).unwrap()
    }
    /// Timeline space this event belongs to (0 = 5th century).
    pub fn space(self) -> usize {
        self.index() / 3
    }
    pub fn of_space(space: usize, roll_slot: usize) -> Option<Ev> {
        ALL.get(space * 3 + roll_slot).copied()
    }
    pub fn name(self) -> &'static str {
        match self {
            Ev::SaxonMigration => "Saxon migration",
            Ev::RomanRoads => "Roman roads",
            Ev::LandGrab => "Land grab",
            Ev::PettyKingdoms => "Petty kingdoms",
            Ev::PlagueOfJustinian => "Plague of Justinian",
            Ev::AgeOfSaints => "Age of saints",
            Ev::ShieldWalls => "Shield walls",
            Ev::PeaceOfTheChurch => "Peace of the Church",
            Ev::UnderdogKings => "Underdog kings",
            Ev::VikingRaids => "Viking raids",
            Ev::OffasDyke => "Offa's Dyke",
            Ev::GoodHarvests => "Good harvests",
            Ev::GreatHeathenArmy => "Great Heathen Army",
            Ev::Danegeld => "Danegeld",
            Ev::AlfredsBurhs => "Alfred's burhs",
            Ev::OneKingdom => "One kingdom",
            Ev::Brunanburh => "Battle of Brunanburh",
            Ev::HardWinter => "Hard winter",
            Ev::NormanCastles => "Norman castles",
            Ev::HarryingOfTheNorth => "Harrying of the North",
            Ev::DomesdaySurvey => "Domesday survey",
            Ev::TheAnarchy => "The Anarchy",
            Ev::GreatFloods => "Great floods",
            Ev::SupplyProblems => "Supply problems",
            Ev::MagnaCarta => "Magna Carta",
            Ev::BaronsWar => "Barons' War",
            Ev::CampFever => "Camp fever",
            Ev::BlackDeath => "Black Death",
            Ev::GreatFamine => "Great Famine",
            Ev::PeasantsRevolt => "Peasants' Revolt",
        }
    }
    /// Events about pacts do nothing until pacts are simulated.
    pub fn simulated(self) -> bool {
        !matches!(self, Ev::PeaceOfTheChurch | Ev::BaronsWar)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventMode {
    Off,
    /// Roll one of the 3 events whenever the cavalry lands on a century.
    Deck,
    /// Only this event, whenever the cavalry lands on its century.
    Force(Ev),
}
