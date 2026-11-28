//! The cookbook's calendar: which recipe falls on which date.
//!
//! The cookbook has [`RECIPE_DAYS`] recipes, numbered 1 to 365, and a plan
//! for each ISO 8601 week (1 to 53) listing seven of them, Monday to Sunday.
//! Plans are tied to a week number, not a year, so the plan for a date is the
//! plan for its ISO week, and today's recipe is the one that plan puts on
//! today. The functions here take the plan and recipe sources as
//! [`PlanReader`] and [`RecipeReader`], so the web app passes its embedded
//! stores and tests can pass anything.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::shopping::{ShoppingList, build_shopping_list};
use crate::{PlanReader, RecipeReader};

/// The number of recipes, one for each day of a common year.
pub const RECIPE_DAYS: u32 = 365;

/// The number of weekly plans: the most ISO weeks a year can have.
pub const PLAN_WEEKS: u32 = 53;

/// How many weeks the sidebar offers plans for, starting with this one.
pub const UPCOMING_PLAN_WEEKS: usize = 4;

/// Monday-to-Sunday dates of ISO 8601 `week` in ISO year `year`, or `None`
/// when that year has no such week (only some years have a week 53).
#[must_use]
pub fn iso_week_dates(year: i32, week: u32) -> Option<[NaiveDate; 7]> {
    let monday = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)?;
    Some(std::array::from_fn(|i| {
        monday + Duration::days(i64::try_from(i).unwrap_or_default())
    }))
}

/// The dates the plan for ISO `week` covers, seen from `today`.
///
/// Plans are not tied to a year, so each one is placed on the occurrence of
/// its ISO week (last, this or next ISO year) whose Monday is closest to
/// `today`. That puts week 53 on 28 Dec 2026 to 3 Jan 2027, and week 1 seen in
/// late December on the coming January.
#[must_use]
pub fn plan_week_dates(week: u32, today: NaiveDate) -> Option<[NaiveDate; 7]> {
    let iso_year = today.iso_week().year();
    [iso_year - 1, iso_year, iso_year + 1]
        .into_iter()
        .filter_map(|year| iso_week_dates(year, week))
        .min_by_key(|dates| (dates[0] - today).num_days().abs())
}

/// The Monday that starts the ISO week containing `date`.
#[must_use]
pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday().into())
}

/// The recipe days before and after `day`, wrapping round the year: day 1
/// follows day 365, as the recipes repeat every year.
#[must_use]
pub fn adjacent_recipe_days(day: u32) -> (u32, u32) {
    let previous = if day <= 1 { RECIPE_DAYS } else { day - 1 };
    let next = if day >= RECIPE_DAYS { 1 } else { day + 1 };
    (previous, next)
}

/// A date as "Sun, 28-Dec", with no leading zero on the day.
#[must_use]
pub fn format_recipe_day(date: NaiveDate) -> String {
    date.format("%a, %-d-%b").to_string()
}

/// A week's start date as "28-Dec", with no leading zero on the day.
#[must_use]
pub fn format_week_start_date(date: NaiveDate) -> String {
    date.format("%-d-%b").to_string()
}

/// The ISO week numbers of the [`UPCOMING_PLAN_WEEKS`] weeks starting with the
/// one containing `today`, each with its Monday as "28-Dec".
#[must_use]
pub fn upcoming_plan_weeks(today: NaiveDate) -> Vec<(u32, String)> {
    let monday = week_start(today);
    (0..UPCOMING_PLAN_WEEKS)
        .map(|i| monday + Duration::weeks(i64::try_from(i).unwrap_or_default()))
        .map(|start| (start.iso_week().week(), format_week_start_date(start)))
        .collect()
}

/// The recipe day of a recipe ID such as "day-42", or `None` when it isn't
/// one of the [`RECIPE_DAYS`].
fn recipe_day(id: &str) -> Option<u32> {
    id.strip_prefix("day-")
        .and_then(|day| day.parse().ok())
        .filter(|day| (1..=RECIPE_DAYS).contains(day))
}

/// The plan for ISO `week` as `(recipe day, title)`, Monday to Sunday.
///
/// Empty when the week has no plan. A recipe the plan names that is missing
/// or has no valid day is left out.
pub fn week_recipes(
    plans: &impl PlanReader,
    recipes: &impl RecipeReader,
    week: u32,
) -> Vec<(u32, String)> {
    let Ok(plan) = plans.get_by_week(week) else {
        return Vec::new();
    };
    plan.recipe_uuids
        .iter()
        .filter_map(|uuid| recipes.get_by_uuid(uuid).ok())
        .filter_map(|recipe| Some((recipe_day(&recipe.id)?, recipe.title)))
        .collect()
}

/// The plan for ISO `week` with the date each recipe falls on, seen from
/// `today` (see [`plan_week_dates`]): `(recipe day, title, date)`.
pub fn week_plan_days(
    plans: &impl PlanReader,
    recipes: &impl RecipeReader,
    week: u32,
    today: NaiveDate,
) -> Vec<(u32, String, Option<NaiveDate>)> {
    let dates = plan_week_dates(week, today);
    week_recipes(plans, recipes, week)
        .into_iter()
        .enumerate()
        .map(|(i, (day, title))| (day, title, dates.map(|d| d[i])))
        .collect()
}

/// This week's plan for the sidebar: `(recipe day, "Mon, 23-Nov")` for each
/// day, Monday to Sunday, of the ISO week containing `today`.
pub fn week_recipe_days(
    plans: &impl PlanReader,
    recipes: &impl RecipeReader,
    today: NaiveDate,
) -> Vec<(u32, String)> {
    week_plan_days(plans, recipes, today.iso_week().week(), today)
        .into_iter()
        .filter_map(|(day, _title, date)| Some((day, format_recipe_day(date?))))
        .collect()
}

/// The recipe day (1-365) for `date`: the one its week's plan puts on it, so
/// the home page, the sidebar and the plan agree.
///
/// When the week has no plan, it falls back to the day of the year, with day
/// 366 (31 December of a leap year) reusing day 365, so it never points past
/// the last recipe.
pub fn recipe_day_for_date(
    plans: &impl PlanReader,
    recipes: &impl RecipeReader,
    date: NaiveDate,
) -> u32 {
    let index = date.weekday().num_days_from_monday() as usize;
    week_recipes(plans, recipes, date.iso_week().week())
        .get(index)
        .map_or_else(|| date.ordinal().min(RECIPE_DAYS), |(day, _title)| *day)
}

/// The shopping list for ISO `week`: every ingredient of its plan's recipes,
/// summed per item and unit, with staples apart (see [`crate::shopping`]).
/// Empty when the week has no plan.
pub fn week_shopping_list(
    plans: &impl PlanReader,
    recipes: &impl RecipeReader,
    week: u32,
) -> ShoppingList {
    let Ok(plan) = plans.get_by_week(week) else {
        return ShoppingList::default();
    };
    let ingredients = plan
        .recipe_uuids
        .iter()
        .filter_map(|uuid| recipes.get_by_uuid(uuid).ok())
        .flat_map(|recipe| recipe.ingredients);
    build_shopping_list(ingredients)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::{Plan, PlanError, PlanResult, Recipe, RecipeError, RecipeResult};
    use uuid::Uuid;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("a real date")
    }

    /// Recipes "day-1" to "day-365", each with one ingredient.
    struct Recipes(Vec<Recipe>);

    impl Recipes {
        fn new() -> Self {
            Self(
                (1..=RECIPE_DAYS)
                    .map(|day| {
                        let mut recipe = Recipe::new_with_uuid(
                            format!("day-{day}"),
                            Uuid::from_u128(u128::from(day)),
                            format!("Recipe {day}"),
                        );
                        recipe.ingredients = vec!["1 cup rice".into()];
                        recipe
                    })
                    .collect(),
            )
        }
    }

    impl RecipeReader for Recipes {
        fn get_by_id(&self, id: &str) -> RecipeResult<Recipe> {
            self.0
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or_else(|| RecipeError::NotFound(id.into()))
        }
        fn get_by_uuid(&self, uuid: &Uuid) -> RecipeResult<Recipe> {
            self.0
                .iter()
                .find(|r| r.uuid == *uuid)
                .cloned()
                .ok_or_else(|| RecipeError::NotFound(uuid.to_string()))
        }
        fn get_all(&self) -> RecipeResult<Vec<Recipe>> {
            Ok(self.0.clone())
        }
        fn exists(&self, id: &str) -> bool {
            self.0.iter().any(|r| r.id == id)
        }
    }

    /// Plans for weeks 1 to 52 listing days 7w-6 to 7w; week 53 has no plan.
    struct Plans;

    impl PlanReader for Plans {
        fn get_by_week(&self, week: u32) -> PlanResult<Plan> {
            if !(1..=52).contains(&week) {
                return Err(PlanError::NotFound(week.to_string()));
            }
            let uuids = (7 * week - 6..=7 * week).map(|d| Uuid::from_u128(u128::from(d)));
            Plan::new(week, uuids.collect())
        }
        fn get_all(&self) -> PlanResult<Vec<Plan>> {
            (1..=52).map(|week| self.get_by_week(week)).collect()
        }
        fn exists(&self, week: u32) -> bool {
            (1..=52).contains(&week)
        }
    }

    #[test]
    fn iso_week_dates_run_monday_to_sunday_and_only_some_years_have_week_53() {
        let dates = iso_week_dates(2026, 53).expect("2026 has a week 53");
        assert_eq!(dates[0], date(2026, 12, 28));
        assert_eq!(dates[6], date(2027, 1, 3));
        assert!(iso_week_dates(2027, 53).is_none(), "2027 has 52 weeks");
        assert!(iso_week_dates(2025, 53).is_none(), "2025 has 52 weeks");
    }

    #[test]
    fn plan_week_dates_pick_the_nearest_occurrence() {
        // Week 1 seen in late December 2026 is the coming January.
        assert_eq!(
            plan_week_dates(1, date(2026, 12, 20)).unwrap()[0],
            date(2027, 1, 4)
        );
        // Week 53 seen in 2027 (which has none) is the one just past.
        assert_eq!(
            plan_week_dates(53, date(2027, 3, 1)).unwrap()[0],
            date(2026, 12, 28)
        );
        // A mid-year week is in the current year.
        assert_eq!(
            plan_week_dates(26, date(2026, 11, 26)).unwrap()[0],
            date(2026, 6, 22)
        );
        for week in 1..=PLAN_WEEKS {
            assert!(
                plan_week_dates(week, date(2026, 11, 26)).is_some(),
                "week {week}"
            );
        }
    }

    #[test]
    fn week_start_is_the_monday_of_the_iso_week() {
        assert_eq!(week_start(date(2026, 11, 28)), date(2026, 11, 23));
        assert_eq!(week_start(date(2026, 11, 23)), date(2026, 11, 23));
        assert_eq!(week_start(date(2027, 1, 3)), date(2026, 12, 28));
    }

    #[test]
    fn adjacent_recipe_days_wrap_round_the_year() {
        assert_eq!(adjacent_recipe_days(10), (9, 11));
        assert_eq!(adjacent_recipe_days(1), (365, 2));
        assert_eq!(adjacent_recipe_days(365), (364, 1));
        for day in 1..=RECIPE_DAYS {
            let (previous, next) = adjacent_recipe_days(day);
            assert!((1..=RECIPE_DAYS).contains(&previous), "before {day}");
            assert!((1..=RECIPE_DAYS).contains(&next), "after {day}");
        }
    }

    #[test]
    fn dates_are_formatted_without_leading_zeros() {
        assert_eq!(format_recipe_day(date(2025, 12, 28)), "Sun, 28-Dec");
        assert_eq!(format_recipe_day(date(2026, 1, 1)), "Thu, 1-Jan");
        assert_eq!(format_week_start_date(date(2025, 12, 28)), "28-Dec");
        assert_eq!(format_week_start_date(date(2026, 1, 4)), "4-Jan");
    }

    #[test]
    fn upcoming_plan_weeks_cross_the_year_with_week_53() {
        assert_eq!(
            upcoming_plan_weeks(date(2026, 12, 20)),
            [
                (51, "14-Dec".to_string()),
                (52, "21-Dec".to_string()),
                (53, "28-Dec".to_string()),
                (1, "4-Jan".to_string()),
            ]
        );
    }

    #[test]
    fn week_recipes_follow_the_plan_monday_to_sunday() {
        let days: Vec<u32> = week_recipes(&Plans, &Recipes::new(), 2)
            .into_iter()
            .map(|(day, _)| day)
            .collect();
        assert_eq!(days, [8, 9, 10, 11, 12, 13, 14]);
        assert!(week_recipes(&Plans, &Recipes::new(), 53).is_empty());
    }

    #[test]
    fn week_recipes_leave_out_missing_recipes() {
        let mut recipes = Recipes::new();
        recipes.0.retain(|r| r.id != "day-9");
        assert_eq!(week_recipes(&Plans, &recipes, 2).len(), 6);
    }

    #[test]
    fn recipe_day_for_date_is_the_plans_recipe_for_that_date() {
        // Saturday 28 Nov 2026 is day 332 but the Saturday of week 48's plan.
        let today = date(2026, 11, 28);
        assert_eq!(today.ordinal(), 332);
        assert_eq!(recipe_day_for_date(&Plans, &Recipes::new(), today), 335);
    }

    #[test]
    fn recipe_day_for_date_falls_back_to_the_day_of_the_year_without_a_plan() {
        // 31 Dec 2026 is in week 53, which these plans don't have.
        let new_years_eve = date(2026, 12, 31);
        assert_eq!(
            recipe_day_for_date(&Plans, &Recipes::new(), new_years_eve),
            365
        );
        // Day 366 of a leap year never points past the last recipe.
        let leap = date(2032, 12, 31);
        assert_eq!(leap.ordinal(), 366);
        assert_eq!(leap.iso_week().week(), 53);
        assert_eq!(recipe_day_for_date(&Plans, &Recipes::new(), leap), 365);
    }

    #[test]
    fn week_recipe_days_label_each_day_of_this_week() {
        let days = week_recipe_days(&Plans, &Recipes::new(), date(2026, 11, 26));
        assert_eq!(days.first(), Some(&(330, "Mon, 23-Nov".to_string())));
        assert_eq!(days.last(), Some(&(336, "Sun, 29-Nov".to_string())));
        assert_eq!(days.len(), 7);
    }

    #[test]
    fn week_shopping_list_sums_the_plans_ingredients() {
        let list = week_shopping_list(&Plans, &Recipes::new(), 1);
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].to_string(), "7 cups rice");
        assert!(week_shopping_list(&Plans, &Recipes::new(), 53).is_empty());
    }
}
