use crate::canvas_js::CANVAS_JS;
use crate::patterns::demonstration_js;
use crate::state::{AppState, PracticePhase};
use crate::tracing_check;
use dioxus::document::eval;
use dioxus::prelude::*;

const CANVAS_ID: &str = "eel-canvas";

/// Core practice component, shared between letter and word pages.
/// Replicates the EelPractice + EelCanvas components from blue.eel.education.
#[component]
pub fn Practice(state: Signal<AppState>, level: String) -> Element {
    let mut phase = use_signal(|| PracticePhase::Start);
    let mut feedback = use_signal(|| "Have fun!".to_string());
    let mut canvas_initialized = use_signal(|| false);

    // Initialize the canvas JS library once
    use_effect(move || {
        if !canvas_initialized() {
            eval(CANVAS_JS);
            canvas_initialized.set(true);
        }
    });

    let is_complete = state.read().is_complete() && *phase.read() != PracticePhase::Start;

    let on_start = move |_| {
        state.write().start_practice();
        let expr = state.read().expression.clone();
        if let Some(ref letter) = expr {
            // Animate the letter demonstration from the bundled patterns
            let letter = letter.clone();
            phase.set(PracticePhase::Watching);
            eval(&format!(
                "window.eelCanvas.clear(); window.eelCanvas.init('{CANVAS_ID}', function() {{ window._eelAnimDone = true; }}); {}",
                demonstration_js(&letter)
            ));
        }
    };

    let on_ready = move |_| {
        // User indicates they're ready to try - enable recording
        phase.set(PracticePhase::Drawing);
        feedback.set("Begin!".to_string());
        eval("window.eelCanvas.clear(); window.eelCanvas.startRecording();");
    };

    let on_done = move |_| async move {
        let Some(expr) = state.read().expression.clone() else {
            return;
        };
        // Check the learner's strokes against the letter's pattern.
        let recording = eval("return window.eelCanvas.stopRecording();")
            .join::<String>()
            .await
            .ok();
        let success = recording.is_some_and(|json| tracing_check::recording_matches(&expr, &json));

        let mut state_write = state.write();
        state_write.record_attempt(success);
        // A miss under the retry limit keeps the same letter; at the limit
        // its attempts reset and the next letter comes up.
        let try_again = !success
            && state_write
                .history
                .get(&expr)
                .is_some_and(|h| h.attempts > 0);
        let next_expr = state_write.expression.clone();
        let is_done = state_write.is_complete();
        drop(state_write);

        if is_done {
            phase.set(PracticePhase::BatchComplete);
            feedback.set("Amazing!".to_string());
        } else if try_again {
            feedback.set("Try again!".to_string());
            eval("window.eelCanvas.clear(); window.eelCanvas.startRecording();");
        } else if let Some(next) = next_expr {
            feedback.set(
                if success {
                    "Great job!"
                } else {
                    "Let's try another one."
                }
                .to_string(),
            );
            phase.set(PracticePhase::Watching);
            eval(&format!(
                "window.eelCanvas.clear(); {}",
                demonstration_js(&next)
            ));
        }
    };

    let on_go = move |_| {
        // Move to word level or restart
        phase.set(PracticePhase::Start);
        state.set(AppState::new(&level));
        eval("window.eelCanvas.clear();");
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: calc(100vh - 80px); max-width: 800px; margin: 0 auto;",

            // Header card
            div {
                style: "background-color: #178CA4; color: white; padding: 16px; text-align: center; border-radius: 8px 8px 0 0;",
                div {
                    style: "font-size: 1.3rem; font-weight: 600; margin-bottom: 4px;",
                    if *phase.read() == PracticePhase::Start {
                        span { "Welcome!" }
                    } else if is_complete || *phase.read() == PracticePhase::BatchComplete {
                        span { "You are ready for the next level!" }
                    } else if let Some(ref expr) = state.read().expression.clone() {
                        span { "Try " }
                        b { "{expr}" }
                    }
                }
                div {
                    style: "font-size: 1rem; opacity: 0.9;",
                    if *phase.read() == PracticePhase::Start {
                        span { "Click start when you are ready to begin." }
                    } else if is_complete || *phase.read() == PracticePhase::BatchComplete {
                        span { "Amazing! " }
                    } else {
                        span { "{feedback}" }
                    }
                }
            }

            // Action buttons
            div {
                style: "display: flex; justify-content: center; padding: 8px; background-color: #f5f5f5;",
                if *phase.read() == PracticePhase::Start {
                    button {
                        style: "background-color: #18B7BE; color: white; border: none; padding: 10px 40px; border-radius: 6px; font-size: 1.1rem; cursor: pointer; width: 100%;",
                        onclick: on_start,
                        "Start"
                    }
                } else if *phase.read() == PracticePhase::Watching {
                    button {
                        style: "background-color: #18B7BE; color: white; border: none; padding: 10px 40px; border-radius: 6px; font-size: 1.1rem; cursor: pointer; width: 100%;",
                        onclick: on_ready,
                        "I'm Ready to Try"
                    }
                } else if *phase.read() == PracticePhase::Drawing {
                    button {
                        style: "background-color: #18B7BE; color: white; border: none; padding: 10px 40px; border-radius: 6px; font-size: 1.4rem; cursor: pointer; width: 100%;",
                        onclick: on_done,
                        "✓"
                    }
                } else if is_complete || *phase.read() == PracticePhase::BatchComplete {
                    button {
                        style: "background-color: #18B7BE; color: white; border: none; padding: 10px 40px; border-radius: 6px; font-size: 1.1rem; cursor: pointer; width: 100%;",
                        onclick: on_go,
                        "Go"
                    }
                }
            }

            // Canvas
            canvas {
                id: CANVAS_ID,
                style: "flex: 1; touch-action: none; width: 100%; background-color: #F9F7F0; border-radius: 0 0 8px 8px;",
                onmounted: move |_| {
                    eval(&format!("window.eelCanvas && window.eelCanvas.init('{CANVAS_ID}', function() {{ window._eelAnimDone = true; }});"));
                }
            }
        }
    }
}
