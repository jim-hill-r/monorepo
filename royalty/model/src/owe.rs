//! The whole calculation for one consumer project in one period: reach,
//! price (steps 3 to 5), the consumer's exceptions and transitive policy,
//! the producer's caps (step 6), the consumer's budget (step 8) and small
//! amounts held back (step 9). The rules are "Exact rules" in
//! `docs/royalty-toml.md`. Phase 1 has no ledger, so year-to-date totals
//! are zero and a `year_cents` limit applies to the one period on its own.

use crate::price::{Priced, Usage, names_family, price};
use crate::schema::*;

/// One producer family the scan found, with its terms and its use.
#[derive(Debug, Clone)]
pub struct FamilyUse<'a> {
    /// The family name, used to sort and to name it in the report.
    pub family: String,
    /// The family's `royalty.toml` (the newest version's).
    pub producer: &'a RoyaltyFile,
    pub usage: Usage,
}

/// What lowered an amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Limit {
    /// `[[consumer.package]] action = "decline"`.
    Declined,
    /// `[[consumer.package]] action = "cap"`.
    ConsumerPackageCap,
    /// `[consumer.policy] transitive = "report"` or `"ignore"`.
    TransitivePolicy,
    /// `[caps.project]`.
    ProjectCap,
    /// `[caps.person]`.
    PersonCap,
    /// `[caps.organization]`.
    OrganizationCap,
    /// `[caps.consumer]`, the fallback payer cap.
    ConsumerCap,
    /// `[consumer.budget] per_producer_period_cents`.
    PerProducerBudget,
    /// `[consumer.budget] period_cents` or `year_cents`, by scaling.
    TotalBudget,
}

impl Limit {
    pub fn as_str(self) -> &'static str {
        match self {
            Limit::Declined => "declined by the consumer",
            Limit::ConsumerPackageCap => "the consumer's cap for this package",
            Limit::TransitivePolicy => "the consumer's transitive policy",
            Limit::ProjectCap => "the producer's project cap",
            Limit::PersonCap => "the producer's person cap",
            Limit::OrganizationCap => "the producer's organization cap",
            Limit::ConsumerCap => "the producer's consumer cap",
            Limit::PerProducerBudget => "the consumer's per-producer budget",
            Limit::TotalBudget => "the consumer's total budget",
        }
    }
}

/// One step that lowered (or would have lowered) an amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub limit: Limit,
    /// The amount after this step.
    pub cents: u64,
}

/// Where a family ended up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// Something is owed this period.
    Owed,
    /// Free: non-production, a free tier, no tier matched, or capped to $0.
    Free,
    /// Transitive, and the producer's `reach` is `direct`.
    NotBillable,
    /// Under `minimum_payment_cents`: owes nothing this period.
    HeldBack,
}

/// The result for one family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyOwed {
    pub family: String,
    pub binding: Binding,
    pub dependency: Dependency,
    pub priced: Priced,
    /// Each limit that lowered the amount, in order.
    pub steps: Vec<Step>,
    /// The final amount owed this period, in cents.
    pub cents: u64,
    pub status: Status,
    /// An `agreement` or `license` amount the consumer's budget can't hold.
    pub over_budget: bool,
    /// The family's `binding` isn't in `[consumer.policy] bindings`.
    pub binding_refused: bool,
    /// Things the consumer should know, such as an exception that couldn't
    /// lower an `agreement` amount.
    pub notes: Vec<String>,
}

impl FamilyOwed {
    /// The price before any cap or budget.
    pub fn before_caps(&self) -> u64 {
        self.priced.cents
    }

    /// What set the final number, if anything lowered it.
    pub fn set_by(&self) -> Option<Limit> {
        self.steps.last().map(|s| s.limit)
    }

    /// The amount after the producer's caps (and the consumer's exceptions
    /// and policy, which come first), before the consumer's budget.
    pub fn after_producer_caps(&self) -> u64 {
        self.steps
            .iter()
            .rev()
            .find(|s| s.limit < Limit::PerProducerBudget)
            .map_or(self.priced.cents, |s| s.cents)
    }

    fn lower(&mut self, limit: Limit, to: u64) {
        let current = self.current();
        if to < current {
            self.steps.push(Step { limit, cents: to });
        }
    }

    fn current(&self) -> u64 {
        self.steps.last().map_or(self.priced.cents, |s| s.cents)
    }
}

/// The result for one consumer project in one period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owed {
    /// Sorted by family name.
    pub families: Vec<FamilyOwed>,
    pub total_cents: u64,
    /// The total limit used: the lower of the budget's `period_cents` and
    /// `year_cents`.
    pub budget_cents: Option<u64>,
    /// The result breaks the consumer's budget, so `royalty check` fails.
    pub over_budget: bool,
}

/// Works out what one consumer project owes every family in one period.
pub fn owe(consumer: &Consumer, uses: &[FamilyUse<'_>]) -> Owed {
    let mut uses: Vec<&FamilyUse<'_>> = uses.iter().collect();
    uses.sort_by(|a, b| a.family.cmp(&b.family));

    let mut families: Vec<FamilyOwed> = uses.iter().map(|u| one_family(consumer, u)).collect();

    // Step 8, the total: agreement and license amounts are kept whole;
    // voluntary amounts share the space that is left.
    let budget = &consumer.budget;
    let budget_cents = match (budget.period_cents, budget.year_cents) {
        (Some(p), Some(y)) => Some(p.min(y)),
        (p, y) => p.or(y),
    };
    let mut over_budget = families.iter().any(|f| f.over_budget);
    if let Some(limit) = budget_cents {
        let total: u64 = families.iter().map(FamilyOwed::current).sum();
        if total > limit {
            let fixed: u64 = families
                .iter()
                .filter(|f| f.binding != Binding::Voluntary)
                .map(FamilyOwed::current)
                .sum();
            let voluntary = total - fixed;
            match budget.on_exceed {
                OnExceed::Fail => over_budget = true,
                OnExceed::Scale => {
                    let space = limit.saturating_sub(fixed);
                    if fixed > limit {
                        over_budget = true;
                    }
                    for family in families
                        .iter_mut()
                        .filter(|f| f.binding == Binding::Voluntary)
                    {
                        let amount = family.current();
                        let scaled = if voluntary == 0 {
                            0
                        } else {
                            u64::try_from(
                                u128::from(amount) * u128::from(space) / u128::from(voluntary),
                            )
                            .unwrap_or(0)
                        };
                        family.lower(Limit::TotalBudget, scaled);
                    }
                }
            }
            if over_budget {
                for family in families
                    .iter_mut()
                    .filter(|f| f.binding != Binding::Voluntary && f.current() > 0)
                {
                    family.over_budget = true;
                }
            }
        }
    }

    // Step 9: small amounts are held back.
    for (family, family_use) in families.iter_mut().zip(&uses) {
        let amount = family.current();
        family.cents = amount;
        if family.status == Status::NotBillable {
            continue;
        }
        family.status = if amount == 0 {
            Status::Free
        } else if amount < family_use.producer.terms.minimum_payment_cents {
            family.cents = 0;
            Status::HeldBack
        } else {
            Status::Owed
        };
    }

    let total_cents = families.iter().map(|f| f.cents).sum();
    Owed {
        families,
        total_cents,
        budget_cents,
        over_budget,
    }
}

fn one_family(consumer: &Consumer, family_use: &FamilyUse<'_>) -> FamilyOwed {
    let producer = family_use.producer;
    let usage = &family_use.usage;
    let binding = producer.license.binding;
    let voluntary = binding == Binding::Voluntary;
    let mut owed = FamilyOwed {
        family: family_use.family.clone(),
        binding,
        dependency: usage.dependency,
        priced: price(producer, consumer, usage),
        steps: Vec::new(),
        cents: 0,
        status: Status::Free,
        over_budget: false,
        binding_refused: false,
        notes: Vec::new(),
    };

    // Step 2: out of reach.
    if usage.dependency == Dependency::Transitive && producer.terms.reach == Reach::Direct {
        owed.priced = Priced {
            tier: crate::price::TierChoice::NoMatch,
            lines: Vec::new(),
            cents: 0,
        };
        owed.status = Status::NotBillable;
        return owed;
    }
    if owed.priced.cents > 0 && !consumer.policy.bindings.contains(&binding) {
        owed.binding_refused = true;
    }

    // The consumer's exceptions: only voluntary amounts can be lowered.
    for exception in &consumer.packages {
        if !names_family(exception.name(), producer) {
            continue;
        }
        let (limit, to) = match exception {
            PackageException::Decline { .. } => (Limit::Declined, 0),
            PackageException::Cap { period_cents, .. } => {
                (Limit::ConsumerPackageCap, *period_cents)
            }
            PackageException::Declare { .. } => continue,
        };
        if voluntary {
            owed.lower(limit, to);
        } else if to < owed.current() {
            owed.notes.push(format!(
                "{} can't lower a \"{}\" amount",
                limit.as_str(),
                binding.as_str()
            ));
        }
    }

    // The consumer's transitive policy.
    if usage.dependency == Dependency::Transitive
        && consumer.policy.transitive != TransitivePolicy::Pay
    {
        if voluntary {
            owed.lower(Limit::TransitivePolicy, 0);
        } else if owed.current() > 0 {
            owed.notes.push(format!(
                "the transitive policy can't lower a \"{}\" amount",
                binding.as_str()
            ));
        }
    }

    // Step 6: the producer's caps. The project cap, then the payer cap.
    let caps = &producer.caps;
    if let Some(cap) = caps.project {
        owed.lower(Limit::ProjectCap, cap_limit(cap));
    }
    let payer = match consumer.entity {
        Entity::Person => caps.person.map(|c| (Limit::PersonCap, c)),
        Entity::Organization => caps.organization.map(|c| (Limit::OrganizationCap, c)),
    }
    .or_else(|| caps.consumer.map(|c| (Limit::ConsumerCap, c)));
    if let Some((limit, cap)) = payer {
        owed.lower(limit, cap_limit(cap));
    }

    // Step 8, per producer.
    if let Some(limit) = consumer.budget.per_producer_period_cents {
        if voluntary {
            owed.lower(Limit::PerProducerBudget, limit);
        } else if owed.current() > limit {
            owed.over_budget = true;
        }
    }
    owed
}

/// The most one cap allows this period: the lower of `period_cents` and
/// what is left of `year_cents` (all of it until the Phase 2 ledger).
fn cap_limit(cap: Cap) -> u64 {
    match (cap.period_cents, cap.year_cents) {
        (Some(p), Some(y)) => p.min(y),
        (Some(p), None) => p,
        (None, Some(y)) => y,
        (None, None) => u64::MAX,
    }
}
