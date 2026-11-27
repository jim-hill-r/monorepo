//! Schema 1 of `royalty.toml`, with every default applied. The fields and
//! their meaning are in `docs/royalty-toml.md`.

/// Declares a word-valued field: its variants, the word for each, and
/// `parse`/`as_str`.
macro_rules! words {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $word:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn parse(word: &str) -> Option<Self> {
                match word {
                    $($word => Some($name::$variant),)+
                    _ => None,
                }
            }

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $word),+
                }
            }

            /// The allowed words, for messages: `"a", "b" or "c"`.
            pub fn choices() -> String {
                let words: Vec<String> =
                    Self::ALL.iter().map(|w| format!("\"{}\"", w.as_str())).collect();
                match words.split_last() {
                    Some((last, rest)) if !rest.is_empty() => {
                        format!("{} or {}", rest.join(", "), last)
                    }
                    _ => words.join(""),
                }
            }
        }
    };
}

words!(
    /// How strong the request for payment is.
    Binding { Voluntary = "voluntary", Agreement = "agreement", License = "license" }
);
words!(
    /// The evidence a producer asks for.
    Mode { Honor = "honor", Report = "report", Metered = "metered" }
);
words!(
    Period { Month = "month", Quarter = "quarter", Year = "year" }
);
words!(
    Reach { Direct = "direct", Transitive = "transitive" }
);
words!(
    /// The only environment a tier can name: the others are always free.
    Environment { Production = "production" }
);
words!(
    Entity { Person = "person", Organization = "organization" }
);
words!(
    Class {
        Commercial = "commercial",
        OpenSource = "open-source",
        Nonprofit = "nonprofit",
        Education = "education",
        Government = "government",
    }
);
words!(
    Dependency { Direct = "direct", Transitive = "transitive" }
);
words!(
    Window { Year = "year", Lifetime = "lifetime" }
);
words!(
    OnReach { Free = "free", Continue = "continue" }
);
words!(
    Source {
        Lockfile = "lockfile",
        Git = "git",
        Declared = "declared",
        Ci = "ci",
        Deploy = "deploy",
        Logs = "logs",
        Runtime = "runtime",
    }
);
words!(
    /// Whether a package offers the in-process counter.
    RuntimeOffer { Never = "never", OptIn = "opt-in" }
);
words!(
    Stage { Development = "development", Production = "production" }
);
words!(
    OnExceed { Fail = "fail", Scale = "scale" }
);
words!(
    TransitivePolicy { Pay = "pay", Report = "report", Ignore = "ignore" }
);
words!(
    Run { Manual = "manual", Ci = "ci", Schedule = "schedule" }
);
words!(
    Action { Decline = "decline", Cap = "cap", Declare = "declare" }
);

/// What one price counts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unit {
    Flat,
    Project,
    Seat,
    Deployment,
    Build,
    Release,
    Call,
    Revenue,
    /// `custom:<name>`, declared by the consumer.
    Custom(String),
}

impl Unit {
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "flat" => Unit::Flat,
            "project" => Unit::Project,
            "seat" => Unit::Seat,
            "deployment" => Unit::Deployment,
            "build" => Unit::Build,
            "release" => Unit::Release,
            "call" => Unit::Call,
            "revenue" => Unit::Revenue,
            _ => {
                let name = word.strip_prefix("custom:")?;
                if name.is_empty() {
                    return None;
                }
                Unit::Custom(name.to_string())
            }
        })
    }

    pub fn as_str(&self) -> String {
        match self {
            Unit::Flat => "flat".into(),
            Unit::Project => "project".into(),
            Unit::Seat => "seat".into(),
            Unit::Deployment => "deployment".into(),
            Unit::Build => "build".into(),
            Unit::Release => "release".into(),
            Unit::Call => "call".into(),
            Unit::Revenue => "revenue".into(),
            Unit::Custom(name) => format!("custom:{name}"),
        }
    }
}

/// A whole `royalty.toml`, producer and consumer sides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoyaltyFile {
    pub schema: u32,
    pub project: Project,
    pub license: License,
    pub payees: Vec<Payee>,
    pub terms: Terms,
    pub tiers: Vec<Tier>,
    pub caps: Caps,
    pub income: Option<Income>,
    pub measure: Measure,
    pub consumer: Option<Consumer>,
    /// Whether the file has `[terms]` or any `[[tier]]`. Without them the
    /// package has "no payment terms".
    pub has_terms: bool,
}

impl RoyaltyFile {
    /// Whether this file covers a package of this name (`-` and `_` are the
    /// same).
    pub fn covers(&self, package: &str) -> bool {
        let wanted = normalize_package(package);
        self.project
            .packages
            .iter()
            .any(|p| normalize_package(p) == wanted)
    }

    /// The consumer side, or the defaults of a file without `[consumer]`: a
    /// person in development with no budget.
    pub fn consumer_or_default(&self) -> Consumer {
        self.consumer.clone().unwrap_or_default()
    }
}

/// Package names compare with `-` and `_` treated as the same.
pub fn normalize_package(name: &str) -> String {
    name.replace('-', "_")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub repository: Option<String>,
    /// Defaults to `[name]`.
    pub packages: Vec<String>,
    /// Defaults to `name`.
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct License {
    pub code: Option<String>,
    pub binding: Binding,
    pub terms: Option<String>,
}

impl Default for License {
    fn default() -> Self {
        License {
            code: None,
            binding: Binding::Voluntary,
            terms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payee {
    pub name: String,
    pub share: u64,
    pub github: Option<String>,
    /// The known methods, in the payee's order. Unknown kinds are left out.
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    GithubSponsors { account: String },
    Invoice { contact: String },
    Link { url: String },
    X402 { network: String, address: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terms {
    pub mode: Mode,
    pub period: Period,
    pub reach: Reach,
    pub trial_days: u64,
    pub minimum_payment_cents: u64,
    pub report_to: Option<String>,
}

impl Default for Terms {
    fn default() -> Self {
        Terms {
            mode: Mode::Honor,
            period: Period::Month,
            reach: Reach::Direct,
            trial_days: 0,
            minimum_payment_cents: 500,
            report_to: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier {
    pub id: String,
    pub title: Option<String>,
    pub when: When,
    /// `free = true`, or no prices at all.
    pub free: bool,
    pub prices: Vec<Price>,
}

/// A left-out condition always holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct When {
    pub environment: Option<Vec<Environment>>,
    pub entity: Option<Vec<Entity>>,
    pub class: Option<Vec<Class>>,
    pub revenue_below_usd: Option<u64>,
    pub seats_at_most: Option<u64>,
    pub dependency: Option<Vec<Dependency>>,
}

impl When {
    pub fn is_empty(&self) -> bool {
        *self == When::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    pub unit: Unit,
    /// 0 for a `revenue` price, which uses `basis_points`.
    pub cents: u64,
    /// At least 1.
    pub per: u64,
    pub included: u64,
    pub max_units: Option<u64>,
    pub basis_points: Option<u64>,
    pub describe: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cap {
    pub period_cents: Option<u64>,
    pub year_cents: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Caps {
    pub project: Option<Cap>,
    pub person: Option<Cap>,
    pub organization: Option<Cap>,
    pub consumer: Option<Cap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Income {
    pub ceiling_cents: Option<u64>,
    pub window: Window,
    pub on_reach: OnReach,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    pub sources: Vec<Source>,
    pub sample_one_in: u64,
    pub runtime: RuntimeOffer,
}

/// The sources both sides allow by default.
pub const DEFAULT_SOURCES: [Source; 3] = [Source::Lockfile, Source::Git, Source::Declared];

impl Default for Measure {
    fn default() -> Self {
        Measure {
            sources: DEFAULT_SOURCES.to_vec(),
            sample_one_in: 1000,
            runtime: RuntimeOffer::Never,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consumer {
    pub id: Option<String>,
    pub entity: Entity,
    pub name: Option<String>,
    pub class: Class,
    pub annual_revenue_below_usd: Option<u64>,
    pub stage: Stage,
    pub seats: Option<u64>,
    pub deployments: Vec<Deployment>,
    pub budget: Budget,
    pub policy: Policy,
    pub measure: ConsumerMeasure,
    pub packages: Vec<PackageException>,
}

impl Default for Consumer {
    fn default() -> Self {
        Consumer {
            id: None,
            entity: Entity::Person,
            name: None,
            class: Class::Commercial,
            annual_revenue_below_usd: None,
            stage: Stage::Development,
            seats: None,
            deployments: Vec::new(),
            budget: Budget::default(),
            policy: Policy::default(),
            measure: ConsumerMeasure::default(),
            packages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deployment {
    pub name: String,
    pub manifest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    pub period_cents: Option<u64>,
    pub year_cents: Option<u64>,
    pub per_producer_period_cents: Option<u64>,
    pub on_exceed: OnExceed,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            period_cents: None,
            year_cents: None,
            per_producer_period_cents: None,
            on_exceed: OnExceed::Fail,
        }
    }
}

impl Budget {
    /// Whether the consumer set any limit.
    pub fn is_set(&self) -> bool {
        self.period_cents.is_some()
            || self.year_cents.is_some()
            || self.per_producer_period_cents.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub bindings: Vec<Binding>,
    pub modes: Vec<Mode>,
    pub transitive: TransitivePolicy,
    pub review_above_cents: u64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            bindings: Binding::ALL.to_vec(),
            modes: vec![Mode::Honor, Mode::Report],
            transitive: TransitivePolicy::Pay,
            review_above_cents: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerMeasure {
    pub sources: Vec<Source>,
    pub run: Run,
    pub runtime: bool,
}

impl Default for ConsumerMeasure {
    fn default() -> Self {
        ConsumerMeasure {
            sources: DEFAULT_SOURCES.to_vec(),
            run: Run::Manual,
            runtime: false,
        }
    }
}

/// One `[[consumer.package]]` exception.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageException {
    Decline {
        name: String,
        reason: Option<String>,
    },
    Cap {
        name: String,
        period_cents: u64,
        reason: Option<String>,
    },
    Declare {
        name: String,
        unit: Unit,
        units: u64,
        reason: Option<String>,
    },
}

impl PackageException {
    pub fn name(&self) -> &str {
        match self {
            PackageException::Decline { name, .. }
            | PackageException::Cap { name, .. }
            | PackageException::Declare { name, .. } => name,
        }
    }
}
