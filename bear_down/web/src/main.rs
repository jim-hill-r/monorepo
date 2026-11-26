//! The bear_down web page: which NFL games a Bears fan should root for this
//! week. It fetches `/rooting.json` (written by the `cli`) when it loads and
//! ranks the week's games by their swing on the selected goal (decisions 0002
//! and 0007).

mod data;
mod teams;

use bear_down_core::NEGLIGIBLE_SWING;
use data::{Game, Goal, Rooting, no_pick_label, percent, swing_percent, timestamp};
use design_system_dioxus::{Stylesheets, ThemePicker};
use dioxus::prelude::*;

const STYLE: &str = include_str!("style.css");
/// The generated "bears" theme (`design_system_cli render`).
const THEME_CSS: &str = include_str!("../../design/theme.css");

/// bear_down's own theme, offered first in the picker.
fn bears() -> Option<(String, String)> {
    Some(("bears".into(), "Chicago Bears".into()))
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let rooting = use_resource(load);
    let body = match &*rooting.read() {
        None => rsx! { p { class: "status", "Loading this week's games…" } },
        Some(Err(e)) => rsx! {
            p { class: "status error", "Couldn't load this week's rooting guide. {e}" }
        },
        Some(Ok(r)) => rsx! { Guide { rooting: r.clone() } },
    };
    rsx! {
        Stylesheets { default_theme: "bears", app_css: Some(THEME_CSS.into()), custom: bears() }
        style { {STYLE} }
        main {
            header {
                h1 { "bear_down" }
                p { class: "tagline", "Which NFL games to root for this week, for Bears fans." }
            }
            {body}
            footer { class: "appearance", ThemePicker { custom: bears() } }
        }
    }
}

/// Fetches and parses `/rooting.json` from the same origin.
async fn load() -> Result<Rooting, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no browser window")?;
    let init = web_sys::RequestInit::new();
    init.set_cache(web_sys::RequestCache::NoCache);
    let response: web_sys::Response =
        JsFuture::from(window.fetch_with_str_and_init("/rooting.json", &init))
            .await
            .map_err(js)?
            .dyn_into()
            .map_err(js)?;
    if !response.ok() {
        return Err(format!("The server answered {}.", response.status()));
    }
    let text = JsFuture::from(response.text().map_err(js)?)
        .await
        .map_err(js)?
        .as_string()
        .ok_or("rooting.json is not text")?;
    Rooting::parse(&text)
}

#[component]
fn Guide(rooting: Rooting) -> Element {
    let mut goal = use_signal(Goal::default);
    let goal_now = goal();
    let odds = &rooting.bears.odds;
    let week = match rooting.week {
        Some(w) => format!("Week {w}, {} season", rooting.season),
        None => format!("{} season", rooting.season),
    };
    let record = rooting.bears.record.display();
    let games = rooting.ranked(goal_now);

    rsx! {
        section { class: "bears",
            h2 { "Chicago Bears " span { class: "record", "{record}" } }
            p { class: "week", "{week}" }
            dl { class: "odds",
                Stat { label: "Win the Super Bowl", value: percent(odds.champion) }
                Stat { label: "Make the playoffs", value: percent(odds.playoffs) }
                Stat { label: "Reach the divisional round", value: percent(odds.divisional_round) }
                Stat { label: "All four NFC North teams in the divisional round", value: percent(rooting.nfc_north_divisional) }
            }
        }

        section { class: "games",
            div { class: "toggle", role: "group", aria_label: "Rank games by",
                button {
                    class: if goal_now == Goal::SuperBowl { "active" },
                    aria_pressed: goal_now == Goal::SuperBowl,
                    onclick: move |_| goal.set(Goal::SuperBowl),
                    "Bears Super Bowl"
                }
                button {
                    class: if goal_now == Goal::Playoffs { "active" },
                    aria_pressed: goal_now == Goal::Playoffs,
                    onclick: move |_| goal.set(Goal::Playoffs),
                    "Make the playoffs"
                }
                button {
                    class: if goal_now == Goal::NfcNorth { "active" },
                    aria_pressed: goal_now == Goal::NfcNorth,
                    onclick: move |_| goal.set(Goal::NfcNorth),
                    "NFC North"
                }
            }
            p { class: "explain",
                match goal_now {
                    Goal::SuperBowl => "Games ranked by how much they move the Bears' chance of winning the Super Bowl.",
                    Goal::Playoffs => "Games ranked by how much they move the Bears' chance of making the playoffs.",
                    Goal::NfcNorth => "Games ranked by how much they move the chance that the Bears, Lions, Packers and Vikings all reach the divisional round.",
                }
            }
            if rooting.bears_eliminated() {
                p { class: "status", "The Bears have been eliminated from the playoffs, so there is nothing left to root for this season." }
            } else if games.is_empty() {
                p { class: "status", "No games this week." }
            } else {
                ol {
                    for game in games {
                        GameRow { key: "{game.id}", game: game.clone(), goal: goal_now }
                    }
                }
            }
        }

        footer {
            p { "Updated {timestamp(&rooting.generated_at)} from {rooting.simulations} simulated seasons." }
            if let Some(g) = &rooting.latest_game {
                p {
                    "Latest game included: {teams::nickname(&g.away)} {g.away_score} at {teams::nickname(&g.home)} {g.home_score} ({g.gameday})."
                }
            } else {
                p { "No games of this season are included yet." }
            }
        }
    }
}

#[component]
fn Stat(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "stat",
            dt { "{label}" }
            dd { "{value}" }
        }
    }
}

#[component]
fn GameRow(game: Game, goal: Goal) -> Element {
    let s = goal.swing(&game);
    let away = teams::nickname(&game.away);
    let home = teams::nickname(&game.home);
    let home_p = percent(game.home_win_probability);
    let (root, tier) = goal.verdict(&game, NEGLIGIBLE_SWING);
    let pick = root.as_deref().map(teams::name);
    let impact = format!("{} {}", swing_percent(s.swing), goal.odds_label());
    // Playoff odds are larger and easier to feel, so the Super Bowl view
    // shows their swing too (decision 0002).
    let playoffs = (goal == Goal::SuperBowl && game.playoffs.root_for.is_some())
        .then(|| format!("{} playoff odds", swing_percent(game.playoffs.swing)));

    rsx! {
        li { class: if pick.is_none() { "game neutral" } else { "game" },
            div { class: "matchup",
                span { class: "teams", "{away} at {home}" }
                span { class: "meta", "{game.gameday} · {home} {home_p} to win" }
            }
            div { class: "pick",
                match pick {
                    Some(team) => rsx! {
                        span { class: "root", "Root for the {team}" }
                        span { class: "impact", "{impact}" }
                        if let Some(p) = playoffs {
                            span { class: "impact secondary", "{p}" }
                        }
                    },
                    None => rsx! { span { class: "root", "{no_pick_label(tier)}" } },
                }
            }
            if !s.why.is_empty() {
                ul { class: "why",
                    for reason in s.why.iter() {
                        li { "{reason}" }
                    }
                }
            }
        }
    }
}
