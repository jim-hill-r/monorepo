//! The cookbook's calendar for today, from the embedded recipes and plans.
//!
//! Thin wrappers over [`cookbook_core::calendar`] that pass the embedded
//! stores and, where a function needs it, today's local date.

use chrono::{Datelike, Local, NaiveDate};
use cookbook_core::calendar;
use cookbook_core::shopping::ShoppingList;
use cookbook_data_md::embedded::{EmbeddedPlanStore, EmbeddedRecipeStore};

/// The recipe day (1-365) the week's plan puts on `date`.
pub fn recipe_day_for_date(date: NaiveDate) -> u32 {
    calendar::recipe_day_for_date(
        EmbeddedPlanStore::global(),
        EmbeddedRecipeStore::global(),
        date,
    )
}

/// Today's recipe day (1-365) in local time: the plan's recipe for today.
pub fn get_current_day_of_year() -> u32 {
    recipe_day_for_date(Local::now().date_naive())
}

/// The ISO 8601 week (1-53) containing today, in local time.
pub fn get_current_week_of_year() -> u32 {
    Local::now().iso_week().week()
}

/// The plan for ISO `week` as `(recipe day, title)`, Monday to Sunday.
#[cfg(test)]
pub fn get_week_recipes(week: u32) -> Vec<(u32, String)> {
    calendar::week_recipes(
        EmbeddedPlanStore::global(),
        EmbeddedRecipeStore::global(),
        week,
    )
}

/// The plan for ISO `week` with the date each recipe falls on, seen from `today`.
pub fn get_week_plan_days(week: u32, today: NaiveDate) -> Vec<(u32, String, Option<NaiveDate>)> {
    calendar::week_plan_days(
        EmbeddedPlanStore::global(),
        EmbeddedRecipeStore::global(),
        week,
        today,
    )
}

/// The sidebar's days of the week containing `today`: `(recipe day, "Mon, 23-Nov")`.
pub fn sidebar_recipe_days(today: NaiveDate) -> Vec<(u32, String)> {
    calendar::week_recipe_days(
        EmbeddedPlanStore::global(),
        EmbeddedRecipeStore::global(),
        today,
    )
}

/// The sidebar's days of this week, in local time.
pub fn get_sidebar_recipe_days() -> Vec<(u32, String)> {
    sidebar_recipe_days(Local::now().date_naive())
}

/// This week and the next three, as `(ISO week, "28-Dec")`, in local time.
pub fn get_sidebar_plan_weeks() -> Vec<(u32, String)> {
    calendar::upcoming_plan_weeks(Local::now().date_naive())
}

/// The shopping list for ISO `week`; empty when the week has no plan.
pub fn get_week_shopping_list(week: u32) -> ShoppingList {
    calendar::week_shopping_list(
        EmbeddedPlanStore::global(),
        EmbeddedRecipeStore::global(),
        week,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookbook_core::calendar::{RECIPE_DAYS as MAX_DAY_OF_YEAR, format_recipe_day};
    use cookbook_core::shopping::build_shopping_list;
    use cookbook_core::{PlanReader, RecipeReader};

    #[test]
    fn test_sidebar_recipe_days_are_monday_to_sunday_of_the_iso_week() {
        // A Thursday in week 48 of 2026: the sidebar lists Mon 23 to Sun 29 Nov,
        // not the 2024 day numbers (which would start on Thursday 26 Nov).
        let days = sidebar_recipe_days(test_date(2026, 11, 26));
        let labels: Vec<&str> = days.iter().map(|(_, label)| label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Mon, 23-Nov",
                "Tue, 24-Nov",
                "Wed, 25-Nov",
                "Thu, 26-Nov",
                "Fri, 27-Nov",
                "Sat, 28-Nov",
                "Sun, 29-Nov"
            ]
        );
        let plan_days: Vec<u32> = get_week_recipes(48).iter().map(|(d, _)| *d).collect();
        let sidebar_days: Vec<u32> = days.iter().map(|(d, _)| *d).collect();
        assert_eq!(sidebar_days, plan_days, "Links follow the week 48 plan");
    }

    #[test]
    fn test_sidebar_recipe_days_week_53() {
        // ISO 2026 has 53 weeks; week 53 runs Mon 28 Dec 2026 to Sun 3 Jan 2027.
        for today in [
            test_date(2026, 12, 28),
            test_date(2027, 1, 1),
            test_date(2027, 1, 3),
        ] {
            let days = sidebar_recipe_days(today);
            let labels: Vec<&str> = days.iter().map(|(_, label)| label.as_str()).collect();
            assert_eq!(
                labels,
                [
                    "Mon, 28-Dec",
                    "Tue, 29-Dec",
                    "Wed, 30-Dec",
                    "Thu, 31-Dec",
                    "Fri, 1-Jan",
                    "Sat, 2-Jan",
                    "Sun, 3-Jan"
                ],
                "sidebar on {today}"
            );
            let recipe_days: Vec<u32> = days.iter().map(|(d, _)| *d).collect();
            assert_eq!(recipe_days, [362, 363, 364, 365, 1, 2, 3]);
        }
    }

    #[test]
    fn test_sidebar_recipe_days_every_day_has_seven_entries() {
        // Every day across two year boundaries (one with a week 53) shows a full week.
        let mut date = test_date(2025, 12, 1);
        while date <= test_date(2027, 1, 31) {
            assert_eq!(sidebar_recipe_days(date).len(), 7, "sidebar on {date}");
            date = date.succ_opt().unwrap();
        }
    }

    #[test]
    fn test_every_week_plan_has_seven_dated_recipes() {
        let today = test_date(2026, 11, 26);
        for week in 1..=53 {
            let days = get_week_plan_days(week, today);
            assert_eq!(days.len(), 7, "week {week} should have 7 recipes");
            assert!(days.iter().all(|(_, _, date)| date.is_some()));
            assert!(!get_week_shopping_list(week).is_empty(), "week {week}");
        }
    }

    #[test]
    fn test_sidebar_plan_weeks_sorted_numerically() {
        // Test that plan weeks are returned with dates
        let weeks = get_sidebar_plan_weeks();

        // Should not be empty
        assert!(!weeks.is_empty(), "Plan weeks should not be empty");

        // Should have exactly 4 entries (4 upcoming weeks)
        assert_eq!(weeks.len(), 4, "Should show 4 upcoming weeks");

        // Verify all weeks are within valid range (1-53)
        for (week, formatted_date) in &weeks {
            assert!(
                (1..=53).contains(week),
                "Week {} should be in valid range 1-53",
                week
            );
            assert!(
                !formatted_date.is_empty(),
                "Formatted date should not be empty"
            );
            assert!(
                formatted_date.contains("-"),
                "Formatted date should contain a dash"
            );
        }
    }

    #[test]
    fn test_sidebar_recipe_days_count() {
        // Test that sidebar shows up to 7 recipe entries (one week, or fewer near year end)
        let days = get_sidebar_recipe_days();
        assert!(
            days.len() >= 1 && days.len() <= 7,
            "Sidebar should show between 1 and 7 recipe entries, found {}",
            days.len()
        );
    }

    #[test]
    fn test_sidebar_plan_weeks_count() {
        // Test that sidebar shows exactly 4 plan entries
        let weeks = get_sidebar_plan_weeks();
        assert_eq!(
            weeks.len(),
            4,
            "Sidebar should show exactly 4 plan entries, found {}",
            weeks.len()
        );
    }

    #[test]
    fn test_current_day_of_year_in_valid_range() {
        // Today's recipe day must always be a valid recipe route (1-365)
        let day = get_current_day_of_year();
        assert!(
            (1..=MAX_DAY_OF_YEAR).contains(&day),
            "Current day {} should be in valid range 1-{}",
            day,
            MAX_DAY_OF_YEAR
        );
    }

    #[test]
    fn test_recipe_day_for_date_is_the_plans_recipe_for_that_date() {
        // Saturday 28 Nov 2026 is day 332 of the year, but week 48's plan
        // puts day 335 on its Saturday, and today's recipe follows the plan.
        let date = test_date(2026, 11, 28);
        assert_eq!(date.ordinal(), 332);
        assert_eq!(recipe_day_for_date(date), 335);
    }

    #[test]
    fn test_recipe_day_for_date_end_of_leap_year_is_a_real_recipe() {
        // 31 Dec 2028 is day 366, a Sunday in ISO week 52: week 52's Sunday.
        let date = test_date(2028, 12, 31);
        assert_eq!(date.ordinal(), 366);
        assert_eq!(recipe_day_for_date(date), 364);
    }

    #[test]
    fn test_recipe_day_for_date_matches_the_sidebar_and_plan_every_day() {
        // The Recipes link and the sidebar's dated link for today must open
        // the same recipe on every date (COOK-001, COOK-006).
        for year in [2026, 2027, 2028] {
            let days = test_date(year, 1, 1)
                .iter_days()
                .take_while(|date| date.year() == year);
            for date in days {
                let index = date.weekday().num_days_from_monday() as usize;
                let sidebar = sidebar_recipe_days(date);
                let (plan_day, label) = &sidebar[index];
                assert_eq!(*label, format_recipe_day(date), "{date}");
                assert_eq!(recipe_day_for_date(date), *plan_day, "{date}");
            }
        }
    }

    #[test]
    fn test_recipe_day_for_date_always_renders_a_recipe() {
        // Every day of a leap year maps to a route the Recipe page accepts
        let days = test_date(2028, 1, 1)
            .iter_days()
            .take_while(|date| date.year() == 2028);
        let mut count = 0;
        for date in days {
            let day = recipe_day_for_date(date);
            assert!((1..=MAX_DAY_OF_YEAR).contains(&day), "{date} -> {day}");
            count += 1;
        }
        assert_eq!(count, 366);
    }

    /// Build a fixed calendar date for tests
    fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => date,
            None => panic!("invalid test date {year}-{month}-{day}"),
        }
    }

    #[test]
    fn test_current_week_of_year_in_valid_range() {
        // Test that current week is in valid range (1-53)
        let week = get_current_week_of_year();
        assert!(
            (1..=53).contains(&week),
            "Current week {} should be in valid range 1-53",
            week
        );
    }

    #[test]
    fn test_sidebar_plan_weeks_starts_from_current_week() {
        // Test that plan weeks start from current week or very close to it
        let current_week = get_current_week_of_year();
        let weeks = get_sidebar_plan_weeks();

        // Should have 4 entries
        assert_eq!(weeks.len(), 4);

        // The first week should be close to the current week.
        // Note: get_sidebar_plan_weeks() is based on calendar weeks (Sun-Sat). Around
        // year boundaries, a calendar week can span two years, so this calendar-based
        // sidebar week calculation may differ from the day-based week calculation used
        // by get_sidebar_recipe_days() / get_week_recipes(). Week 52 -> Week 1
        // transition means they're adjacent across the year boundary.
        let first_week = weeks[0].0;
        let week_diff = if first_week > current_week {
            first_week - current_week
        } else {
            current_week - first_week
        };

        // Allow difference of 1 week, or 51 weeks (which represents week 52 -> week 1 transition)
        assert!(
            week_diff <= 1 || week_diff >= 51,
            "First week {} should be within 1 week of current week {} (diff: {})",
            first_week,
            current_week,
            week_diff
        );

        // All weeks should be in valid range
        for (week, formatted_date) in &weeks {
            assert!(
                (1..=53).contains(week),
                "Week {} should be in valid range 1-53",
                week
            );
            assert!(
                !formatted_date.is_empty(),
                "Formatted date should not be empty"
            );
        }

        // Note: Weeks may wrap around year boundary (53 -> 1 or 52 -> 1), so we don't check strict consecutive order
        // Just verify they represent 4 consecutive calendar weeks
    }

    #[test]
    fn test_sidebar_recipe_days_includes_formatted_dates() {
        // Test that recipe days include formatted date strings
        let days = get_sidebar_recipe_days();

        // Should have at least 1 day, at most 7 days
        // May be fewer than 7 at year boundaries when ISO week spans two calendar years
        assert!(
            !days.is_empty() && days.len() <= 7,
            "Should have 1-7 days, got {}",
            days.len()
        );

        // Check that all entries have formatted dates
        for (day_num, formatted) in &days {
            assert!(
                *day_num >= 1 && *day_num <= 366,
                "Day number should be valid"
            );
            assert!(!formatted.is_empty(), "Formatted date should not be empty");
            assert!(
                formatted.contains("-"),
                "Formatted date should contain a dash"
            );
            assert!(
                formatted.contains(","),
                "Formatted date should contain a comma"
            );
        }
    }

    #[test]
    fn test_embedded_recipe_store_loads() {
        // Test that the embedded recipe store can be initialized
        let store = EmbeddedRecipeStore::global();
        let recipes = store.get_all().unwrap();

        // Should have exactly 365 recipes
        assert_eq!(
            recipes.len(),
            365,
            "Embedded store should contain exactly 365 recipes"
        );
    }

    #[test]
    fn test_embedded_recipe_store_get_by_id() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();

        // Test first day
        let recipe1 = store.get_by_id("day-1").unwrap();
        assert_eq!(recipe1.id, "day-1");
        assert!(!recipe1.title.is_empty());

        // Test middle day
        let recipe100 = store.get_by_id("day-100").unwrap();
        assert_eq!(recipe100.id, "day-100");
        assert!(!recipe100.title.is_empty());

        // Test last day
        let recipe365 = store.get_by_id("day-365").unwrap();
        assert_eq!(recipe365.id, "day-365");
        assert!(!recipe365.title.is_empty());
    }

    #[test]
    fn test_embedded_recipe_has_content() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();
        let recipe = store.get_by_id("day-1").unwrap();

        // Verify recipe has all expected content
        assert!(!recipe.title.is_empty(), "Recipe should have a title");
        assert!(
            recipe.description.is_some(),
            "Recipe should have a description"
        );
        assert!(
            !recipe.ingredients.is_empty(),
            "Recipe should have ingredients"
        );
        assert!(
            !recipe.instructions.is_empty(),
            "Recipe should have instructions"
        );
        assert!(
            recipe.prep_time_minutes.is_some(),
            "Recipe should have prep time"
        );
        assert!(
            recipe.cook_time_minutes.is_some(),
            "Recipe should have cook time"
        );
        assert!(recipe.servings.is_some(), "Recipe should have servings");
    }

    #[test]
    fn test_embedded_recipe_invalid_days() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();

        // Day 0 should fail
        assert!(store.get_by_id("day-0").is_err(), "Day 0 should be invalid");

        // Day 366 should fail
        assert!(
            store.get_by_id("day-366").is_err(),
            "Day 366 should be invalid"
        );
    }

    #[test]
    fn test_get_week_recipes() {
        // Test that we can get recipes for ISO week 1 using PlanReader
        let recipes = get_week_recipes(1);
        assert_eq!(
            recipes.len(),
            7,
            "Week 1 should have exactly 7 recipes from the plan"
        );

        // Days should be in ascending order
        for i in 1..recipes.len() {
            assert!(
                recipes[i].0 > recipes[i - 1].0,
                "Days should be in ascending order"
            );
        }

        // Test week 2
        let recipes_week2 = get_week_recipes(2);
        assert_eq!(
            recipes_week2.len(),
            7,
            "Week 2 should have exactly 7 recipes from the plan"
        );

        // All days should be valid (1-365)
        for (day, _) in &recipes_week2 {
            assert!(
                *day >= 1 && *day <= 365,
                "Day {} should be in range 1-365",
                day
            );
        }
    }

    #[test]
    fn test_get_week_recipes_uses_plan_reader() {
        // Test that get_week_recipes uses PlanReader to load plans
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        // Test a few different weeks
        for week in [1, 2, 10, 26, 52] {
            let plan = plan_store.get_by_week(week);
            let recipes = get_week_recipes(week);

            if let Ok(plan) = plan {
                // If plan exists, recipes should match the plan's recipe_uuids
                assert_eq!(
                    recipes.len(),
                    plan.recipe_uuids.len(),
                    "Week {} should have {} recipes matching the plan",
                    week,
                    plan.recipe_uuids.len()
                );

                // Verify the UUIDs match
                for (i, uuid) in plan.recipe_uuids.iter().enumerate() {
                    let recipe = recipe_store.get_by_uuid(uuid).unwrap();
                    let (day, title) = &recipes[i];
                    assert_eq!(
                        recipe.title, *title,
                        "Recipe title should match at position {} for week {}",
                        i, week
                    );
                    // Verify day number matches recipe ID
                    let expected_day = recipe
                        .id
                        .strip_prefix("day-")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap();
                    assert_eq!(
                        *day, expected_day,
                        "Recipe day should match at position {} for week {}",
                        i, week
                    );
                }
            } else {
                // If plan doesn't exist, recipes should be empty
                assert!(
                    recipes.is_empty(),
                    "Week {} should have no recipes when plan doesn't exist",
                    week
                );
            }
        }
    }

    #[test]
    fn test_get_week_recipes_plan_consistency() {
        // Test that plans have consistent structure (7 days, consecutive)
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        // Test week 1 plan
        let plan1 = plan_store.get_by_week(1).expect("Week 1 plan should exist");
        assert_eq!(plan1.recipe_uuids.len(), 7, "Week 1 should have 7 UUIDs");

        // Verify the UUIDs correspond to recipes for days 1-7
        for (i, uuid) in plan1.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let expected_day = i + 1;
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id, expected_day as u32,
                "Week 1 UUID at position {} should be for day {}",
                i, expected_day
            );
        }

        // Test week 2 plan
        let plan2 = plan_store.get_by_week(2).expect("Week 2 plan should exist");
        assert_eq!(plan2.recipe_uuids.len(), 7, "Week 2 should have 7 UUIDs");

        // Verify the UUIDs correspond to recipes for days 8-14
        for (i, uuid) in plan2.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let expected_day = i + 8;
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id, expected_day as u32,
                "Week 2 UUID at position {} should be for day {}",
                i, expected_day
            );
        }

        // Test a mid-year week (week 26) should have 7 consecutive days
        let plan26 = plan_store
            .get_by_week(26)
            .expect("Week 26 plan should exist");
        assert_eq!(plan26.recipe_uuids.len(), 7, "Week 26 should have 7 UUIDs");

        // Verify UUIDs correspond to consecutive days
        let first_recipe = recipe_store.get_by_uuid(&plan26.recipe_uuids[0]).unwrap();
        let first_day = first_recipe
            .id
            .strip_prefix("day-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap();

        for (i, uuid) in plan26.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id,
                first_day + i as u32,
                "Days within a week should be consecutive"
            );
        }

        // Test that recipes are retrieved for the correct days
        let recipes = get_week_recipes(26);
        assert_eq!(
            recipes.len(),
            plan26.recipe_uuids.len(),
            "Should have recipes for all UUIDs in the week"
        );
        for (i, uuid) in plan26.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).unwrap();
            let (day, title) = &recipes[i];
            let expected_day = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(*day, expected_day, "Recipe day should match UUID's recipe");
            assert_eq!(
                *title, recipe.title,
                "Recipe title should match UUID's recipe"
            );
        }
    }

    #[test]
    fn test_get_week_recipes_titles_loaded() {
        // Test that recipe titles are loaded correctly
        let recipes = get_week_recipes(1);

        for (day, title) in &recipes {
            assert!(
                !title.is_empty(),
                "Recipe title for day {} should not be empty",
                day
            );
        }
    }

    #[test]
    fn test_sidebar_recipes_match_current_week_plan() {
        // Test that sidebar recipes match the Plan component for the current week
        let current_week = get_current_week_of_year();
        let sidebar_days = get_sidebar_recipe_days();
        let plan_recipes = get_week_recipes(current_week);

        // Both should have the same number of recipes (7 or fewer)
        assert_eq!(
            sidebar_days.len(),
            plan_recipes.len(),
            "Sidebar and plan should have the same number of recipes"
        );

        // The day numbers should match exactly
        for i in 0..sidebar_days.len() {
            assert_eq!(
                sidebar_days[i].0, plan_recipes[i].0,
                "Sidebar day {} should match plan day {} for position {}",
                sidebar_days[i].0, plan_recipes[i].0, i
            );
        }
    }

    #[test]
    fn test_sidebar_recipes_use_plan_reader() {
        // Test that sidebar uses PlanReader via get_week_recipes
        let sidebar_days = get_sidebar_recipe_days();
        let current_week = get_current_week_of_year();

        // Get the plan for the current week
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();
        let plan = plan_store.get_by_week(current_week);

        if let Ok(plan) = plan {
            // Sidebar should have the same number of days as the plan
            assert_eq!(
                sidebar_days.len(),
                plan.recipe_uuids.len(),
                "Sidebar should have {} UUIDs from the plan for ISO week {} but has {}",
                plan.recipe_uuids.len(),
                current_week,
                sidebar_days.len()
            );

            // First sidebar day should match the first recipe's day from the plan
            if !plan.recipe_uuids.is_empty() {
                let first_recipe = recipe_store.get_by_uuid(&plan.recipe_uuids[0]).unwrap();
                let first_day = first_recipe
                    .id
                    .strip_prefix("day-")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap();
                assert_eq!(
                    sidebar_days[0].0, first_day,
                    "Sidebar should start at day {} for ISO week {}",
                    first_day, current_week
                );
            }

            // Verify all days match the plan's recipe UUIDs
            for (i, (day, _)) in sidebar_days.iter().enumerate() {
                let recipe = recipe_store.get_by_uuid(&plan.recipe_uuids[i]).unwrap();
                let expected_day = recipe
                    .id
                    .strip_prefix("day-")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap();
                assert_eq!(
                    *day, expected_day,
                    "Sidebar day at position {} should be {} but got {}",
                    i, expected_day, day
                );
            }
        }
    }

    #[test]
    fn test_get_week_shopping_list_returns_ingredients() {
        // Test that shopping list returns ingredients from week 1
        let shopping_list = get_week_shopping_list(1);

        // Shopping list should not be empty for week 1
        assert!(!shopping_list.is_empty(), "Week 1 should have ingredients");
    }

    #[test]
    fn test_get_week_shopping_list_is_sorted() {
        // Items and staples are each sorted by name.
        let shopping_list = get_week_shopping_list(1);
        for lines in [&shopping_list.items, &shopping_list.staples] {
            let names: Vec<&str> = lines.iter().map(|line| line.item.as_str()).collect();
            let mut sorted = names.clone();
            sorted.sort_unstable();
            assert_eq!(names, sorted, "Shopping list should be sorted");
        }
    }

    #[test]
    fn test_get_week_shopping_list_one_line_per_item() {
        // An item used by several recipes appears once, in one of the two
        // lists, in every week.
        for week in 1..=53 {
            let shopping_list = get_week_shopping_list(week);
            let names: Vec<&str> = shopping_list
                .items
                .iter()
                .chain(&shopping_list.staples)
                .map(|line| line.item.as_str())
                .collect();
            let unique: std::collections::HashSet<_> = names.iter().collect();
            assert_eq!(names.len(), unique.len(), "week {week} repeats an item");
        }
    }

    #[test]
    fn test_get_week_shopping_list_adds_up() {
        // Week 48 has seven recipes, each with 2 tablespoons olive oil, 2
        // cloves garlic and salt and pepper. The old list showed each once.
        let shopping_list = get_week_shopping_list(48);
        let staples: Vec<String> = shopping_list
            .staples
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            staples,
            [
                "14 cloves garlic",
                "14 tablespoons olive oil",
                "Salt and pepper"
            ]
        );
        let items: Vec<String> = shopping_list
            .items
            .iter()
            .map(ToString::to_string)
            .collect();
        for expected in [
            "3 cups quinoa",
            "2 cups couscous",
            "2 lb shrimp",
            "6 cups mushrooms",
            "1 lb beef",
        ] {
            assert!(
                items.iter().any(|line| line == expected),
                "{expected} in {items:?}"
            );
        }
        assert!(
            !items.iter().any(|line| line.contains("olive oil")),
            "staples are not in the items to buy"
        );
    }

    #[test]
    fn test_get_week_shopping_list_valid_weeks() {
        // Test that shopping list works for various valid weeks
        let shopping_list_week_1 = get_week_shopping_list(1);
        let shopping_list_week_26 = get_week_shopping_list(26);
        let _shopping_list_week_52 = get_week_shopping_list(52);
        let _shopping_list_week_53 = get_week_shopping_list(53);

        // All should return some ingredients (assuming recipes have ingredients)
        // Note: Week 52 and 53 might be empty or have fewer recipes since the recipe
        // system is based on days 1-365 (MAX_DAY_OF_YEAR), and these weeks may
        // extend beyond day 365 depending on the year
        assert!(
            !shopping_list_week_1.is_empty(),
            "Week 1 should have ingredients"
        );
        assert!(
            !shopping_list_week_26.is_empty(),
            "Week 26 should have ingredients"
        );
    }

    #[test]
    fn test_get_week_shopping_list_invalid_weeks() {
        // Test that shopping list returns empty for invalid weeks
        let shopping_list_week_0 = get_week_shopping_list(0);
        let shopping_list_week_54 = get_week_shopping_list(54);
        let shopping_list_week_100 = get_week_shopping_list(100);

        assert!(
            shopping_list_week_0.is_empty(),
            "Week 0 should return empty list"
        );
        assert!(
            shopping_list_week_54.is_empty(),
            "Week 54 should return empty list"
        );
        assert!(
            shopping_list_week_100.is_empty(),
            "Week 100 should return empty list"
        );
    }

    #[test]
    fn test_get_week_shopping_list_uses_plan_reader() {
        // The list is built from the ingredients of the plan's recipe UUIDs.
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        let plan = plan_store.get_by_week(1).expect("Week 1 plan should exist");
        let expected = build_shopping_list(
            plan.recipe_uuids
                .iter()
                .filter_map(|uuid| recipe_store.get_by_uuid(uuid).ok())
                .flat_map(|recipe| recipe.ingredients),
        );

        assert_eq!(
            get_week_shopping_list(1),
            expected,
            "Shopping list should match ingredients from plan's recipe UUIDs"
        );
    }
}
