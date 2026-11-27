//! Build a shopping list that adds up.
//!
//! Recipe ingredients are free text such as `"2 tablespoons olive oil"`,
//! `"1 1/2 cups flour"` or `"Salt and pepper to taste"`. [`parse_ingredient`]
//! splits one into a quantity, a unit and an item, and [`build_shopping_list`]
//! sums the quantities of the same item per unit across a week's recipes, so
//! seven recipes that each need 2 tablespoons of olive oil give one line,
//! "14 tablespoons olive oil", instead of one line that hides six of them.
//!
//! Staples that most kitchens already have (oils, salt, pepper, garlic) are
//! listed separately from the things to buy.

use std::collections::BTreeMap;
use std::fmt;

/// A non-negative fraction, kept exact so `1/3 + 2/3` sums to `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantity {
    numerator: u64,
    denominator: u64,
}

impl Quantity {
    /// A fraction, reduced to lowest terms. Returns `None` for a zero
    /// denominator.
    pub fn new(numerator: u64, denominator: u64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let divisor = gcd(numerator, denominator).max(1);
        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    /// A whole number.
    pub fn whole(n: u64) -> Self {
        Self {
            numerator: n,
            denominator: 1,
        }
    }

    /// True when the quantity is more than one, so its unit is plural.
    pub fn is_plural(self) -> bool {
        self.numerator > self.denominator
    }

    fn add(self, other: Self) -> Self {
        let denominator =
            self.denominator / gcd(self.denominator, other.denominator).max(1) * other.denominator;
        let numerator = self.numerator * (denominator / self.denominator)
            + other.numerator * (denominator / other.denominator);
        Self::new(numerator, denominator).unwrap_or(self)
    }
}

impl fmt::Display for Quantity {
    /// Whole numbers print as `2`, fractions as `1/2` and mixed numbers as
    /// `1 3/4`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.numerator / self.denominator;
        let rest = self.numerator % self.denominator;
        match (whole, rest) {
            (w, 0) => write!(f, "{w}"),
            (0, r) => write!(f, "{r}/{}", self.denominator),
            (w, r) => write!(f, "{w} {r}/{}", self.denominator),
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// One ingredient line split into its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingredient {
    /// How much, if the line says (`"Salt and pepper to taste"` doesn't).
    pub quantity: Option<Quantity>,
    /// The unit's singular name, such as `"cup"`, if the line has one.
    pub unit: Option<&'static str>,
    /// The item, lower case, without its preparation note (`"garlic"` for
    /// `"2 cloves garlic, minced"`).
    pub item: String,
}

/// Units the recipes use: (singular, plural, other spellings).
const UNITS: &[(&str, &str, &[&str])] = &[
    ("cup", "cups", &[]),
    ("tablespoon", "tablespoons", &["tbsp", "tbsp."]),
    ("teaspoon", "teaspoons", &["tsp", "tsp."]),
    ("lb", "lb", &["lbs", "pound", "pounds"]),
    ("oz", "oz", &["ounce", "ounces"]),
    ("clove", "cloves", &[]),
    ("slice", "slices", &[]),
    ("pinch", "pinches", &[]),
    ("can", "cans", &[]),
];

fn unit_named(word: &str) -> Option<&'static str> {
    let word = word.to_lowercase();
    UNITS
        .iter()
        .find(|(singular, plural, others)| {
            *singular == word || *plural == word || others.contains(&word.as_str())
        })
        .map(|(singular, _, _)| *singular)
}

fn plural_of(unit: &'static str) -> &'static str {
    UNITS
        .iter()
        .find(|(singular, _, _)| *singular == unit)
        .map_or(unit, |(_, plural, _)| *plural)
}

/// Parse a number such as `2`, `1/2` or `0.5` (decimals to two places).
fn parse_number(word: &str) -> Option<Quantity> {
    if let Some((n, d)) = word.split_once('/') {
        return Quantity::new(n.parse().ok()?, d.parse().ok()?);
    }
    if let Some((whole, frac)) = word.split_once('.') {
        if frac.is_empty() || frac.len() > 2 {
            return None;
        }
        let scale = 10u64.pow(frac.len() as u32);
        let whole: u64 = if whole.is_empty() {
            0
        } else {
            whole.parse().ok()?
        };
        return Quantity::new(whole * scale + frac.parse::<u64>().ok()?, scale);
    }
    word.parse().ok().map(Quantity::whole)
}

/// Split an ingredient line into quantity, unit and item.
///
/// ```
/// use cookbook_core::shopping::{parse_ingredient, Quantity};
///
/// let garlic = parse_ingredient("2 cloves garlic, minced");
/// assert_eq!(garlic.quantity, Some(Quantity::whole(2)));
/// assert_eq!(garlic.unit, Some("clove"));
/// assert_eq!(garlic.item, "garlic");
/// ```
pub fn parse_ingredient(line: &str) -> Ingredient {
    let mut words = line.split_whitespace().peekable();

    let mut quantity: Option<Quantity> = None;
    while let Some(n) = words.peek().and_then(|w| parse_number(w)) {
        quantity = Some(quantity.map_or(n, |q| q.add(n)));
        words.next();
    }

    let mut unit = None;
    if let Some(u) = words.peek().and_then(|w| unit_named(w)) {
        unit = Some(u);
        words.next();
        if words.peek().is_some_and(|w| w.eq_ignore_ascii_case("of")) {
            words.next();
        }
    }

    let rest = words.collect::<Vec<_>>().join(" ").to_lowercase();
    let item = rest.split(',').next().unwrap_or("").trim();
    let item = item.strip_suffix(" to taste").unwrap_or(item).trim();
    let item = if item.is_empty() {
        // A line that is only a unit ("1 pinch") or only numbers.
        line.trim().to_lowercase()
    } else {
        item.to_string()
    };

    Ingredient {
        quantity,
        unit,
        item,
    }
}

/// True for things most kitchens already have: oils, salt, pepper and
/// garlic.
pub fn is_staple(item: &str) -> bool {
    matches!(
        item,
        "salt" | "pepper" | "salt and pepper" | "black pepper" | "garlic" | "garlic cloves"
    ) || item == "oil"
        || item.ends_with(" oil")
}

/// One line of the shopping list: an item and how much of it, summed per
/// unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingItem {
    /// The item, lower case, such as `"olive oil"`.
    pub item: String,
    /// The summed quantity per unit, in the order units first appear. A
    /// `None` unit is a count (`4 eggs`).
    pub amounts: Vec<(Option<&'static str>, Quantity)>,
    /// True when at least one recipe gives no quantity (`"to taste"`).
    pub unmeasured: bool,
}

impl fmt::Display for ShoppingItem {
    /// `"14 tablespoons olive oil"`, `"1 cup + 2 tablespoons butter"`, or
    /// just `"Salt and pepper"` when no recipe gives a quantity.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.amounts.is_empty() {
            let mut chars = self.item.chars();
            return match chars.next() {
                Some(first) => write!(f, "{}{}", first.to_uppercase(), chars.as_str()),
                None => Ok(()),
            };
        }
        let amounts: Vec<String> = self
            .amounts
            .iter()
            .map(|(unit, quantity)| match unit {
                Some(unit) if quantity.is_plural() => format!("{quantity} {}", plural_of(unit)),
                Some(unit) => format!("{quantity} {unit}"),
                None => quantity.to_string(),
            })
            .collect();
        write!(f, "{} {}", amounts.join(" + "), self.item)?;
        if self.unmeasured {
            write!(f, " (and more to taste)")?;
        }
        Ok(())
    }
}

/// A week's shopping list: what to buy, and the staples it assumes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShoppingList {
    /// Items to buy, sorted by name.
    pub items: Vec<ShoppingItem>,
    /// Staples (oils, salt and pepper, garlic), sorted by name.
    pub staples: Vec<ShoppingItem>,
}

impl ShoppingList {
    /// True when there is nothing to buy and no staples.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.staples.is_empty()
    }
}

/// Sum ingredient lines into a shopping list: one line per item, with its
/// quantities summed per unit, and staples listed separately.
///
/// ```
/// use cookbook_core::shopping::build_shopping_list;
///
/// let list = build_shopping_list(["1 lb chicken", "2 tablespoons olive oil",
///     "1 lb chicken", "2 tablespoons olive oil"]);
/// assert_eq!(list.items[0].to_string(), "2 lb chicken");
/// assert_eq!(list.staples[0].to_string(), "4 tablespoons olive oil");
/// ```
pub fn build_shopping_list<I, S>(lines: I) -> ShoppingList
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut by_item: BTreeMap<String, ShoppingItem> = BTreeMap::new();
    for line in lines {
        let Ingredient {
            quantity,
            unit,
            item,
        } = parse_ingredient(line.as_ref());
        let entry = by_item.entry(item.clone()).or_insert_with(|| ShoppingItem {
            item,
            amounts: Vec::new(),
            unmeasured: false,
        });
        match quantity {
            Some(q) => match entry.amounts.iter_mut().find(|(u, _)| *u == unit) {
                Some((_, total)) => *total = total.add(q),
                None => entry.amounts.push((unit, q)),
            },
            None => entry.unmeasured = true,
        }
    }

    let (staples, items) = by_item
        .into_values()
        .partition(|line: &ShoppingItem| is_staple(&line.item));
    ShoppingList { items, staples }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: u64, d: u64) -> Quantity {
        Quantity::new(n, d).unwrap_or(Quantity::whole(0))
    }

    #[test]
    fn test_quantity_sums_and_reduces_fractions() {
        assert_eq!(q(1, 3).add(q(2, 3)), Quantity::whole(1));
        assert_eq!(q(1, 2).add(q(1, 4)), q(3, 4));
        assert_eq!(q(2, 4), q(1, 2));
        assert_eq!(Quantity::new(1, 0), None);
    }

    #[test]
    fn test_quantity_display() {
        assert_eq!(Quantity::whole(14).to_string(), "14");
        assert_eq!(q(1, 2).to_string(), "1/2");
        assert_eq!(q(7, 4).to_string(), "1 3/4");
    }

    #[test]
    fn test_parse_ingredient_with_unit() {
        let oil = parse_ingredient("2 tablespoons olive oil");
        assert_eq!(oil.quantity, Some(Quantity::whole(2)));
        assert_eq!(oil.unit, Some("tablespoon"));
        assert_eq!(oil.item, "olive oil");
    }

    #[test]
    fn test_parse_ingredient_mixed_number_and_singular_unit() {
        let flour = parse_ingredient("1 3/4 cup all-purpose flour");
        assert_eq!(flour.quantity, Some(q(7, 4)));
        assert_eq!(flour.unit, Some("cup"));
        assert_eq!(flour.item, "all-purpose flour");
    }

    #[test]
    fn test_parse_ingredient_drops_preparation_note() {
        let onion = parse_ingredient("1 onion, diced");
        assert_eq!(onion.quantity, Some(Quantity::whole(1)));
        assert_eq!(onion.unit, None);
        assert_eq!(onion.item, "onion");
    }

    #[test]
    fn test_parse_ingredient_without_quantity() {
        let seasoning = parse_ingredient("Salt and pepper to taste");
        assert_eq!(seasoning.quantity, None);
        assert_eq!(seasoning.unit, None);
        assert_eq!(seasoning.item, "salt and pepper");
    }

    #[test]
    fn test_parse_ingredient_unit_of() {
        let salt = parse_ingredient("Pinch of salt");
        assert_eq!(salt.unit, Some("pinch"));
        assert_eq!(salt.item, "salt");
    }

    #[test]
    fn test_parse_ingredient_decimal_and_abbreviation() {
        let butter = parse_ingredient("0.5 tbsp butter");
        assert_eq!(butter.quantity, Some(q(1, 2)));
        assert_eq!(butter.unit, Some("tablespoon"));
        assert_eq!(butter.item, "butter");
    }

    #[test]
    fn test_parse_ingredient_only_a_number_keeps_the_line() {
        assert_eq!(parse_ingredient("2").item, "2");
        assert_eq!(parse_ingredient("").item, "");
    }

    #[test]
    fn test_staples() {
        for staple in [
            "olive oil",
            "vegetable oil",
            "salt and pepper",
            "salt",
            "garlic",
        ] {
            assert!(is_staple(staple), "{staple}");
        }
        for item in ["chicken", "garlic bread", "boiled eggs"] {
            assert!(!is_staple(item), "{item}");
        }
    }

    #[test]
    fn test_shopping_list_sums_repeated_ingredients() {
        // Seven recipes that each use the same three staples: the old list
        // showed each once, hiding six of the seven.
        let mut lines = Vec::new();
        for _ in 0..7 {
            lines.extend([
                "2 tablespoons olive oil",
                "Salt and pepper to taste",
                "2 cloves garlic, minced",
            ]);
        }
        lines.extend(["1 lb chicken", "1 lb chicken", "2 cups broccoli"]);
        let list = build_shopping_list(lines);

        let items: Vec<String> = list.items.iter().map(ToString::to_string).collect();
        assert_eq!(items, ["2 cups broccoli", "2 lb chicken"]);
        let staples: Vec<String> = list.staples.iter().map(ToString::to_string).collect();
        assert_eq!(
            staples,
            [
                "14 cloves garlic",
                "14 tablespoons olive oil",
                "Salt and pepper"
            ]
        );
    }

    #[test]
    fn test_shopping_list_keeps_units_apart() {
        let list = build_shopping_list(["1 cup butter", "2 tablespoons butter", "1/2 cup butter"]);
        assert_eq!(list.items.len(), 1);
        assert_eq!(
            list.items[0].to_string(),
            "1 1/2 cups + 2 tablespoons butter"
        );
    }

    #[test]
    fn test_shopping_list_counts_and_unmeasured() {
        let list = build_shopping_list(["4 eggs", "2 eggs", "Olive oil", "1 tablespoon olive oil"]);
        assert_eq!(list.items[0].to_string(), "6 eggs");
        assert_eq!(
            list.staples[0].to_string(),
            "1 tablespoon olive oil (and more to taste)"
        );
    }

    #[test]
    fn test_shopping_list_empty() {
        let list = build_shopping_list(Vec::<String>::new());
        assert!(list.is_empty());
    }
}
