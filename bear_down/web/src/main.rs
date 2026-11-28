//! The bear_down web page: which NFL games a Bears fan, or a fan of any
//! team they pick, should root for this week. It fetches the team's guide
//! (`/rooting.json` for the Bears, `/teams/<team>.json` otherwise, written by
//! the `cli`) and ranks the week's games by their swing on the selected goal
//! (decisions 0002, 0007 and 0010).

mod data;
mod feedback;
mod teams;

use data::{
    ArchiveIndex, ArchiveWeek, Game, Goal, Rooting, no_pick_label, percent, swing_percent,
    timestamp,
};
use design_system_dioxus::{Stylesheets, ThemePicker};
use dioxus::prelude::*;
use feedback::{
    ARM_ID_KEY, Arm, Ballot, MAX_NOTE, TELEMETRY_URL, Vote, exposure, exposure_key, verdict,
    vote_key,
};

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
    let team = use_signal(initial_team);
    let rooting = use_resource(move || load(team()));
    let archive = use_resource(load_archive);
    // The past week being shown, or `None` for this week's guide.
    let mut showing = use_signal(|| None::<ArchiveWeek>);
    let past = use_resource(move || async move {
        match showing() {
            Some(week) => Some((
                week,
                fetch_text(&week.path())
                    .await
                    .and_then(|t| Rooting::parse(&t)),
            )),
            None => None,
        }
    });
    let current = rooting
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(Rooting::archive_week);
    // The archive holds only the Bears' past guides (decision 0010).
    let weeks = archive
        .read()
        .as_ref()
        .and_then(|index| index.as_ref().ok())
        .filter(|_| team() == bear_down_core::BEARS)
        .map(|index| index.newest_first(current))
        .unwrap_or_default();
    let fans = teams::nickname(&team()).to_string();
    let body = match showing() {
        None => match &*rooting.read() {
            None => rsx! { p { class: "status", "Loading this week's games…" } },
            Some(Err(e)) => rsx! {
                p { class: "status error", "Couldn't load this week's rooting guide. {e}" }
            },
            Some(Ok(r)) => rsx! { Guide { key: "{r.team}", rooting: r.clone(), archived: false } },
        },
        Some(week) => {
            let label = week.label();
            let guide = match &*past.read() {
                Some(Some((loaded, Ok(r)))) if *loaded == week => {
                    rsx! { Guide { rooting: r.clone(), archived: true } }
                }
                Some(Some((loaded, Err(e)))) if *loaded == week => rsx! {
                    p { class: "status error", "Couldn't load the guide for {label}. {e}" }
                },
                _ => rsx! { p { class: "status", "Loading {label}…" } },
            };
            rsx! {
                div { class: "archived", role: "note",
                    p {
                        "You're looking at {label}, rebuilt with today's model from the results known before that week. It can differ a little from what this page showed then."
                    }
                    button { class: "back", onclick: move |_| showing.set(None), "Back to this week" }
                }
                {guide}
            }
        }
    };
    rsx! {
        Stylesheets { default_theme: "bears", app_css: Some(THEME_CSS.into()), custom: bears() }
        style { {STYLE} }
        main {
            header {
                h1 { "bear_down" }
                p { class: "tagline", "Which NFL games to root for this week, for {fans} fans." }
                TeamPicker { team, showing }
            }
            {body}
            if !weeks.is_empty() {
                WeekPicker { weeks, showing }
            }
            footer { class: "appearance", ThemePicker { custom: bears() } }
        }
    }
}

/// The team the page opens on: `?team=` if it names one, then the team this
/// browser picked last, then the Bears.
fn initial_team() -> String {
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    teams::from_query(&search)
        .or_else(|| {
            local_storage()
                .and_then(|storage| storage.get_item(teams::STORAGE_KEY).ok().flatten())
                .and_then(|team| teams::known(&team))
        })
        .unwrap_or(bear_down_core::BEARS)
        .to_string()
}

/// "Your team": a picker of all 32 teams. Choosing one loads its guide,
/// remembers it in this browser and puts it in the address, so the link
/// opens on that team (decision 0010).
#[component]
fn TeamPicker(team: Signal<String>, showing: Signal<Option<ArchiveWeek>>) -> Element {
    let selected = team();
    rsx! {
        label { class: "team-picker",
            "Your team "
            select {
                value: "{selected}",
                onchange: move |event| {
                    let Some(picked) = teams::known(&event.value()) else { return };
                    if let Some(storage) = local_storage() {
                        let _ = storage.set_item(teams::STORAGE_KEY, picked);
                    }
                    set_address(picked);
                    showing.set(None);
                    team.set(picked.to_string());
                },
                for abbr in teams::ALL {
                    option {
                        key: "{abbr}",
                        value: "{abbr}",
                        selected: selected == abbr,
                        "{teams::name(abbr)}"
                    }
                }
            }
        }
    }
}

/// Puts `team` in the address bar (`?team=DET`, nothing for the Bears)
/// without reloading, keeping any other query parameter such as `?arm=`.
fn set_address(team: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let path = location.pathname().unwrap_or_else(|_| "/".to_string());
    let search = location.search().unwrap_or_default();
    let mut params: Vec<String> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty() && !pair.starts_with("team="))
        .map(str::to_string)
        .collect();
    let team_query = teams::query(team);
    if let Some(pair) = team_query.strip_prefix('?') {
        params.push(pair.to_string());
    }
    let query = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(
            &wasm_bindgen::JsValue::NULL,
            "",
            Some(&format!("{path}{query}")),
        );
    }
}

/// "Past weeks": a picker of the archive's weeks under the guide. Choosing
/// one shows its guide in place of this week's, from the top of the page.
#[component]
fn WeekPicker(weeks: Vec<ArchiveWeek>, showing: Signal<Option<ArchiveWeek>>) -> Element {
    let selected = showing().map(ArchiveWeek::value).unwrap_or_default();
    rsx! {
        section { class: "past-weeks",
            h2 { "Past weeks" }
            label {
                "See what the guide says about an earlier week "
                select {
                    value: "{selected}",
                    onchange: move |event| {
                        showing.set(ArchiveWeek::from_value(&event.value()));
                        if let Some(window) = web_sys::window() {
                            window.scroll_to_with_x_and_y(0.0, 0.0);
                        }
                    },
                    option { value: "", selected: selected.is_empty(), "This week" }
                    for week in weeks {
                        option {
                            key: "{week.value()}",
                            value: "{week.value()}",
                            selected: selected == week.value(),
                            "{week.label()}"
                        }
                    }
                }
            }
        }
    }
}

/// Fetches and parses `team`'s guide from the same origin.
async fn load(team: String) -> Result<Rooting, String> {
    Rooting::parse(&fetch_text(&teams::guide_path(&team)).await?)
}

/// Fetches and parses `/archive/index.json`. Until the deploy builds the
/// archive there is none, and the page shows no picker.
async fn load_archive() -> Result<ArchiveIndex, String> {
    ArchiveIndex::parse(&fetch_text("/archive/index.json").await?)
}

/// Fetches a file from the same origin as text, skipping the browser cache
/// so a nightly rebuild shows at once.
async fn fetch_text(path: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no browser window")?;
    let init = web_sys::RequestInit::new();
    init.set_cache(web_sys::RequestCache::NoCache);
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(path, &init))
        .await
        .map_err(js)?
        .dyn_into()
        .map_err(js)?;
    if !response.ok() {
        return Err(format!("The server answered {}.", response.status()));
    }
    JsFuture::from(response.text().map_err(js)?)
        .await
        .map_err(js)?
        .as_string()
        .ok_or_else(|| format!("{path} is not text"))
}

/// A week's guide. `archived` is a past week from the archive: it has no
/// banner and no voting, and reports no exposures, since votes on old weeks
/// would mix into the A/B test's results. Another team's guide than the
/// Bears' has no voting or exposures either, for the same reason (decision
/// 0010), and the NFC North goal only for an NFC North team.
#[component]
fn Guide(rooting: Rooting, archived: bool) -> Element {
    let mut goal = use_signal(Goal::default);
    let nfc_north = rooting.nfc_north_team();
    let goal_now = match goal() {
        Goal::NfcNorth if !nfc_north => Goal::default(),
        goal => goal,
    };
    let bears = rooting.team == bear_down_core::BEARS;
    let nick = teams::nickname(&rooting.team).to_string();
    let arm = use_hook(current_arm);
    // One exposure per arm, week and tab in each visit.
    let season = rooting.season;
    // The week votes and exposures are for; none for a past week.
    let week_now = rooting.week.filter(|_| !archived && bears);
    use_effect(move || {
        let goal = goal();
        if let Some(week) = week_now.filter(|_| voting_allowed()) {
            let key = exposure_key(arm, season, week, goal);
            if mark_once(session_storage(), &key) {
                post(exposure(arm, season, week, goal));
            }
        }
    });
    let odds = &rooting.focus.odds;
    let week = match rooting.week {
        Some(w) => format!("Week {w}, {} season", rooting.season),
        None => format!("{} season", rooting.season),
    };
    let record = rooting.focus.record.display();
    let name = teams::name(&rooting.team).to_string();
    let mascot = if bears { "🐻" } else { "🏈" };
    let games = rooting.ranked(goal_now);

    rsx! {
        if let Some(banner) = rooting.banner.as_ref().filter(|_| !archived) {
            div { class: "banner", role: "status",
                span { class: "banner-bear", aria_hidden: "true", "{mascot}" }
                div {
                    p { class: "banner-headline", "{banner.headline}" }
                    for line in &banner.lines {
                        p { class: "banner-line", "{line}" }
                    }
                }
                span { class: "banner-bear", aria_hidden: "true", "🏈" }
            }
        }
        section { class: "bears",
            h2 { "{name} " span { class: "record", "{record}" } }
            p { class: "week", "{week}" }
            dl { class: "odds",
                Stat { label: "Win the Super Bowl", value: percent(odds.champion) }
                Stat { label: "Make the playoffs", value: percent(odds.playoffs) }
                Stat { label: "Reach the divisional round", value: percent(odds.divisional_round) }
                if nfc_north {
                    Stat { label: "All four NFC North teams in the divisional round", value: percent(rooting.nfc_north_divisional) }
                }
            }
        }

        section { class: "games",
            div { class: "toggle", role: "group", aria_label: "Rank games by",
                button {
                    class: if goal_now == Goal::SuperBowl { "active" },
                    aria_pressed: goal_now == Goal::SuperBowl,
                    onclick: move |_| goal.set(Goal::SuperBowl),
                    "{nick} Super Bowl"
                }
                button {
                    class: if goal_now == Goal::Playoffs { "active" },
                    aria_pressed: goal_now == Goal::Playoffs,
                    onclick: move |_| goal.set(Goal::Playoffs),
                    "Make the playoffs"
                }
                if nfc_north {
                    button {
                        class: if goal_now == Goal::NfcNorth { "active" },
                        aria_pressed: goal_now == Goal::NfcNorth,
                        onclick: move |_| goal.set(Goal::NfcNorth),
                        "NFC North"
                    }
                }
            }
            p { class: "explain",
                match goal_now {
                    Goal::SuperBowl => format!("Games ranked by how much they move the {nick}' chance of winning the Super Bowl."),
                    Goal::Playoffs => format!("Games ranked by how much they move the {nick}' chance of making the playoffs."),
                    Goal::NfcNorth => "Games ranked by how much they move the chance that the Bears, Lions, Packers and Vikings all reach the divisional round.".to_string(),
                }
            }
            if rooting.eliminated() {
                p { class: "status", "The {nick} have been eliminated from the playoffs, so there is nothing left to root for this season." }
            } else if games.is_empty() {
                p { class: "status", "No games this week." }
            } else {
                ol {
                    for game in games {
                        GameRow {
                            key: "{goal_now.tab()}-{game.id}",
                            game: game.clone(),
                            goal: goal_now,
                            season: rooting.season,
                            week: week_now,
                            arm,
                        }
                    }
                }
            }
        }

        footer {
            if voting_allowed() && week_now.is_some() {
                p { class: "privacy",
                    "A vote sends the game, the verdict shown, your answer and any note to this site's own server. No cookies, no sign-in, and Do Not Track turns voting off."
                }
            }
            p {
                if archived { "Rebuilt " } else { "Updated " }
                "{timestamp(&rooting.generated_at)} from {rooting.simulations} simulated seasons."
            }
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
fn GameRow(game: Game, goal: Goal, season: u16, week: Option<u8>, arm: Arm) -> Element {
    let s = goal.swing(&game);
    let away = teams::nickname(&game.away);
    let home = teams::nickname(&game.home);
    let home_p = percent(game.home_win_probability);
    let (root, tier) = goal.verdict(&game, arm.line());
    // The published explanation was written for the 0.05-point line; when
    // this arm's line turns the game into a pick, say why instead.
    let why = match root.as_deref() {
        Some(team) if s.root_for.as_deref() != Some(team) => {
            vec![small_edge(team, s.swing, s.std_error, goal, arm)]
        }
        _ => s.why.clone(),
    };
    let pick = root.as_deref().map(teams::name);
    let ballot = week.filter(|_| voting_allowed()).map(|week| Ballot {
        arm: arm.name().to_string(),
        season,
        week,
        goal,
        game: game.id.clone(),
        verdict: verdict(root.as_deref(), tier),
    });
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
            if !why.is_empty() {
                ul { class: "why",
                    for reason in why.iter() {
                        li { "{reason}" }
                    }
                }
            }
            if let Some(ballot) = ballot {
                VoteBox { ballot }
            }
        }
    }
}

/// The explanation for a pick that only this arm's lower line makes, in
/// the CLI's words ("A small edge: an Eagles win is better by 0.029 give or
/// take 0.010 Super Bowl points").
fn small_edge(team: &str, swing: f64, std_error: Option<f64>, goal: Goal, arm: Arm) -> String {
    let name = teams::nickname(team);
    let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    let points = |p: f64| percent(p).trim_end_matches('%').to_string();
    let noise = std_error
        .map(|se| format!(" give or take {}", points(se)))
        .unwrap_or_default();
    let unit = match goal {
        Goal::Playoffs => "playoff",
        Goal::SuperBowl | Goal::NfcNorth => "Super Bowl",
    };
    format!(
        "A small edge: {article} {name} win is better by {}{noise} {unit} points, over this page's {}-point line.",
        points(swing.abs()),
        arm.line_label(),
    )
}

/// This browser's A/B arm: `?arm=a` or `?arm=b` if given, otherwise from a
/// random id kept in `localStorage`, so the browser stays in one arm all
/// season. Without storage the page falls back to arm A.
fn current_arm() -> Arm {
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    if let Some(arm) = Arm::from_query(&search) {
        return arm;
    }
    let Some(storage) = local_storage() else {
        return Arm::A;
    };
    let id = match storage.get_item(ARM_ID_KEY).ok().flatten() {
        Some(id) => id,
        None => {
            let id = format!("{:013x}", (js_sys::Math::random() * 2f64.powi(52)) as u64);
            let _ = storage.set_item(ARM_ID_KEY, &id);
            id
        }
    };
    Arm::for_id(&id)
}

/// Where a game's vote stands in this browser.
#[derive(Clone, Copy, PartialEq)]
enum Voting {
    Open,
    /// Voted; the optional note box is showing.
    Voted,
    Done,
}

/// "Does this verdict look right?" with Yes, No and Confusing, then
/// "Thanks" and an optional note (BEAR-010).
#[component]
fn VoteBox(ballot: Ballot) -> Element {
    let key = vote_key(ballot.season, ballot.week, ballot.goal, &ballot.game);
    let mut state = use_signal({
        let key = key.clone();
        move || {
            if has_voted(&key) {
                Voting::Done
            } else {
                Voting::Open
            }
        }
    });
    let mut note = use_signal(String::new);
    match state() {
        Voting::Open => rsx! {
            div { class: "vote", role: "group", aria_label: "Does this verdict look right?",
                span { class: "vote-question", "Does this verdict look right?" }
                for vote in Vote::ALL {
                    button {
                        class: "vote-button",
                        onclick: {
                            let (ballot, key) = (ballot.clone(), key.clone());
                            move |_| {
                                post(ballot.vote(vote));
                                remember_vote(&key);
                                state.set(Voting::Voted);
                            }
                        },
                        "{vote.label()}"
                    }
                }
            }
        },
        Voting::Voted => rsx! {
            form {
                class: "vote thanks",
                onsubmit: {
                    let ballot = ballot.clone();
                    move |event: FormEvent| {
                        event.prevent_default();
                        if let Some(body) = ballot.note(&note()) {
                            post(body);
                        }
                        state.set(Voting::Done);
                    }
                },
                span { class: "vote-thanks", "Thanks!" }
                label { class: "vote-note",
                    "What's off? (optional)"
                    input {
                        r#type: "text",
                        maxlength: "{MAX_NOTE}",
                        value: "{note}",
                        oninput: move |event| note.set(event.value()),
                    }
                }
                button { r#type: "submit", class: "vote-button", "Send" }
            }
        },
        Voting::Done => rsx! {
            div { class: "vote thanks", span { class: "vote-thanks", "Thanks for your vote." } }
        },
    }
}

/// Whether the browser allows sending votes: Do Not Track and Global
/// Privacy Control both turn it off.
fn voting_allowed() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let navigator = window.navigator();
    let get = |target: &wasm_bindgen::JsValue, name: &str| {
        js_sys::Reflect::get(target, &name.into()).unwrap_or(wasm_bindgen::JsValue::UNDEFINED)
    };
    let dnt = get(&navigator, "doNotTrack").as_string().as_deref() == Some("1")
        || get(&window, "doNotTrack").as_string().as_deref() == Some("1");
    let gpc = get(&navigator, "globalPrivacyControl").as_bool() == Some(true);
    !(dnt || gpc)
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn session_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}

/// Records `key` in `storage` and says whether it was new. Without storage
/// it says yes, so at worst an exposure is sent twice.
fn mark_once(storage: Option<web_sys::Storage>, key: &str) -> bool {
    let Some(storage) = storage else {
        return true;
    };
    if storage.get_item(key).ok().flatten().is_some() {
        return false;
    }
    let _ = storage.set_item(key, "1");
    true
}

fn has_voted(key: &str) -> bool {
    local_storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .is_some()
}

fn remember_vote(key: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(key, "1");
    }
}

/// Posts an event to the `telemetry` Worker without waiting for it. The
/// body goes as `text/plain`, which needs no CORS preflight, and `keepalive`
/// lets it finish if the fan leaves the page.
fn post(body: String) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&body.into());
    let _ = js_sys::Reflect::set(&init, &"keepalive".into(), &true.into());
    let request = window.fetch_with_str_and_init(TELEMETRY_URL, &init);
    wasm_bindgen_futures::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(request).await;
    });
}
