//! Static board data for Risk: The Battle for Medieval England.
//! Transcribed from photos of the physical board and the rulebook, with every
//! uncertain border confirmed by the owner of the game (Nov 2026).

pub const N_TERR: usize = 42;
pub const N_REGIONS: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    North,
    Midlands,
    EastAnglia,
    SouthEast,
    WestCountry,
}

impl Region {
    pub const ALL: [Region; N_REGIONS] = [
        Region::North,
        Region::Midlands,
        Region::EastAnglia,
        Region::SouthEast,
        Region::WestCountry,
    ];
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn bonus(self) -> u32 {
        match self {
            Region::North => 5,
            Region::Midlands => 7,
            Region::EastAnglia => 3,
            Region::SouthEast => 5,
            Region::WestCountry => 3,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Region::North => "North",
            Region::Midlands => "Midlands",
            Region::EastAnglia => "East Anglia",
            Region::SouthEast => "South East",
            Region::WestCountry => "West Country",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EdgeKind {
    Land,
    /// A border with a river on it. Manoeuvres across it: max 5 troops per
    /// turn, and the troops stop on the far side for the rest of the turn.
    /// Attacks are not affected.
    River,
    /// A dashed sea line between two port territories.
    Sea,
}

pub struct Territory {
    pub name: &'static str,
    pub region: Region,
    /// Manoeuvres entering a mountain territory stop there for the turn.
    pub mountain: bool,
    /// Stars on this territory's card (1 or 2).
    pub stars: u8,
}

use EdgeKind::*;
use Region::*;

const fn t(name: &'static str, region: Region, mountain: bool, stars: u8) -> Territory {
    Territory { name, region, mountain, stars }
}

pub static TERRITORIES: [Territory; N_TERR] = [
    // North (9)
    t("Dunstanburgh", North, false, 2),       // 0
    t("Carlisle", North, true, 2),            // 1
    t("Barnard", North, false, 1),            // 2
    t("Brougham", North, true, 1),            // 3
    t("Piel", North, false, 1),               // 4
    t("Cliffords", North, true, 2),           // 5
    t("Whalley", North, false, 2),            // 6
    t("Leeds", North, false, 1),              // 7
    t("Conisbrough", North, false, 2),        // 8
    // Midlands (11)
    t("Beeston", Midlands, true, 2),          // 9
    t("Bolsover", Midlands, false, 2),        // 10
    t("Nottingham", Midlands, false, 1),      // 11
    t("Bolingbroke", Midlands, false, 2),     // 12
    t("Wall", Midlands, true, 2),             // 13
    t("Stokesay", Midlands, false, 2),        // 14
    t("Ashby de la Zouch", Midlands, false, 1), // 15
    t("Kenilworth", Midlands, false, 1),      // 16
    t("Leigh Court", Midlands, false, 1),     // 17
    t("Goodrich", Midlands, false, 1),        // 18
    t("Bosworth", Midlands, false, 2),        // 19
    // East Anglia (6)
    t("Castle Acre", EastAnglia, false, 2),   // 20
    t("Longthorpe", EastAnglia, false, 2),    // 21
    t("Framlingham", EastAnglia, false, 2),   // 22
    t("Bushmede", EastAnglia, false, 1),      // 23
    t("Berkhamstead", EastAnglia, false, 1),  // 24
    t("Colchester", EastAnglia, false, 2),    // 25
    // South East (9)
    t("Uffington", SouthEast, false, 2),      // 26
    t("High Wycombe", SouthEast, false, 1),   // 27
    t("Donnington", SouthEast, false, 1),     // 28
    t("Jewel Tower", SouthEast, false, 1),    // 29
    t("Farnham", SouthEast, false, 1),        // 30
    t("Portchester", SouthEast, false, 2),    // 31
    t("Dover", SouthEast, false, 2),          // 32
    t("Battle", SouthEast, false, 2),         // 33
    t("Carisbrooke", SouthEast, false, 1),    // 34
    // West Country (7)
    t("St Briavels", WestCountry, true, 2),   // 35
    t("Old Sarum", WestCountry, true, 2),     // 36
    t("Farleigh", WestCountry, false, 2),     // 37
    t("Sherborne", WestCountry, false, 1),    // 38
    t("Okehampton", WestCountry, false, 2),   // 39
    t("Tintagel", WestCountry, false, 2),     // 40
    t("Scilly", WestCountry, false, 1),       // 41
];

pub static EDGES: &[(&str, &str, EdgeKind)] = &[
    ("Dunstanburgh", "Carlisle", Land), ("Dunstanburgh", "Barnard", Land),
    ("Carlisle", "Barnard", Land), ("Carlisle", "Brougham", Land), ("Carlisle", "Piel", Land),
    ("Barnard", "Brougham", Land), ("Barnard", "Cliffords", Land),
    ("Brougham", "Piel", Land), ("Brougham", "Whalley", Land), ("Brougham", "Cliffords", Land),
    ("Piel", "Whalley", Land), ("Cliffords", "Whalley", Land), ("Cliffords", "Leeds", Land),
    ("Cliffords", "Conisbrough", Land), ("Whalley", "Leeds", Land), ("Whalley", "Beeston", Land),
    ("Leeds", "Conisbrough", River), ("Leeds", "Beeston", River), ("Leeds", "Bolsover", River),
    ("Conisbrough", "Bolsover", Land), ("Conisbrough", "Nottingham", Land), ("Conisbrough", "Bolingbroke", Land),
    ("Beeston", "Bolsover", Land), ("Beeston", "Wall", Land), ("Beeston", "Stokesay", Land),
    ("Bolsover", "Nottingham", Land), ("Bolsover", "Wall", Land), ("Bolsover", "Ashby de la Zouch", Land),
    ("Nottingham", "Bolingbroke", Land), ("Nottingham", "Ashby de la Zouch", Land),
    ("Bolingbroke", "Ashby de la Zouch", Land), ("Bolingbroke", "Longthorpe", Land), ("Bolingbroke", "Castle Acre", Land),
    ("Wall", "Stokesay", Land), ("Wall", "Kenilworth", Land), ("Wall", "Leigh Court", Land),
    ("Stokesay", "Leigh Court", Land), ("Stokesay", "Goodrich", Land),
    ("Ashby de la Zouch", "Kenilworth", Land), ("Ashby de la Zouch", "Bosworth", Land), ("Ashby de la Zouch", "Longthorpe", Land),
    ("Kenilworth", "Bosworth", Land), ("Kenilworth", "Leigh Court", Land), ("Kenilworth", "St Briavels", Land), ("Kenilworth", "Uffington", Land),
    ("Leigh Court", "Goodrich", Land), ("Leigh Court", "St Briavels", Land), ("Goodrich", "St Briavels", Land),
    ("Bosworth", "Longthorpe", Land), ("Bosworth", "Bushmede", Land), ("Bosworth", "High Wycombe", Land), ("Bosworth", "Uffington", Land),
    ("Castle Acre", "Longthorpe", Land), ("Castle Acre", "Framlingham", Land),
    ("Longthorpe", "Framlingham", Land), ("Longthorpe", "Bushmede", Land), ("Longthorpe", "Berkhamstead", Land), ("Longthorpe", "Colchester", Land),
    ("Framlingham", "Colchester", Land), ("Bushmede", "Berkhamstead", Land), ("Bushmede", "High Wycombe", Land),
    ("Berkhamstead", "Colchester", Land), ("Berkhamstead", "High Wycombe", Land), ("Berkhamstead", "Jewel Tower", Land),
    ("Colchester", "Jewel Tower", Land), ("Colchester", "Dover", River),
    ("Uffington", "High Wycombe", Land), ("Uffington", "Donnington", Land), ("Uffington", "St Briavels", Land), ("Uffington", "Old Sarum", Land),
    ("High Wycombe", "Donnington", Land), ("High Wycombe", "Jewel Tower", Land),
    ("Donnington", "Jewel Tower", Land), ("Donnington", "Old Sarum", Land),
    ("Donnington", "Portchester", River), ("Donnington", "Farnham", River),
    ("Jewel Tower", "Farnham", River), ("Jewel Tower", "Dover", River),
    ("Farnham", "Portchester", Land), ("Farnham", "Dover", Land), ("Farnham", "Battle", Land),
    ("Portchester", "Battle", Land), ("Portchester", "Old Sarum", Land), ("Portchester", "Sherborne", Land),
    ("Dover", "Battle", Land),
    ("St Briavels", "Old Sarum", Land), ("St Briavels", "Farleigh", Land),
    ("Old Sarum", "Farleigh", Land), ("Old Sarum", "Sherborne", Land), ("Farleigh", "Sherborne", Land),
    ("Farleigh", "Okehampton", Land), ("Sherborne", "Okehampton", Land), ("Okehampton", "Tintagel", Land),
    ("Whalley", "St Briavels", Sea), ("Barnard", "Bolingbroke", Sea), ("Bolingbroke", "Dover", Sea),
    ("Dover", "Portchester", Sea), ("Portchester", "Carisbrooke", Sea), ("Tintagel", "Scilly", Sea),
];

pub fn id(name: &str) -> usize {
    TERRITORIES
        .iter()
        .position(|t| t.name == name)
        .unwrap_or_else(|| panic!("unknown territory {name}"))
}

/// Precomputed adjacency: for each territory, (neighbour, edge kind).
pub struct Graph {
    pub adj: Vec<Vec<(usize, EdgeKind)>>,
    pub region_members: [Vec<usize>; N_REGIONS],
    pub sea_routes: Vec<(usize, usize)>,
}

impl Graph {
    pub fn build() -> Self {
        let mut adj = vec![Vec::new(); N_TERR];
        let mut sea_routes = Vec::new();
        for &(a, b, k) in EDGES {
            let (a, b) = (id(a), id(b));
            assert!(!adj[a].iter().any(|&(n, _)| n == b), "duplicate edge {a}-{b}");
            adj[a].push((b, k));
            adj[b].push((a, k));
            if k == Sea {
                sea_routes.push((a, b));
            }
        }
        let mut region_members: [Vec<usize>; N_REGIONS] = Default::default();
        for (i, t) in TERRITORIES.iter().enumerate() {
            region_members[t.region.idx()].push(i);
        }
        Graph { adj, region_members, sea_routes }
    }

    pub fn edge(&self, a: usize, b: usize) -> Option<EdgeKind> {
        self.adj[a].iter().find(|&&(n, _)| n == b).map(|&(_, k)| k)
    }
}

/// Troops for trading in cards, by total stars traded (board chart).
pub fn troops_for_stars(stars: u32) -> u32 {
    match stars {
        0 | 1 => 0,
        2 => 2,
        3 => 4,
        4 => 7,
        5 => 10,
        6 => 13,
        7 => 17,
        8 => 21,
        9 => 25,
        _ => 30, // 10 is the top of the chart
    }
}

/// Bonus troops for number of territories held (board chart): 12-14 => +1 ... 40-42 => +10.
pub fn territory_bonus(n: usize) -> u32 {
    if n < 12 {
        0
    } else {
        (((n - 9) / 3) as u32).min(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_shape() {
        let g = Graph::build();
        assert_eq!(TERRITORIES.len(), 42);
        let stars: u32 = TERRITORIES.iter().map(|t| t.stars as u32).sum();
        assert_eq!(stars, 66);
        assert_eq!(TERRITORIES.iter().filter(|t| t.stars == 1).count(), 18);
        let sizes: Vec<usize> = g.region_members.iter().map(|m| m.len()).collect();
        assert_eq!(sizes, vec![9, 11, 6, 9, 7]);
        assert_eq!(EDGES.iter().filter(|e| e.2 == River).count(), 8);
        assert_eq!(g.sea_routes.len(), 6);
        // connected
        let mut seen = [false; N_TERR];
        let mut stack = vec![0];
        while let Some(x) = stack.pop() {
            if seen[x] { continue; }
            seen[x] = true;
            stack.extend(g.adj[x].iter().map(|&(n, _)| n));
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn charts() {
        assert_eq!(territory_bonus(11), 0);
        assert_eq!(territory_bonus(12), 1);
        assert_eq!(territory_bonus(15), 2);
        assert_eq!(territory_bonus(39), 10);
        assert_eq!(territory_bonus(42), 10);
        assert_eq!(troops_for_stars(10), 30);
    }
}
