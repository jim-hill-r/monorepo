use std::collections::HashMap;

/// Letter and word sequences matching blue.eel.education
pub const DEFAULT_LETTER_SEQUENCES: &[&[&str]] = &[
    &["b", "c", "d", "f"],
    &["g", "h", "l", "r", "s", "t"],
    &["a", "e", "i", "o", "u"],
    &["v", "m", "n", "p"],
    &["j", "k", "w"],
    &["q", "w", "x", "y", "z"],
];

pub const DEFAULT_WORD_SEQUENCES: &[&[&str]] = &[&["hi", "have", "fun"]];

pub const RETRY_LIMIT: usize = 3;
pub const STABILIZE_COUNT: usize = 3;
pub const REINTRODUCE_COUNT: usize = 8;

/// The level that follows `level`, or `None` after the last one.
pub fn next_level(level: &str) -> Option<&'static str> {
    match level {
        "letter" => Some("word"),
        _ => None,
    }
}

/// Phase of the practice session for a single expression
#[derive(Clone, Debug, PartialEq)]
pub enum PracticePhase {
    /// Before the session begins
    Start,
    /// Letter demonstration is playing
    Watching,
    /// User is actively tracing
    Drawing,
    /// No more expressions in this batch
    BatchComplete,
}

/// History for a single expression
#[derive(Clone, Debug, Default)]
pub struct ExpressionHistory {
    pub attempts: usize,
    pub total_attempts: usize,
    pub single_attempt_successes: usize,
    pub total_single_attempt_successes: usize,
    pub success: bool,
}

/// Application state for the practice session
#[derive(Clone, Debug)]
pub struct AppState {
    pub expression: Option<String>,
    pub active_queue: Vec<String>,
    pub stable_queue: Vec<String>,
    pub pending_queue: Vec<Vec<String>>,
    pub history: HashMap<String, ExpressionHistory>,
    pub retry_limit: usize,
    pub stabilize_count: usize,
    pub reintroduce_count: usize,
    pub consecutive_fails: usize,
    pub stale_fails: usize,
}

impl AppState {
    pub fn new(level: &str) -> Self {
        let sequences = if level == "word" {
            DEFAULT_WORD_SEQUENCES
                .iter()
                .map(|group| group.iter().map(|s| s.to_string()).collect())
                .collect()
        } else {
            DEFAULT_LETTER_SEQUENCES
                .iter()
                .map(|group| group.iter().map(|s| s.to_string()).collect())
                .collect()
        };

        Self {
            expression: None,
            active_queue: Vec::new(),
            stable_queue: Vec::new(),
            pending_queue: sequences,
            history: HashMap::new(),
            retry_limit: RETRY_LIMIT,
            stabilize_count: STABILIZE_COUNT,
            reintroduce_count: REINTRODUCE_COUNT,
            consecutive_fails: 0,
            stale_fails: 0,
        }
    }

    /// Activate the next batch from the pending queue
    pub fn activate_next_batch(&mut self) {
        if let Some(batch) = self.pending_queue.first().cloned() {
            self.pending_queue.remove(0);
            for expr in batch {
                self.active_queue.push(expr);
            }
        }
    }

    /// Move to the next expression in the active queue
    pub fn next_expression(&mut self) {
        if self.active_queue.is_empty() {
            self.activate_expressions();
        }
        if self.active_queue.is_empty() {
            self.expression = None;
        } else {
            let expr = self.active_queue.remove(0);
            self.expression = Some(expr.clone());
            self.active_queue.push(expr);
        }
    }

    /// Open the next batch, bringing back up to `reintroduce_count` stable
    /// expressions to review alongside it. Once every batch has been opened
    /// and stabilized nothing comes back, so the level ends.
    pub fn activate_expressions(&mut self) {
        if self.pending_queue.is_empty() {
            return;
        }
        self.activate_next_batch();
        let reintro_count = self.reintroduce_count.min(self.stable_queue.len());
        for _ in 0..reintro_count {
            if let Some(expr) = self.stable_queue.first().cloned() {
                self.stable_queue.remove(0);
                self.active_queue.push(expr);
            }
        }
    }

    /// Start the practice session
    pub fn start_practice(&mut self) {
        self.active_queue.clear();
        self.stable_queue.clear();
        self.history.clear();
        self.consecutive_fails = 0;
        self.stale_fails = 0;
        self.activate_expressions();
        self.next_expression();
    }

    /// Record an attempt for the current expression
    pub fn record_attempt(&mut self, success: bool) {
        if let Some(expr) = self.expression.clone() {
            let history = self.history.entry(expr.clone()).or_default();
            history.attempts += 1;
            history.total_attempts += 1;

            if success {
                history.success = true;
                self.consecutive_fails = 0;
                if history.attempts == 1 {
                    history.single_attempt_successes += 1;
                    history.total_single_attempt_successes += 1;
                }
                // Count the next time this expression comes up afresh.
                history.attempts = 0;
                // Stabilize if enough single-attempt successes
                if history.single_attempt_successes >= self.stabilize_count {
                    // Remove from active queue and add to stable
                    self.active_queue.retain(|e| e != &expr);
                    self.stable_queue.push(expr);
                }
                self.next_expression();
            } else if history.attempts >= self.retry_limit {
                self.consecutive_fails += 1;
                history.attempts = 0;
                self.next_expression();
            }
        }
    }

    /// Whether the level is done: every batch has been opened and every
    /// expression in it has stabilized, so nothing is left to practise.
    pub fn is_complete(&self) -> bool {
        self.expression.is_none() && self.active_queue.is_empty() && self.pending_queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attempts(state: &AppState, expr: &str) -> usize {
        state.history.get(expr).map_or(0, |h| h.attempts)
    }

    #[test]
    fn a_miss_keeps_the_letter_until_the_retry_limit() {
        let mut state = AppState::new("letter");
        state.start_practice();
        assert_eq!(state.expression.as_deref(), Some("b"));
        for tries in 1..RETRY_LIMIT {
            state.record_attempt(false);
            assert_eq!(state.expression.as_deref(), Some("b"));
            assert_eq!(attempts(&state, "b"), tries);
        }
        state.record_attempt(false);
        assert_eq!(state.expression.as_deref(), Some("c"));
        assert_eq!(attempts(&state, "b"), 0);
        assert_eq!(state.consecutive_fails, 1);
    }

    #[test]
    fn a_success_moves_on_and_counts_afresh_next_time() {
        let mut state = AppState::new("letter");
        state.start_practice();
        state.record_attempt(true);
        assert_eq!(state.expression.as_deref(), Some("c"));
        assert_eq!(attempts(&state, "b"), 0);
        assert_eq!(state.history["b"].single_attempt_successes, 1);
    }

    #[test]
    fn letters_traced_right_first_time_stabilize_and_open_the_next_batch() {
        let mut state = AppState::new("letter");
        state.start_practice();
        let mut steps = 0;
        while !state.pending_queue.is_empty() {
            state.record_attempt(true);
            steps += 1;
            assert!(steps < 1_000, "the next batch never opens");
        }
        assert!(state.history["b"].single_attempt_successes >= STABILIZE_COUNT);
    }

    /// Trace every expression right first time until the level ends.
    fn finish_level(state: &mut AppState) -> usize {
        state.start_practice();
        let mut steps = 0;
        while !state.is_complete() {
            state.record_attempt(true);
            steps += 1;
            assert!(steps < 1_000, "the level never ends");
        }
        steps
    }

    #[test]
    fn the_letter_level_ends_once_every_letter_is_stable() {
        let mut state = AppState::new("letter");
        finish_level(&mut state);
        assert!(state.pending_queue.is_empty());
        assert!(state.active_queue.is_empty());
        assert_eq!(state.expression, None);
        for group in DEFAULT_LETTER_SEQUENCES {
            for letter in *group {
                assert!(
                    state.stable_queue.iter().any(|e| e == letter),
                    "{letter} is not stable"
                );
            }
        }
    }

    #[test]
    fn the_word_level_ends_once_every_word_is_stable() {
        let mut state = AppState::new("word");
        let steps = finish_level(&mut state);
        assert_eq!(steps, DEFAULT_WORD_SEQUENCES[0].len() * STABILIZE_COUNT);
    }

    #[test]
    fn misses_alone_never_end_a_level() {
        let mut state = AppState::new("letter");
        state.start_practice();
        for _ in 0..RETRY_LIMIT * 50 {
            state.record_attempt(false);
        }
        assert!(!state.is_complete());
    }

    #[test]
    fn a_level_is_not_complete_before_it_starts() {
        assert!(!AppState::new("letter").is_complete());
    }

    #[test]
    fn letters_lead_to_words_and_words_to_the_end() {
        assert_eq!(next_level("letter"), Some("word"));
        assert_eq!(next_level("word"), None);
    }
}
