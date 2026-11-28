//! The home page: the intro, links to today's recipe and this week's plan, and the privacy note.

use dioxus::prelude::*;

use crate::components::MarkdownBlocks;
use crate::route::Route;
use crate::schedule::{get_current_day_of_year, get_current_week_of_year};

pub const INTRO_MD: &str = include_str!("../../../content/intro.md");

/// The one-line privacy note on the home page, for PostHog session replay
/// (public/posthog.js, decision 0001).
pub const PRIVACY_NOTE: &str = "Privacy: visits to this site may be recorded with PostHog to find bugs. What you type is never recorded, and Do Not Track turns recording off.";

#[component]
pub fn Home() -> Element {
    let today = get_current_day_of_year();
    let current_week = get_current_week_of_year();

    rsx! {
        div {
            class: "home-container",
            h1 { "The Engineer's 365 Cookbook" }

            div {
                class: "intro-description",
                MarkdownBlocks { markdown: INTRO_MD }
            }

            div {
                class: "navigation-cards",

                div {
                    class: "navigation-card recipe-card",
                    span { class: "card-icon", "🍳" }
                    h2 { "Daily Recipes" }
                    p { "Explore 365 delicious recipes - one for each day of the year. From quick weeknight dinners to special occasion dishes." }
                    Link { to: Route::Recipe { day: today }, "Browse Recipes" }
                }

                div {
                    class: "navigation-card plan-card",
                    span { class: "card-icon", "📅" }
                    h2 { "Weekly Meal Plans" }
                    p { "Get organized with complete meal plans - one for every week of the year (up to 53 weeks). Perfect for planning ahead!" }
                    Link { to: Route::Plan { week: current_week }, "View Meal Plans" }
                }
            }

            p { class: "privacy-note", "{PRIVACY_NOTE}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookbook_core::markdown::{self, Block, Inline};

    #[test]
    fn test_intro_md_content_loaded() {
        // Test that intro.md content is loaded and not empty
        assert!(!INTRO_MD.is_empty(), "INTRO_MD should not be empty");
        assert!(
            INTRO_MD.contains("Engineer's 365 Cookbook"),
            "INTRO_MD should contain the cookbook title"
        );
    }

    #[test]
    fn test_intro_md_renders_as_paragraphs_with_emphasised_title() {
        let blocks = markdown::parse(INTRO_MD);
        assert_eq!(blocks.len(), 3, "intro should have three paragraphs");
        let Some(Block::Paragraph(first)) = blocks.first() else {
            panic!("intro should start with a paragraph");
        };
        assert_eq!(
            first.first(),
            Some(&Inline::Emphasis("The Engineer's 365 Cookbook".to_string()))
        );
        for block in &blocks {
            let Block::Paragraph(spans) = block else {
                panic!("intro should only have paragraphs");
            };
            for span in spans {
                if let Inline::Text(text) = span {
                    assert!(!text.contains('*'), "literal Markdown marker in {text:?}");
                    assert!(!text.contains('\n'), "unjoined line break in {text:?}");
                }
            }
        }
    }

    #[test]
    fn test_intro_md_content_quality() {
        // Test that intro.md has expected content
        assert!(
            INTRO_MD.contains("Jim R. Hill"),
            "INTRO_MD should contain the author name"
        );
        assert!(
            INTRO_MD.contains("year"),
            "INTRO_MD should mention the year"
        );
        assert!(
            INTRO_MD.len() > 100,
            "INTRO_MD should have substantial content"
        );
    }
}
