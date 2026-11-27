//! One problem `royalty check` found in a `royalty.toml`. The codes are
//! listed in `docs/royalty-toml.md` ("What `royalty check` reports").

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// Not valid TOML, or a field with the wrong type or an unknown value.
    Rtoml001,
    /// A `schema` this reader doesn't know.
    Rtoml002,
    /// A field or table this reader doesn't know.
    Rtoml003,
    /// `[project]`, its `name` or its `description` is missing or empty.
    Rtoml004,
    /// `agreement` or `license` binding with no `[license] terms`.
    Rtoml005,
    /// A payee with no usable payment method.
    Rtoml006,
    /// A value that looks like a secret.
    Rtoml007,
    /// An unusable tier or price.
    Rtoml008,
    /// A negative amount or count, or a currency other than USD.
    Rtoml009,
    /// Terms or tiers with nobody to pay.
    Rtoml010,
    /// A tier that can never be reached.
    Rtoml011,
    /// An invalid `[consumer]` field.
    Rtoml012,
    /// `binding = "license"`: source-available, not open source.
    Rtoml013,
}

impl Code {
    pub fn as_str(self) -> &'static str {
        match self {
            Code::Rtoml001 => "RTOML-001",
            Code::Rtoml002 => "RTOML-002",
            Code::Rtoml003 => "RTOML-003",
            Code::Rtoml004 => "RTOML-004",
            Code::Rtoml005 => "RTOML-005",
            Code::Rtoml006 => "RTOML-006",
            Code::Rtoml007 => "RTOML-007",
            Code::Rtoml008 => "RTOML-008",
            Code::Rtoml009 => "RTOML-009",
            Code::Rtoml010 => "RTOML-010",
            Code::Rtoml011 => "RTOML-011",
            Code::Rtoml012 => "RTOML-012",
            Code::Rtoml013 => "RTOML-013",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Warning,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Warning => "warning",
            Level::Error => "error",
        }
    }
}

/// Printed as `error[RTOML-008] tier[0].price[0]: ... (ROYL-002)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: Code,
    pub level: Level,
    /// Where in the file: a key path such as `tier[1].price[0].cents`, or
    /// `line 3` for a syntax error. Empty for the whole file.
    pub at: String,
    pub message: String,
    /// The requirement in `docs/requirements.md` this finding protects.
    pub requirement: Option<&'static str>,
}

impl Finding {
    pub fn error(code: Code, at: impl Into<String>, message: impl Into<String>) -> Self {
        Finding {
            code,
            level: Level::Error,
            at: at.into(),
            message: message.into(),
            requirement: None,
        }
    }

    pub fn warning(code: Code, at: impl Into<String>, message: impl Into<String>) -> Self {
        Finding {
            level: Level::Warning,
            ..Finding::error(code, at, message)
        }
    }

    pub fn protects(mut self, requirement: &'static str) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn is_error(&self) -> bool {
        self.level == Level::Error
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.level.as_str(), self.code.as_str())?;
        if !self.at.is_empty() {
            write!(f, " {}", self.at)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(requirement) = self.requirement {
            write!(f, " ({requirement})")?;
        }
        Ok(())
    }
}
