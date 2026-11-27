//! Text to [`RoyaltyFile`]: parse, apply the defaults, and validate. The
//! checks are the codes in `docs/royalty-toml.md` ("What `royalty check`
//! reports").

use crate::finding::{Code, Finding};
use crate::raw::*;
use crate::schema::*;
use crate::secrets::looks_secret;

/// The only schema this reader knows.
pub const SCHEMA: u32 = 1;

/// The result of reading one `royalty.toml`.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// `None` when the file can't be read at all: bad TOML, a wrong type,
    /// or an unknown `schema`.
    pub file: Option<RoyaltyFile>,
    pub findings: Vec<Finding>,
}

impl Parsed {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(Finding::is_error)
    }

    /// The file, only if it has no errors: what a calculation may use.
    pub fn valid(&self) -> Option<&RoyaltyFile> {
        if self.has_errors() {
            None
        } else {
            self.file.as_ref()
        }
    }
}

/// Reads one `royalty.toml` from its text.
pub fn parse(text: &str) -> Parsed {
    let mut findings = Vec::new();

    let table: toml::Table = match toml::from_str(text) {
        Ok(table) => table,
        Err(error) => {
            findings.push(toml_error(text, &error));
            return Parsed {
                file: None,
                findings,
            };
        }
    };
    find_secrets("", &toml::Value::Table(table), &mut findings);

    let mut unknown = Vec::new();
    let raw: RawFile = match toml::Deserializer::parse(text)
        .and_then(|de| serde_ignored::deserialize(de, |path| unknown.push(path_string(&path))))
    {
        Ok(raw) => raw,
        Err(error) => {
            findings.push(toml_error(text, &error));
            return Parsed {
                file: None,
                findings,
            };
        }
    };
    for path in unknown {
        findings.push(Finding::warning(
            Code::Rtoml003,
            path,
            "not a field of schema 1 (a typo?); it is ignored",
        ));
    }

    let schema = match raw.schema {
        None | Some(1) => SCHEMA,
        Some(other) => {
            findings.push(
                Finding::error(
                    Code::Rtoml002,
                    "schema",
                    format!("schema {other} is not known to this reader, which reads schema 1"),
                )
                .protects("ROYL-007"),
            );
            return Parsed {
                file: None,
                findings,
            };
        }
    };

    let mut v = Validator {
        findings: &mut findings,
    };
    let file = v.file(schema, raw);
    Parsed {
        file: Some(file),
        findings,
    }
}

fn toml_error(text: &str, error: &toml::de::Error) -> Finding {
    let at = match error.span() {
        Some(span) => format!(
            "line {}",
            text[..span.start.min(text.len())].matches('\n').count() + 1
        ),
        None => String::new(),
    };
    Finding::error(Code::Rtoml001, at, error.message().trim().to_string())
}

fn path_string(path: &serde_ignored::Path) -> String {
    use serde_ignored::Path;
    match path {
        Path::Root => String::new(),
        Path::Seq { parent, index } => format!("{}[{index}]", path_string(parent)),
        Path::Map { parent, key } => {
            let parent = path_string(parent);
            if parent.is_empty() {
                key.clone()
            } else {
                format!("{parent}.{key}")
            }
        }
        Path::Some { parent }
        | Path::NewtypeStruct { parent }
        | Path::NewtypeVariant { parent } => path_string(parent),
    }
}

fn find_secrets(at: &str, value: &toml::Value, findings: &mut Vec<Finding>) {
    match value {
        toml::Value::String(text) => {
            let key = at.rsplit('.').next().unwrap_or(at);
            let key = key.split('[').next().unwrap_or(key);
            if let Some(why) = looks_secret(key, text) {
                findings.push(
                    Finding::error(
                        Code::Rtoml007,
                        at,
                        format!(
                            "looks like a secret ({why}); a royalty.toml is public, so it holds account names only"
                        ),
                    )
                    .protects("ROYL-008"),
                );
            }
        }
        toml::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                find_secrets(&format!("{at}[{i}]"), item, findings);
            }
        }
        toml::Value::Table(table) => {
            for (key, item) in table {
                let at = if at.is_empty() {
                    key.clone()
                } else {
                    format!("{at}.{key}")
                };
                find_secrets(&at, item, findings);
            }
        }
        _ => {}
    }
}

struct Validator<'a> {
    findings: &'a mut Vec<Finding>,
}

impl Validator<'_> {
    fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// A count or amount: never negative (RTOML-009).
    fn count(&mut self, at: &str, value: Option<i64>) -> Option<u64> {
        let value = value?;
        match u64::try_from(value) {
            Ok(value) => Some(value),
            Err(_) => {
                self.push(
                    Finding::error(Code::Rtoml009, at, format!("{value} is negative"))
                        .protects("ROYL-009"),
                );
                Some(0)
            }
        }
    }

    /// A word field. An unknown word is reported with `code` and the default
    /// is used, so the rest of the file is still checked.
    fn word<T: Copy>(
        &mut self,
        code: Code,
        at: &str,
        value: Option<String>,
        default: T,
        parse: fn(&str) -> Option<T>,
        choices: fn() -> String,
    ) -> T {
        let Some(value) = value else {
            return default;
        };
        match parse(&value) {
            Some(word) => word,
            None => {
                self.push(Finding::error(
                    code,
                    at,
                    format!("\"{value}\" is not one of {}", choices()),
                ));
                default
            }
        }
    }

    fn words<T: Copy>(
        &mut self,
        code: Code,
        at: &str,
        values: Vec<String>,
        parse: fn(&str) -> Option<T>,
        choices: fn() -> String,
    ) -> Vec<T> {
        let mut out = Vec::new();
        for (i, value) in values.into_iter().enumerate() {
            match parse(&value) {
                Some(word) => out.push(word),
                None => self.push(Finding::error(
                    code,
                    format!("{at}[{i}]"),
                    format!("\"{value}\" is not one of {}", choices()),
                )),
            }
        }
        out
    }

    fn file(&mut self, schema: u32, raw: RawFile) -> RoyaltyFile {
        let has_terms = raw.terms.is_some() || !raw.tier.is_empty();
        let project = self.project(raw.project);
        let license = self.license(raw.license.unwrap_or_default());
        let payees = raw
            .payee
            .into_iter()
            .enumerate()
            .map(|(i, p)| self.payee(i, p))
            .collect::<Vec<_>>();
        let terms = self.terms(raw.terms.unwrap_or_default());
        let tiers = self.tiers(raw.tier);
        let caps = self.caps(raw.caps.unwrap_or_default());
        let income = raw.income.map(|i| self.income(i));
        let measure = self.measure(raw.measure.unwrap_or_default());
        let consumer = raw.consumer.map(|c| self.consumer(c));

        if has_terms && payees.is_empty() {
            self.push(Finding::warning(
                Code::Rtoml010,
                "payee",
                "the file has [terms] or [[tier]] but no [[payee]], so nobody could be paid",
            ));
        }
        if license.binding == Binding::License {
            self.push(Finding::warning(
                Code::Rtoml013,
                "license.binding",
                "payment is a condition of the licence, so the code is source-available, not open source",
            ));
        }

        RoyaltyFile {
            schema,
            project,
            license,
            payees,
            terms,
            tiers,
            caps,
            income,
            measure,
            consumer,
            has_terms,
        }
    }

    fn project(&mut self, raw: Option<RawProject>) -> Project {
        let Some(raw) = raw else {
            self.push(
                Finding::error(Code::Rtoml004, "project", "the [project] table is required")
                    .protects("ROYL-007"),
            );
            return Project::default();
        };
        let mut required = |field: &str, value: Option<String>| -> String {
            let value = value.unwrap_or_default();
            if value.trim().is_empty() {
                self.push(
                    Finding::error(
                        Code::Rtoml004,
                        format!("project.{field}"),
                        format!("[project] {field} is required and can't be empty"),
                    )
                    .protects("ROYL-007"),
                );
            }
            value
        };
        let name = required("name", raw.name);
        let description = required("description", raw.description);
        let packages = match raw.packages {
            Some(packages) if !packages.is_empty() => packages,
            _ => vec![name.clone()],
        };
        let family = raw.family.unwrap_or_else(|| name.clone());
        Project {
            name,
            description,
            repository: raw.repository,
            packages,
            family,
        }
    }

    fn license(&mut self, raw: RawLicense) -> License {
        let binding = self.word(
            Code::Rtoml001,
            "license.binding",
            raw.binding,
            Binding::Voluntary,
            Binding::parse,
            Binding::choices,
        );
        if binding != Binding::Voluntary && raw.terms.as_deref().is_none_or(str::is_empty) {
            self.push(Finding::error(
                Code::Rtoml005,
                "license.terms",
                format!(
                    "binding \"{}\" needs [license] terms: the text a consumer agrees to",
                    binding.as_str()
                ),
            ));
        }
        License {
            code: raw.code,
            binding,
            terms: raw.terms,
        }
    }

    fn payee(&mut self, i: usize, raw: RawPayee) -> Payee {
        let at = format!("payee[{i}]");
        let name = raw.name.unwrap_or_default();
        if name.trim().is_empty() {
            self.push(Finding::error(
                Code::Rtoml006,
                format!("{at}.name"),
                "a payee needs a name",
            ));
        }
        let share = self.count(&format!("{at}.share"), raw.share).unwrap_or(1);
        if share == 0 {
            self.push(Finding::error(
                Code::Rtoml006,
                format!("{at}.share"),
                "share must be at least 1",
            ));
        }
        let mut methods = Vec::new();
        for (j, m) in raw.method.into_iter().enumerate() {
            if let Some(method) = self.method(&format!("{at}.method[{j}]"), m) {
                methods.push(method);
            }
        }
        if methods.is_empty() {
            self.push(Finding::error(
                Code::Rtoml006,
                format!("{at}.method"),
                format!("payee \"{name}\" has no [[payee.method]] this reader can use"),
            ));
        }
        Payee {
            name,
            share,
            github: raw.github,
            methods,
        }
    }

    fn method(&mut self, at: &str, raw: RawMethod) -> Option<Method> {
        let Some(kind) = raw.kind else {
            self.push(Finding::error(
                Code::Rtoml006,
                format!("{at}.kind"),
                "a method needs a kind",
            ));
            return None;
        };
        let mut need = |field: &str, value: Option<String>| -> Option<String> {
            match value {
                Some(value) if !value.trim().is_empty() => Some(value),
                _ => {
                    self.push(Finding::error(
                        Code::Rtoml006,
                        format!("{at}.{field}"),
                        format!("a \"{kind}\" method needs {field}"),
                    ));
                    None
                }
            }
        };
        match kind.as_str() {
            "github-sponsors" => {
                need("account", raw.account).map(|account| Method::GithubSponsors { account })
            }
            "invoice" => need("contact", raw.contact).map(|contact| Method::Invoice { contact }),
            "link" => need("url", raw.url).map(|url| Method::Link { url }),
            "x402" => {
                let network = need("network", raw.network);
                let address = need("address", raw.address);
                Some(Method::X402 {
                    network: network?,
                    address: address?,
                })
            }
            _ => {
                self.push(Finding::warning(
                    Code::Rtoml006,
                    format!("{at}.kind"),
                    format!("method kind \"{kind}\" is not known to this reader; it is skipped"),
                ));
                None
            }
        }
    }

    fn terms(&mut self, raw: RawTerms) -> Terms {
        let defaults = Terms::default();
        if let Some(currency) = raw.currency
            && currency != "USD"
        {
            self.push(
                Finding::error(
                    Code::Rtoml009,
                    "terms.currency",
                    format!("currency \"{currency}\": schema 1 has only \"USD\""),
                )
                .protects("ROYL-009"),
            );
        }
        Terms {
            mode: self.word(
                Code::Rtoml001,
                "terms.mode",
                raw.mode,
                defaults.mode,
                Mode::parse,
                Mode::choices,
            ),
            period: self.word(
                Code::Rtoml001,
                "terms.period",
                raw.period,
                defaults.period,
                Period::parse,
                Period::choices,
            ),
            reach: self.word(
                Code::Rtoml001,
                "terms.reach",
                raw.reach,
                defaults.reach,
                Reach::parse,
                Reach::choices,
            ),
            trial_days: self
                .count("terms.trial_days", raw.trial_days)
                .unwrap_or(defaults.trial_days),
            minimum_payment_cents: self
                .count("terms.minimum_payment_cents", raw.minimum_payment_cents)
                .unwrap_or(defaults.minimum_payment_cents),
            report_to: raw.report_to,
        }
    }

    fn tiers(&mut self, raw: Vec<RawTier>) -> Vec<Tier> {
        let mut tiers: Vec<Tier> = Vec::new();
        let mut catch_all: Option<String> = None;
        for (i, raw) in raw.into_iter().enumerate() {
            let at = format!("tier[{i}]");
            let tier = self.tier(&at, raw);
            if !tier.id.is_empty() && tiers.iter().any(|t| t.id == tier.id) {
                self.push(Finding::error(
                    Code::Rtoml008,
                    format!("{at}.id"),
                    format!("tier id \"{}\" is used twice", tier.id),
                ));
            }
            if let Some(earlier) = &catch_all {
                self.push(Finding::warning(
                    Code::Rtoml011,
                    &at,
                    format!(
                        "tier \"{}\" can never apply: tier \"{earlier}\" before it has no conditions and matches every production use",
                        tier.id
                    ),
                ));
            } else if tier.when.is_empty() {
                catch_all = Some(tier.id.clone());
            }
            tiers.push(tier);
        }
        tiers
    }

    fn tier(&mut self, at: &str, raw: RawTier) -> Tier {
        let id = raw.id.unwrap_or_default();
        if id.trim().is_empty() {
            self.push(Finding::error(
                Code::Rtoml008,
                format!("{at}.id"),
                "a tier needs an id",
            ));
        }
        let when = self.when(&format!("{at}.when"), raw.when.unwrap_or_default());
        let prices = raw
            .price
            .into_iter()
            .enumerate()
            .filter_map(|(j, p)| self.price(&format!("{at}.price[{j}]"), p))
            .collect::<Vec<_>>();
        let free = raw.free.unwrap_or(false) || prices.is_empty();
        Tier {
            id,
            title: raw.title,
            when,
            free,
            prices,
        }
    }

    fn when(&mut self, at: &str, raw: RawWhen) -> When {
        for key in raw.unknown.keys() {
            self.push(Finding::error(
                Code::Rtoml008,
                format!("{at}.{key}"),
                format!(
                    "\"{key}\" is not a tier condition; an ignored condition would make the tier match more than intended"
                ),
            ));
        }
        let environment = raw.environment.map(|e| {
            let words = e.into_vec();
            let mut out = Vec::new();
            for (i, word) in words.into_iter().enumerate() {
                match Environment::parse(&word) {
                    Some(env) => out.push(env),
                    None => self.push(
                        Finding::error(
                            Code::Rtoml008,
                            format!("{at}.environment[{i}]"),
                            format!(
                                "\"{word}\": a tier can only name \"production\"; development, test, ci and staging are always free"
                            ),
                        )
                        .protects("ROYL-002"),
                    ),
                }
            }
            out
        });
        let entity = raw.entity.map(|e| {
            self.words(
                Code::Rtoml008,
                &format!("{at}.entity"),
                e.into_vec(),
                Entity::parse,
                Entity::choices,
            )
        });
        let class = raw.class.map(|c| {
            self.words(
                Code::Rtoml008,
                &format!("{at}.class"),
                c.into_vec(),
                Class::parse,
                Class::choices,
            )
        });
        let dependency = raw.dependency.map(|d| {
            self.words(
                Code::Rtoml008,
                &format!("{at}.dependency"),
                d.into_vec(),
                Dependency::parse,
                Dependency::choices,
            )
        });
        When {
            environment,
            entity,
            class,
            revenue_below_usd: self
                .count(&format!("{at}.revenue_below_usd"), raw.revenue_below_usd),
            seats_at_most: self.count(&format!("{at}.seats_at_most"), raw.seats_at_most),
            dependency,
        }
    }

    fn price(&mut self, at: &str, raw: RawPrice) -> Option<Price> {
        let unit = match raw.unit.as_deref().map(|u| (u, Unit::parse(u))) {
            Some((_, Some(unit))) => unit,
            Some((word, None)) => {
                self.push(Finding::error(
                    Code::Rtoml008,
                    format!("{at}.unit"),
                    format!(
                        "\"{word}\" is not a unit: use flat, project, seat, deployment, build, release, call, revenue or custom:<name>"
                    ),
                ));
                return None;
            }
            None => {
                self.push(Finding::error(
                    Code::Rtoml008,
                    format!("{at}.unit"),
                    "a price needs a unit",
                ));
                return None;
            }
        };
        let cents = self.count(&format!("{at}.cents"), raw.cents);
        let per = self.count(&format!("{at}.per"), raw.per).unwrap_or(1);
        let included = self
            .count(&format!("{at}.included"), raw.included)
            .unwrap_or(0);
        let max_units = self.count(&format!("{at}.max_units"), raw.max_units);
        let basis_points = self.count(&format!("{at}.basis_points"), raw.basis_points);
        let mut ok = true;
        if per == 0 {
            self.push(Finding::error(
                Code::Rtoml008,
                format!("{at}.per"),
                "per must be at least 1",
            ));
            ok = false;
        }
        if unit == Unit::Revenue {
            if basis_points.is_none() {
                self.push(Finding::error(
                    Code::Rtoml008,
                    format!("{at}.basis_points"),
                    "a revenue price needs basis_points",
                ));
                ok = false;
            }
        } else if cents.is_none() {
            self.push(Finding::error(
                Code::Rtoml008,
                format!("{at}.cents"),
                "a price needs cents",
            ));
            ok = false;
        }
        if matches!(unit, Unit::Custom(_))
            && raw.describe.as_deref().is_none_or(|d| d.trim().is_empty())
        {
            self.push(Finding::error(
                Code::Rtoml008,
                format!("{at}.describe"),
                "a custom unit needs describe: what one unit is",
            ));
            ok = false;
        }
        ok.then_some(Price {
            unit,
            cents: cents.unwrap_or(0),
            per,
            included,
            max_units,
            basis_points,
            describe: raw.describe,
        })
    }

    fn cap(&mut self, at: &str, raw: Option<RawCap>) -> Option<Cap> {
        let raw = raw?;
        Some(Cap {
            period_cents: self.count(&format!("{at}.period_cents"), raw.period_cents),
            year_cents: self.count(&format!("{at}.year_cents"), raw.year_cents),
        })
    }

    fn caps(&mut self, raw: RawCaps) -> Caps {
        Caps {
            project: self.cap("caps.project", raw.project),
            person: self.cap("caps.person", raw.person),
            organization: self.cap("caps.organization", raw.organization),
            consumer: self.cap("caps.consumer", raw.consumer),
        }
    }

    fn income(&mut self, raw: RawIncome) -> Income {
        Income {
            ceiling_cents: self.count("income.ceiling_cents", raw.ceiling_cents),
            window: self.word(
                Code::Rtoml001,
                "income.window",
                raw.window,
                Window::Year,
                Window::parse,
                Window::choices,
            ),
            on_reach: self.word(
                Code::Rtoml001,
                "income.on_reach",
                raw.on_reach,
                OnReach::Free,
                OnReach::parse,
                OnReach::choices,
            ),
            status: raw.status,
        }
    }

    fn measure(&mut self, raw: RawMeasure) -> Measure {
        let defaults = Measure::default();
        Measure {
            sources: raw
                .sources
                .map(|s| {
                    self.words(
                        Code::Rtoml001,
                        "measure.sources",
                        s,
                        Source::parse,
                        Source::choices,
                    )
                })
                .unwrap_or(defaults.sources),
            sample_one_in: match self.count("measure.sample_one_in", raw.sample_one_in) {
                Some(0) => {
                    self.push(Finding::error(
                        Code::Rtoml001,
                        "measure.sample_one_in",
                        "sample_one_in must be at least 1",
                    ));
                    defaults.sample_one_in
                }
                Some(n) => n,
                None => defaults.sample_one_in,
            },
            runtime: self.word(
                Code::Rtoml001,
                "measure.runtime",
                raw.runtime,
                defaults.runtime,
                RuntimeOffer::parse,
                RuntimeOffer::choices,
            ),
        }
    }

    fn consumer(&mut self, raw: RawConsumer) -> Consumer {
        let c = Code::Rtoml012;
        let d = Consumer::default();
        let entity = self.word(
            c,
            "consumer.entity",
            raw.entity,
            d.entity,
            Entity::parse,
            Entity::choices,
        );
        let class = self.word(
            c,
            "consumer.class",
            raw.class,
            d.class,
            Class::parse,
            Class::choices,
        );
        let stage = self.word(
            c,
            "consumer.stage",
            raw.stage,
            d.stage,
            Stage::parse,
            Stage::choices,
        );
        let annual_revenue_below_usd = self.count(
            "consumer.annual_revenue_below_usd",
            raw.annual_revenue_below_usd,
        );
        let seats = self.count("consumer.seats", raw.seats);

        let mut deployments = Vec::new();
        for (i, dep) in raw.deployment.into_iter().enumerate() {
            let at = format!("consumer.deployment[{i}]");
            let name = dep.name.unwrap_or_default();
            match dep.manifest {
                Some(manifest) if !manifest.trim().is_empty() => {
                    deployments.push(Deployment { name, manifest })
                }
                _ => self.push(Finding::error(
                    c,
                    format!("{at}.manifest"),
                    "a deployment needs a manifest",
                )),
            }
        }

        let budget = raw.budget.unwrap_or_default();
        let budget = Budget {
            period_cents: self.count("consumer.budget.period_cents", budget.period_cents),
            year_cents: self.count("consumer.budget.year_cents", budget.year_cents),
            per_producer_period_cents: self.count(
                "consumer.budget.per_producer_period_cents",
                budget.per_producer_period_cents,
            ),
            on_exceed: self.word(
                c,
                "consumer.budget.on_exceed",
                budget.on_exceed,
                OnExceed::Fail,
                OnExceed::parse,
                OnExceed::choices,
            ),
        };

        let policy = raw.policy.unwrap_or_default();
        let dp = Policy::default();
        let policy = Policy {
            bindings: policy
                .bindings
                .map(|b| {
                    self.words(
                        c,
                        "consumer.policy.bindings",
                        b,
                        Binding::parse,
                        Binding::choices,
                    )
                })
                .unwrap_or(dp.bindings),
            modes: policy
                .modes
                .map(|m| self.words(c, "consumer.policy.modes", m, Mode::parse, Mode::choices))
                .unwrap_or(dp.modes),
            transitive: self.word(
                c,
                "consumer.policy.transitive",
                policy.transitive,
                dp.transitive,
                TransitivePolicy::parse,
                TransitivePolicy::choices,
            ),
            review_above_cents: self
                .count(
                    "consumer.policy.review_above_cents",
                    policy.review_above_cents,
                )
                .unwrap_or(0),
        };

        let measure = raw.measure.unwrap_or_default();
        let dm = ConsumerMeasure::default();
        let measure = ConsumerMeasure {
            sources: measure
                .sources
                .map(|s| {
                    self.words(
                        c,
                        "consumer.measure.sources",
                        s,
                        Source::parse,
                        Source::choices,
                    )
                })
                .unwrap_or(dm.sources),
            run: self.word(
                c,
                "consumer.measure.run",
                measure.run,
                dm.run,
                Run::parse,
                Run::choices,
            ),
            runtime: measure.runtime.unwrap_or(false),
        };

        let mut packages = Vec::new();
        for (i, p) in raw.package.into_iter().enumerate() {
            if let Some(exception) = self.package(&format!("consumer.package[{i}]"), p) {
                packages.push(exception);
            }
        }

        Consumer {
            id: raw.id,
            entity,
            name: raw.name,
            class,
            annual_revenue_below_usd,
            stage,
            seats,
            deployments,
            budget,
            policy,
            measure,
            packages,
        }
    }

    fn package(&mut self, at: &str, raw: RawPackage) -> Option<PackageException> {
        let c = Code::Rtoml012;
        let name = match raw.name {
            Some(name) if !name.trim().is_empty() => name,
            _ => {
                self.push(Finding::error(
                    c,
                    format!("{at}.name"),
                    "an exception needs the package or family name",
                ));
                return None;
            }
        };
        let action = raw.action.as_deref().and_then(Action::parse);
        let Some(action) = action else {
            self.push(Finding::error(
                c,
                format!("{at}.action"),
                format!("action must be one of {}", Action::choices()),
            ));
            return None;
        };
        let reason = raw.reason;
        match action {
            Action::Decline => Some(PackageException::Decline { name, reason }),
            Action::Cap => match self.count(&format!("{at}.period_cents"), raw.period_cents) {
                Some(period_cents) => Some(PackageException::Cap {
                    name,
                    period_cents,
                    reason,
                }),
                None => {
                    self.push(Finding::error(
                        c,
                        format!("{at}.period_cents"),
                        "action \"cap\" needs period_cents",
                    ));
                    None
                }
            },
            Action::Declare => {
                let unit = raw.unit.as_deref().and_then(Unit::parse);
                let units = self.count(&format!("{at}.units"), raw.units);
                match (unit, units) {
                    (Some(unit), Some(units)) => Some(PackageException::Declare {
                        name,
                        unit,
                        units,
                        reason,
                    }),
                    _ => {
                        self.push(Finding::error(
                            c,
                            at,
                            "action \"declare\" needs a known unit and units",
                        ));
                        None
                    }
                }
            }
        }
    }
}
