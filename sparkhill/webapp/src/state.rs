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

    /// Determine what to activate next
    pub fn activate_expressions(&mut self) {
        let reintro_count = if !self.pending_queue.is_empty() {
            self.activate_next_batch();
            self.reintroduce_count.min(self.stable_queue.len())
        } else {
            self.stable_queue.len()
        };
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

    /// Whether the session is complete (no active, stable, or pending)
    pub fn is_complete(&self) -> bool {
        self.expression.is_none() && self.active_queue.is_empty() && self.pending_queue.is_empty()
    }
}
