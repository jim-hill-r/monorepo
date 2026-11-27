//! Checks a learner's tracing against the letter patterns it should match.
//!
//! The canvas records the learner's strokes in canvas pixels, together with
//! where its guide lines are. The patterns (`patterns.json`) use their own
//! units. Both are first put in the same space: scaled so the cap line to
//! the base line is 100 units, with the cap line at y = 0 and the leftmost
//! point at x = 0, so where on the canvas the letter is drawn sideways doesn't
//! matter, but where it sits against the guide lines does.
//!
//! A tracing passes when its size, height on the guide lines, the
//! start and end of every stroke and the shape of its ink are all close to the
//! pattern's. The tolerances allow for a young learner's wobbly finger, but
//! are tight enough that no letter passes as another (tested below).

use crate::patterns::patterns_json;
use serde::Deserialize;

/// Units from the cap line to the base line once normalized.
const LINE_HEIGHT: f64 = 100.0;
/// Space between the letters of a word, in normalized units.
const LETTER_GAP: f64 = 15.0;
/// Fewest recorded points that can count as a tracing.
const MIN_POINTS: usize = 10;
/// How far a stroke's start or end may be from the pattern's, as a fraction
/// of the pattern's size (its larger side).
const POINT_TOLERANCE: f64 = 0.15;
/// How far the width or height may be from the pattern's, as a fraction of
/// the pattern's size.
const SIZE_TOLERANCE: f64 = 0.25;
/// How far the vertical centre may be from the pattern's, as a fraction of
/// the line height.
const HEIGHT_TOLERANCE: f64 = 0.2;
/// How far a point of ink may be from the other path's ink (the learner's
/// from the pattern's, and the pattern's from the learner's) and still count
/// as on it, in normalized units.
const SHAPE_TOLERANCE: f64 = 8.0;
/// The share of ink that may be off the other path's ink.
const OFF_SHAPE_SHARE: f64 = 0.1;
/// Spacing of the points the shape check compares, in normalized units.
const SAMPLE_STEP: f64 = 4.0;

/// Where the guide lines are, in the units of the points that go with them.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Boundary {
    pub cap_line: f64,
    pub base_line: f64,
}

/// One point of a path; `kind` is `start` or `end` at a stroke's ends.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
}

/// A path with its guide lines: one letter's pattern, or the canvas recording.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Path {
    pub boundary: Boundary,
    pub path: Vec<Point>,
}

/// Whether the canvas recording `recording_json` (from
/// `eelCanvas.stopRecording()`) is a good enough tracing of `expression`.
/// False when there is no pattern for it or the recording doesn't parse.
pub fn recording_matches(expression: &str, recording_json: &str) -> bool {
    let Some(patterns) =
        patterns_json(expression).and_then(|json| serde_json::from_str::<Vec<Path>>(&json).ok())
    else {
        return false;
    };
    serde_json::from_str::<Path>(recording_json)
        .is_ok_and(|recording| traced_well(&patterns, &recording))
}

/// Whether `recording` is a good enough tracing of `patterns` (one per letter
/// of the expression, in order).
pub fn traced_well(patterns: &[Path], recording: &Path) -> bool {
    if patterns.is_empty() || recording.path.len() < MIN_POINTS {
        return false;
    }
    let Some(template) = combine(patterns) else {
        return false;
    };
    let Some(user) = normalize(recording) else {
        return false;
    };
    let (t, u) = (bounds(&template), bounds(&user));
    let size = t.width.max(t.height).max(1.0);

    if (t.width - u.width).abs() > SIZE_TOLERANCE * size
        || (t.height - u.height).abs() > SIZE_TOLERANCE * size
        || (t.centre_y - u.centre_y).abs() > HEIGHT_TOLERANCE * LINE_HEIGHT
    {
        return false;
    }
    let near = |a: &[(f64, f64)], b: &[(f64, f64)]| {
        a.iter()
            .all(|p| b.iter().any(|q| distance(*p, *q) <= POINT_TOLERANCE * size))
    };
    let (t_ends, u_ends) = (stroke_ends(&template), stroke_ends(&user));
    // Both ways: every stroke of the pattern is drawn, and nothing extra is.
    if !(near(&t_ends.0, &u_ends.0)
        && near(&t_ends.1, &u_ends.1)
        && near(&u_ends.0, &t_ends.0)
        && near(&u_ends.1, &t_ends.1))
    {
        return false;
    }
    let (t_ink, u_ink) = (resample(&template), resample(&user));
    off_share(&u_ink, &t_ink) <= OFF_SHAPE_SHARE && off_share(&t_ink, &u_ink) <= OFF_SHAPE_SHARE
}

/// Points along every stroke, about `SAMPLE_STEP` apart.
fn resample(strokes: &Strokes) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    for stroke in strokes {
        let Some(&first) = stroke.first() else {
            continue;
        };
        points.push(first);
        for w in stroke.windows(2) {
            let steps = (distance(w[0], w[1]) / SAMPLE_STEP).ceil().max(1.0);
            for i in 1..=steps as usize {
                let t = i as f64 / steps;
                points.push((
                    w[0].0 + (w[1].0 - w[0].0) * t,
                    w[0].1 + (w[1].1 - w[0].1) * t,
                ));
            }
        }
    }
    points
}

/// The share of the points of `from` farther than `SHAPE_TOLERANCE` from
/// every point of `to`.
fn off_share(from: &[(f64, f64)], to: &[(f64, f64)]) -> f64 {
    if from.is_empty() || to.is_empty() {
        return 1.0;
    }
    let off = from
        .iter()
        .filter(|p| to.iter().all(|q| distance(**p, *q) > SHAPE_TOLERANCE))
        .count();
    off as f64 / from.len() as f64
}

/// A normalized point, (x, y).
type Xy = (f64, f64);
/// A normalized path: strokes of points.
type Strokes = Vec<Vec<Xy>>;

/// Splits `path` into strokes at each `start` point, scaled and moved so the
/// cap line is at y = 0, the base line at y = 100 and the leftmost point at
/// x = 0. `None` when the guide lines are unusable or there are no points.
fn normalize(path: &Path) -> Option<Strokes> {
    let span = path.boundary.base_line - path.boundary.cap_line;
    if !span.is_finite() || span <= 0.0 || path.path.is_empty() {
        return None;
    }
    let scale = LINE_HEIGHT / span;
    let mut strokes: Strokes = Vec::new();
    for p in &path.path {
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }
        let point = (p.x * scale, (p.y - path.boundary.cap_line) * scale);
        match strokes.last_mut() {
            Some(stroke) if p.kind.as_deref() != Some("start") => stroke.push(point),
            _ => strokes.push(vec![point]),
        }
    }
    let min_x = strokes
        .iter()
        .flatten()
        .map(|p| p.0)
        .fold(f64::INFINITY, f64::min);
    for p in strokes.iter_mut().flatten() {
        p.0 -= min_x;
    }
    Some(strokes)
}

/// The letters' patterns side by side, `LETTER_GAP` apart, as one path.
fn combine(patterns: &[Path]) -> Option<Strokes> {
    let mut combined: Strokes = Vec::new();
    for pattern in patterns {
        let strokes = normalize(pattern)?;
        let offset = if combined.is_empty() {
            0.0
        } else {
            bounds(&combined).max_x + LETTER_GAP
        };
        combined.extend(
            strokes
                .into_iter()
                .map(|s| s.into_iter().map(|(x, y)| (x + offset, y)).collect()),
        );
    }
    Some(combined)
}

struct Bounds {
    max_x: f64,
    width: f64,
    height: f64,
    centre_y: f64,
}

fn bounds(strokes: &Strokes) -> Bounds {
    let points = || strokes.iter().flatten();
    let min_x = points().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = points().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = points().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    Bounds {
        max_x,
        width: max_x - min_x,
        height: max_y - min_y,
        centre_y: (min_y + max_y) / 2.0,
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// The first and the last point of every stroke.
fn stroke_ends(strokes: &Strokes) -> (Vec<Xy>, Vec<Xy>) {
    let starts = strokes.iter().filter_map(|s| s.first().copied()).collect();
    let ends = strokes.iter().filter_map(|s| s.last().copied()).collect();
    (starts, ends)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(expression: &str) -> Vec<Path> {
        let json = patterns_json(expression).expect("patterns exist");
        serde_json::from_str(&json).expect("patterns parse")
    }

    /// The patterns drawn on a canvas whose cap line is at `cap`, base line
    /// at `base`, shifted right by `dx`, as the canvas would record them.
    fn drawn(expression: &str, cap: f64, base: f64, dx: f64) -> Path {
        let mut path = Vec::new();
        let mut offset = 0.0;
        for p in patterns(expression) {
            let scale = (base - cap) / (p.boundary.base_line - p.boundary.cap_line);
            let min_x = p.path.iter().map(|q| q.x).fold(f64::INFINITY, f64::min);
            let mut max_x = f64::NEG_INFINITY;
            for q in &p.path {
                let x = (q.x - min_x) * scale + offset + dx;
                max_x = max_x.max(x);
                path.push(Point {
                    x,
                    y: cap + (q.y - p.boundary.cap_line) * scale,
                    kind: q.kind.clone(),
                });
            }
            offset = max_x - dx + LETTER_GAP * (base - cap) / LINE_HEIGHT;
        }
        Path {
            boundary: Boundary {
                cap_line: cap,
                base_line: base,
            },
            path,
        }
    }

    fn map_points(path: &Path, f: impl Fn(&Point) -> Point) -> Path {
        Path {
            boundary: path.boundary.clone(),
            path: path.path.iter().map(f).collect(),
        }
    }

    #[test]
    fn the_pattern_itself_passes_at_any_canvas_size_and_place() {
        for letter in ["a", "b", "i", "k", "w", "z"] {
            let p = patterns(letter);
            assert!(
                traced_well(&p, &drawn(letter, 20.0, 220.0, 0.0)),
                "{letter}"
            );
            assert!(
                traced_well(&p, &drawn(letter, 5.0, 55.0, 300.0)),
                "{letter}"
            );
        }
    }

    #[test]
    fn a_word_passes_when_its_letters_are_traced() {
        for word in ["hi", "have", "fun"] {
            assert!(
                traced_well(&patterns(word), &drawn(word, 30.0, 330.0, 40.0)),
                "{word}"
            );
        }
    }

    #[test]
    fn a_shaky_tracing_still_passes() {
        // A 200 px line height: the hand wobbles 4 px and sits 6 px high.
        let rec = drawn("d", 20.0, 220.0, 50.0);
        let shaky = Path {
            boundary: rec.boundary.clone(),
            path: rec
                .path
                .iter()
                .enumerate()
                .map(|(i, p)| Point {
                    x: p.x + if i % 2 == 0 { 4.0 } else { -4.0 },
                    y: p.y - 6.0,
                    kind: p.kind.clone(),
                })
                .collect(),
        };
        assert!(traced_well(&patterns("d"), &shaky));
    }

    #[test]
    fn no_letter_passes_as_another() {
        let letters: Vec<String> = ('a'..='z').map(|c| c.to_string()).collect();
        for want in &letters {
            for got in &letters {
                let passed = traced_well(&patterns(want), &drawn(got, 20.0, 220.0, 0.0));
                assert_eq!(passed, want == got, "{got} traced for {want}");
            }
        }
    }

    #[test]
    fn touch_strokes_without_end_marks_pass() {
        let no_ends = map_points(&drawn("t", 20.0, 220.0, 0.0), |p| Point {
            kind: p.kind.clone().filter(|k| k == "start"),
            ..p.clone()
        });
        assert!(traced_well(&patterns("t"), &no_ends));
    }

    #[test]
    fn a_different_letter_fails() {
        assert!(!traced_well(&patterns("b"), &drawn("o", 20.0, 220.0, 0.0)));
        assert!(!traced_well(&patterns("l"), &drawn("m", 20.0, 220.0, 0.0)));
    }

    #[test]
    fn a_stroke_drawn_backwards_fails() {
        let mut backwards = drawn("l", 20.0, 220.0, 0.0);
        backwards.path.reverse();
        for p in &mut backwards.path {
            p.kind = match p.kind.as_deref() {
                Some("start") => Some("end".into()),
                Some("end") => Some("start".into()),
                _ => None,
            };
        }
        assert!(!traced_well(&patterns("l"), &backwards));
    }

    #[test]
    fn a_letter_too_small_or_on_the_wrong_lines_fails() {
        let tiny = map_points(&drawn("c", 20.0, 220.0, 0.0), |p| Point {
            x: p.x / 3.0,
            y: 20.0 + (p.y - 20.0) / 3.0,
            kind: p.kind.clone(),
        });
        assert!(!traced_well(&patterns("c"), &tiny));
        let low = map_points(&drawn("c", 20.0, 220.0, 0.0), |p| Point {
            y: p.y + 100.0,
            ..p.clone()
        });
        assert!(!traced_well(&patterns("c"), &low));
    }

    #[test]
    fn too_little_or_unusable_input_fails() {
        let p = patterns("a");
        let mut short = drawn("a", 20.0, 220.0, 0.0);
        short.path.truncate(MIN_POINTS - 1);
        assert!(!traced_well(&p, &short));
        assert!(!traced_well(&[], &drawn("a", 20.0, 220.0, 0.0)));
        let mut flat = drawn("a", 20.0, 220.0, 0.0);
        flat.boundary.base_line = flat.boundary.cap_line;
        assert!(!traced_well(&p, &flat));
        let mut nan = drawn("a", 20.0, 220.0, 0.0);
        nan.path[3].x = f64::NAN;
        assert!(!traced_well(&p, &nan));
    }

    #[test]
    fn a_recording_from_the_canvas_is_checked() {
        let rec = drawn("e", 20.0, 220.0, 10.0);
        let points: Vec<String> = rec
            .path
            .iter()
            .map(|p| match &p.kind {
                Some(k) => format!(r#"{{"x":{},"y":{},"type":"{k}"}}"#, p.x, p.y),
                None => format!(r#"{{"x":{},"y":{}}}"#, p.x, p.y),
            })
            .collect();
        let json = format!(
            r#"{{"boundary":{{"top":0,"capLine":20,"meanLine":120,"baseLine":220,"beardLine":320,"bottom":340}},"path":[{}]}}"#,
            points.join(",")
        );
        assert!(recording_matches("e", &json));
        assert!(!recording_matches("o", &json));
        assert!(!recording_matches("E", &json));
        assert!(!recording_matches("e", "not json"));
        assert!(!recording_matches("e", r#"{"boundary":{},"path":[]}"#));
    }

    #[test]
    fn a_canvas_recording_parses() {
        let json = r#"{"boundary":{"top":0,"capLine":10,"meanLine":60,"baseLine":110,"beardLine":160,"bottom":170},
            "path":[{"x":1,"y":2,"type":"start"},{"x":3,"y":4},{"x":5,"y":6,"type":"end"}]}"#;
        let rec: Path = serde_json::from_str(json).expect("parses");
        assert_eq!(rec.boundary.base_line, 110.0);
        assert_eq!(rec.path[1].kind, None);
        assert_eq!(rec.path[2].kind.as_deref(), Some("end"));
    }
}
