//! Steps 3 to 5 of "How the amount is worked out" in `docs/royalty-toml.md`:
//! the environment, the tier, and the price, for one producer family and one
//! consumer in one period. Pure integer arithmetic (ROYL-009).

use std::collections::BTreeMap;

use crate::schema::*;

/// Millionths of a cent in one cent.
pub const MILLIONTHS: u128 = 1_000_000;

/// What the scan found about how one consumer uses one producer family in
/// one period. Filled in by the scanner; pure data here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    /// Whether any use is production: false when the family is reached only
    /// through dev or build dependencies, or the consumer's `stage` is
    /// `development`.
    pub production: bool,
    pub dependency: Dependency,
    /// Measured counts. A unit that is absent was not measured.
    pub measured: BTreeMap<Unit, u64>,
}

impl Usage {
    pub fn production(dependency: Dependency) -> Self {
        Usage {
            production: true,
            dependency,
            measured: BTreeMap::new(),
        }
    }

    pub fn with(mut self, unit: Unit, count: u64) -> Self {
        self.measured.insert(unit, count);
        self
    }
}

/// Which tier applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TierChoice {
    /// The built-in free tier: development, tests, CI or staging (ROYL-002).
    NonProduction,
    /// A tier from the producer's file, by `id`.
    Tier { id: String, free: bool },
    /// No tier matched, so the use is free.
    NoMatch,
}

/// One price of the chosen tier, worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceLine {
    pub unit: Unit,
    /// The count used, or `None` when the unit was neither measured nor
    /// declared ("not measured": the line adds nothing).
    pub units: Option<u64>,
    /// Whether the count came from `[[consumer.package]] action = "declare"`.
    pub declared: bool,
    /// Units charged, after `max_units` and `included`. For `revenue`, the
    /// declared revenue in cents.
    pub charged: u64,
    pub millionths: u128,
}

/// The price of one family for one consumer in one period, before any cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Priced {
    pub tier: TierChoice,
    pub lines: Vec<PriceLine>,
    /// The lines added up and rounded down to whole cents, once.
    pub cents: u64,
}

impl Priced {
    fn free(tier: TierChoice) -> Self {
        Priced {
            tier,
            lines: Vec::new(),
            cents: 0,
        }
    }
}

/// Prices one producer family for one consumer.
///
/// `producer` is the family's file (the newest version's terms),
/// `consumer` the consumer's side, `usage` what the scan measured.
pub fn price(producer: &RoyaltyFile, consumer: &Consumer, usage: &Usage) -> Priced {
    // Step 3: non-production is free, always, before any tier is looked at.
    if !usage.production || consumer.stage == Stage::Development {
        return Priced::free(TierChoice::NonProduction);
    }

    // Step 4: the first tier whose conditions all hold.
    let Some(tier) = producer
        .tiers
        .iter()
        .find(|tier| matches(&tier.when, consumer, usage))
    else {
        return Priced::free(TierChoice::NoMatch);
    };
    let choice = TierChoice::Tier {
        id: tier.id.clone(),
        free: tier.free,
    };
    if tier.free {
        return Priced::free(choice);
    }

    // Step 5: each price, in millionths of a cent, rounded down once.
    let lines: Vec<PriceLine> = tier
        .prices
        .iter()
        .map(|p| price_line(p, producer, consumer, usage))
        .collect();
    let total: u128 = lines
        .iter()
        .fold(0u128, |sum, line| sum.saturating_add(line.millionths));
    Priced {
        tier: choice,
        lines,
        cents: u64::try_from(total / MILLIONTHS).unwrap_or(u64::MAX),
    }
}

/// Whether every condition of a tier holds. A left-out condition holds; a
/// condition whose input the consumer didn't give does not.
pub fn matches(when: &When, consumer: &Consumer, usage: &Usage) -> bool {
    let environment = when
        .environment
        .as_ref()
        .is_none_or(|envs| usage.production && envs.contains(&Environment::Production));
    let entity = when
        .entity
        .as_ref()
        .is_none_or(|list| list.contains(&consumer.entity));
    let class = when
        .class
        .as_ref()
        .is_none_or(|list| list.contains(&consumer.class));
    let revenue = when.revenue_below_usd.is_none_or(|limit| {
        consumer
            .annual_revenue_below_usd
            .is_some_and(|band| band <= limit)
    });
    let seats = when
        .seats_at_most
        .is_none_or(|limit| seats(consumer, usage).is_some_and(|n| n <= limit));
    let dependency = when
        .dependency
        .as_ref()
        .is_none_or(|list| list.contains(&usage.dependency));
    environment && entity && class && revenue && seats && dependency
}

/// Seats: the consumer's own number wins over the count from git.
fn seats(consumer: &Consumer, usage: &Usage) -> Option<u64> {
    consumer
        .seats
        .or_else(|| usage.measured.get(&Unit::Seat).copied())
}

/// The count for a unit: a `declare` exception for this family wins, then
/// the consumer's seat number, then what was measured.
fn units_for(
    unit: &Unit,
    producer: &RoyaltyFile,
    consumer: &Consumer,
    usage: &Usage,
) -> (Option<u64>, bool) {
    let declared = consumer
        .packages
        .iter()
        .find_map(|exception| match exception {
            PackageException::Declare {
                name,
                unit: declared_unit,
                units,
                ..
            } if declared_unit == unit && names_family(name, producer) => Some(*units),
            _ => None,
        });
    if declared.is_some() {
        return (declared, true);
    }
    if *unit == Unit::Seat {
        return (seats(consumer, usage), false);
    }
    (usage.measured.get(unit).copied(), false)
}

/// Whether an exception's `name` names this family: its family or one of its
/// packages.
pub fn names_family(name: &str, producer: &RoyaltyFile) -> bool {
    let name = normalize_package(name);
    normalize_package(&producer.project.family) == name || producer.covers(&name)
}

fn price_line(p: &Price, producer: &RoyaltyFile, consumer: &Consumer, usage: &Usage) -> PriceLine {
    let (units, declared) = units_for(&p.unit, producer, consumer, usage);
    let Some(count) = units else {
        return PriceLine {
            unit: p.unit.clone(),
            units: None,
            declared,
            charged: 0,
            millionths: 0,
        };
    };
    let (charged, millionths) = if p.unit == Unit::Revenue {
        let bp = u128::from(p.basis_points.unwrap_or(0));
        (
            count,
            u128::from(count)
                .saturating_mul(bp)
                .saturating_mul(MILLIONTHS)
                / 10_000,
        )
    } else {
        let limited = p.max_units.map_or(count, |max| count.min(max));
        let charged = limited.saturating_sub(p.included);
        let per = u128::from(p.per.max(1));
        let amount = u128::from(charged)
            .saturating_mul(u128::from(p.cents))
            .saturating_mul(MILLIONTHS)
            / per;
        (charged, amount)
    };
    PriceLine {
        unit: p.unit.clone(),
        units,
        declared,
        charged,
        millionths,
    }
}
