//! Bot personalities. Bots do not need to play well; they need to play
//! consistently so that rule variants can be compared against each other.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TradePolicy {
    /// Trade as soon as the hand holds at least this many stars
    /// (always trading the largest set up to 10 stars).
    AtStars(u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlipPolicy {
    Never,
    /// Flip one face-down Power card every turn while any remain.
    Always,
    /// Flip only when holding fewer territories than the average player.
    WhenBehind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Diplomacy {
    /// Never proposes or accepts pacts.
    None,
    /// Signs pacts and always keeps them.
    Loyal,
    /// Signs pacts and betrays when attacking the partner scores at least
    /// `factor` times the best other attack (or there is no other attack).
    Opportunist { factor: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct Strategy {
    pub name: &'static str,
    /// Start an attack only if committed attackers / defenders >= this.
    pub attack_ratio: f64,
    /// Withdraw from a battle when attackers < this * defenders.
    pub withdraw_ratio: f64,
    pub trade: TradePolicy,
    pub flip: FlipPolicy,
    /// Share of new troops placed on the attack spearhead (rest defends).
    pub spearhead_share: f64,
    /// Troops left behind when attacking out of a territory that still
    /// borders enemies (human players keep a rear guard of about 2).
    pub rear_guard: u32,
    /// Before building the spearhead, top up thin frontier territories to
    /// this many troops (uses at most a third of new troops).
    pub frontier_min: u32,
    pub diplomacy: Diplomacy,
    /// Pact length this bot proposes, in turns each (1-4).
    pub pact_len: u32,
}

impl Strategy {
    pub const BALANCED: Strategy = Strategy {
        name: "balanced",
        attack_ratio: 1.5,
        withdraw_ratio: 0.8,
        trade: TradePolicy::AtStars(6),
        flip: FlipPolicy::WhenBehind,
        spearhead_share: 0.7,
        rear_guard: 2,
        frontier_min: 2,
        diplomacy: Diplomacy::None,
        pact_len: 2,
    };
    pub const AGGRESSIVE: Strategy = Strategy {
        name: "aggressive",
        attack_ratio: 1.0,
        withdraw_ratio: 0.5,
        trade: TradePolicy::AtStars(4),
        flip: FlipPolicy::Always,
        spearhead_share: 1.0,
        rear_guard: 1,
        frontier_min: 0,
        diplomacy: Diplomacy::None,
        pact_len: 2,
    };
    pub const CAUTIOUS: Strategy = Strategy {
        name: "cautious",
        attack_ratio: 2.5,
        withdraw_ratio: 1.2,
        trade: TradePolicy::AtStars(6),
        flip: FlipPolicy::Never,
        spearhead_share: 0.4,
        rear_guard: 3,
        frontier_min: 3,
        diplomacy: Diplomacy::None,
        pact_len: 2,
    };
    pub const HOARDER: Strategy = Strategy {
        name: "hoarder",
        trade: TradePolicy::AtStars(10),
        ..Strategy::BALANCED
    };
    pub const TRADE_AT_3: Strategy = Strategy {
        name: "trade-at-3",
        trade: TradePolicy::AtStars(3),
        ..Strategy::BALANCED
    };
    pub const TRADE_AT_8: Strategy = Strategy {
        name: "trade-at-8",
        trade: TradePolicy::AtStars(8),
        ..Strategy::BALANCED
    };
    pub const FLIPPER: Strategy = Strategy {
        name: "flipper",
        flip: FlipPolicy::Always,
        ..Strategy::BALANCED
    };
    pub const LOYAL: Strategy = Strategy {
        name: "loyal",
        diplomacy: Diplomacy::Loyal,
        ..Strategy::BALANCED
    };
    pub const LOYAL_LONG: Strategy = Strategy {
        name: "loyal-long",
        diplomacy: Diplomacy::Loyal,
        pact_len: 4,
        ..Strategy::BALANCED
    };
    pub const LOYAL_SHORT: Strategy = Strategy {
        name: "loyal-1turn",
        diplomacy: Diplomacy::Loyal,
        pact_len: 1,
        ..Strategy::BALANCED
    };
    pub const OPPORTUNIST: Strategy = Strategy {
        name: "opportunist",
        diplomacy: Diplomacy::Opportunist { factor: 1.5 },
        ..Strategy::BALANCED
    };
    pub const BACKSTABBER: Strategy = Strategy {
        name: "backstabber",
        diplomacy: Diplomacy::Opportunist { factor: 1.0 },
        ..Strategy::BALANCED
    };
    pub const NO_PACTS: Strategy = Strategy {
        name: "no-pacts",
        diplomacy: Diplomacy::None,
        ..Strategy::BALANCED
    };
    pub const NEVER_FLIP: Strategy = Strategy {
        name: "never-flip",
        flip: FlipPolicy::Never,
        ..Strategy::BALANCED
    };
}
