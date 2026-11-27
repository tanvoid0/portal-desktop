//! Run progress: the `::portal-progress` line protocol, expected step durations
//! from past runs, and the percent / ETA the UI shows. All pure.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const PREFIX: &str = "::portal-progress";
/// A step with history but no protocol never shows more than this until it ends.
const RUNNING_CAP: f64 = 0.95;
/// Successful runs of a step that feed its expected duration.
const HISTORY_RUNS: usize = 5;

/// What a step reported through the protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepProgress {
    pub done: u64,
    pub total: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// `::portal-progress <done>/<total> [label]`, the whole line after trim.
/// Anything else (sign, fraction, total 0, no space) is an ordinary log line.
pub fn parse_progress_line(line: &str) -> Option<StepProgress> {
    let rest = line.trim().strip_prefix(PREFIX)?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = rest.trim_start();
    let (frac, label) = match rest.split_once(char::is_whitespace) {
        Some((f, l)) => (f, l.trim()),
        None => (rest, ""),
    };
    let (d, t) = frac.split_once('/')?;
    let num = |s: &str| {
        (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse::<u64>().ok())
            .flatten()
    };
    let (done, total) = (num(d)?, num(t)?);
    if total == 0 {
        return None;
    }
    Some(StepProgress {
        done: done.min(total),
        total,
        label: (!label.is_empty()).then(|| label.to_string()),
    })
}

/// One past execution's steps: `(stepId, status, duration ms)`.
pub type HistoryRun = Vec<(String, String, Option<i64>)>;

/// Median duration per step over its last [`HISTORY_RUNS`] successful runs.
/// `history` is newest first. Steps with no successful run are absent.
pub fn expected_durations(history: &[HistoryRun], step_ids: &[String]) -> HashMap<String, i64> {
    let mut out = HashMap::new();
    for id in step_ids {
        let mut ds: Vec<i64> = history
            .iter()
            .filter_map(|run| {
                run.iter()
                    .find(|(s, st, _)| s == id && st == "success")
                    .and_then(|(_, _, d)| *d)
            })
            .take(HISTORY_RUNS)
            .collect();
        if ds.is_empty() {
            continue;
        }
        ds.sort_unstable();
        let n = ds.len();
        let med = if n % 2 == 1 { ds[n / 2] } else { (ds[n / 2 - 1] + ds[n / 2]) / 2 };
        out.insert(id.clone(), med);
    }
    out
}

#[derive(Clone)]
pub struct ProgressStep {
    pub step_id: String,
    pub name: String,
    pub status: String,
    pub long_running: bool,
    pub expected_ms: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub progress: Option<StepProgress>,
    /// Measured run time of a finished step.
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub step_id: String,
    pub name: String,
    pub status: String,
    pub weight: f64,
    pub fraction: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub percent: f64,
    pub eta_at: Option<String>,
    pub segments: Vec<Segment>,
}

fn ended(status: &str) -> bool {
    matches!(status, "success" | "failed" | "cancelled" | "skipped")
}

/// `run_status` freezes the bar (no elapsed-time creep, no ETA) once a run is
/// failed or cancelled.
pub fn compute_progress(steps: &[ProgressStep], run_status: &str, now: DateTime<Utc>) -> Progress {
    let active = !matches!(run_status, "failed" | "cancelled");
    let exp = |s: &ProgressStep| s.expected_ms.filter(|e| *e > 0).map(|e| e as f64);
    let known: Vec<f64> = steps.iter().filter(|s| !s.long_running).filter_map(exp).collect();
    let mean = if known.is_empty() { 1.0 } else { known.iter().sum::<f64>() / known.len() as f64 };
    let elapsed = |s: &ProgressStep| s.started_at.map_or(0.0, |t| (now - t).num_milliseconds().max(0) as f64);

    let mut segments = Vec::with_capacity(steps.len());
    let (mut wsum, mut wf) = (0.0, 0.0);
    let mut remaining = Some(0.0_f64);
    for s in steps {
        let weight = if s.long_running { 0.0 } else { exp(s).unwrap_or(mean) };
        let fraction = match s.status.as_str() {
            "success" | "skipped" => 1.0,
            "running" => match (&s.progress, exp(s)) {
                (Some(p), _) => p.done as f64 / p.total as f64,
                (None, Some(e)) if active => (elapsed(s) / e).min(RUNNING_CAP),
                _ => 0.0,
            },
            _ => 0.0,
        };
        wsum += weight;
        wf += weight * fraction;

        if !s.long_running && !ended(&s.status) {
            // ponytail: ETA is a sum; switch to max-per-group if the runner (run_execution's
            // `for group in order`) ever runs a group's steps concurrently.
            let left = if s.status == "running" {
                match (exp(s), &s.progress) {
                    (Some(e), _) => Some(e * (1.0 - fraction)),
                    (None, Some(_)) if fraction > 0.0 => Some(elapsed(s) * (1.0 - fraction) / fraction),
                    _ => None,
                }
            } else {
                exp(s)
            };
            remaining = remaining.zip(left).map(|(a, b)| a + b);
        }
        segments.push(Segment {
            step_id: s.step_id.clone(),
            name: s.name.clone(),
            status: s.status.clone(),
            weight,
            fraction,
            started_at: s.started_at.map(|t| t.to_rfc3339()),
            expected_ms: s.expected_ms,
            label: s.progress.as_ref().and_then(|p| p.label.clone()),
            duration_ms: s.duration_ms,
        });
    }
    Progress {
        percent: if wsum > 0.0 { 100.0 * wf / wsum } else { 0.0 },
        eta_at: remaining
            .filter(|_| active)
            .map(|ms| (now + Duration::milliseconds(ms.round() as i64)).to_rfc3339()),
        segments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(line: &str) -> Option<(u64, u64, Option<String>)> {
        parse_progress_line(line).map(|p| (p.done, p.total, p.label))
    }

    #[test]
    fn parses_progress_lines() {
        assert_eq!(p("::portal-progress 3/10"), Some((3, 10, None)));
        assert_eq!(p("  ::portal-progress 3/10 compiling x  \r"), Some((3, 10, Some("compiling x".into()))));
        assert_eq!(p("::portal-progress 12/10"), Some((10, 10, None)), "done clamps");
        assert_eq!(p("::portal-progress 0/1"), Some((0, 1, None)));
    }

    #[test]
    fn malformed_is_a_log_line() {
        for l in [
            "::portal-progress 3/0",
            "::portal-progress -1/5",
            "::portal-progress +1/5",
            "::portal-progress 1.5/5",
            "::portal-progress 3",
            "::portal-progress /5",
            "::portal-progress",
            "::portal-progress3/5",
            "x ::portal-progress 3/5",
            "::portal-progress 99999999999999999999/5",
            "plain text",
        ] {
            assert_eq!(p(l), None, "{l}");
        }
    }

    fn run(items: &[(&str, &str, Option<i64>)]) -> HistoryRun {
        items.iter().map(|(a, b, c)| (a.to_string(), b.to_string(), *c)).collect()
    }

    #[test]
    fn median_of_last_five_successes_only() {
        let mut h = vec![run(&[("a", "failed", Some(1))])];
        // newest first: 6 successes, only the first 5 count (10,20,30,40,50 -> 30)
        for d in [10, 20, 30, 40, 50, 9999] {
            h.push(run(&[("a", "success", Some(d))]));
        }
        let ids = vec!["a".to_string(), "b".to_string()];
        let e = expected_durations(&h, &ids);
        assert_eq!(e["a"], 30);
        assert!(!e.contains_key("b"), "no history -> absent");
        let even = [run(&[("a", "success", Some(10))]), run(&[("a", "success", Some(20))])];
        assert_eq!(expected_durations(&even, &ids)["a"], 15);
    }

    fn step(id: &str, status: &str, exp: Option<i64>) -> ProgressStep {
        ProgressStep {
            step_id: id.into(),
            name: id.into(),
            status: status.into(),
            long_running: false,
            expected_ms: exp,
            started_at: None,
            progress: None,
            duration_ms: None,
        }
    }

    fn at(s: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_000_000 + s, 0).unwrap()
    }

    #[test]
    fn first_run_equal_segments_no_eta() {
        let mut a = step("a", "success", None);
        a.started_at = Some(at(0));
        let r = compute_progress(&[a, step("b", "running", None), step("c", "pending", None)], "running", at(5));
        assert!(r.segments.iter().all(|s| s.weight == 1.0));
        assert!((r.percent - 100.0 / 3.0).abs() < 1e-9);
        assert_eq!(r.eta_at, None);
    }

    #[test]
    fn weights_fraction_and_eta_with_history() {
        // a done (1000), b running 500 of 3000, c pending (1000), d pending no history -> mean
        let mut b = step("b", "running", Some(3000));
        b.started_at = Some(at(0));
        let steps = [step("a", "success", Some(1000)), b, step("c", "pending", Some(1000))];
        let r = compute_progress(&steps, "running", at(0) + Duration::milliseconds(500));
        // fraction = 500/3000; percent = (1000 + 3000/6) / 5000
        assert!((r.percent - 30.0).abs() < 1e-6, "{}", r.percent);
        // eta = 3000*(1-1/6) + 1000 = 3500ms
        let eta = DateTime::parse_from_rfc3339(r.eta_at.as_ref().unwrap()).unwrap();
        assert_eq!((eta.with_timezone(&Utc) - (at(0) + Duration::milliseconds(500))).num_milliseconds(), 3500);

        let mut d = steps;
        d[2].expected_ms = None;
        assert_eq!(compute_progress(&d, "running", at(1)).eta_at, None, "pending without history");
        assert_eq!(compute_progress(&d, "running", at(1)).segments[2].weight, 2000.0, "mean of known");
    }

    #[test]
    fn running_caps_at_95_percent() {
        let mut b = step("b", "running", Some(1000));
        b.started_at = Some(at(0));
        assert_eq!(compute_progress(&[b], "running", at(60)).segments[0].fraction, 0.95);
    }

    #[test]
    fn protocol_wins_and_estimates_eta_without_history() {
        let mut b = step("b", "running", None);
        b.started_at = Some(at(0));
        b.progress = Some(StepProgress { done: 1, total: 4, label: Some("x".into()) });
        let r = compute_progress(&[b.clone()], "running", at(10));
        assert_eq!(r.segments[0].fraction, 0.25);
        assert_eq!(r.segments[0].label.as_deref(), Some("x"));
        // elapsed 10s, f=.25 -> 30s left
        let eta = DateTime::parse_from_rfc3339(r.eta_at.as_ref().unwrap()).unwrap();
        assert_eq!((eta.with_timezone(&Utc) - at(10)).num_seconds(), 30);
        // with history, expectedMs*(1-f)
        b.expected_ms = Some(8000);
        let r = compute_progress(&[b], "running", at(10));
        let eta = DateTime::parse_from_rfc3339(r.eta_at.as_ref().unwrap()).unwrap();
        assert_eq!((eta.with_timezone(&Utc) - at(10)).num_milliseconds(), 6000);
    }

    #[test]
    fn long_running_is_excluded() {
        let mut s = step("srv", "running", Some(5000));
        s.long_running = true;
        s.started_at = Some(at(0));
        let r = compute_progress(&[step("a", "success", Some(1000)), s], "running", at(9));
        assert_eq!(r.segments[1].weight, 0.0);
        assert_eq!(r.percent, 100.0);
        assert!(r.eta_at.is_some(), "long-running adds nothing to the ETA");
    }

    #[test]
    fn failed_run_is_frozen_without_eta() {
        let mut b = step("b", "failed", Some(1000));
        b.started_at = Some(at(0));
        let mut c = step("c", "running", Some(1000));
        c.started_at = Some(at(0));
        let steps = [step("a", "success", Some(1000)), b, c];
        let one = compute_progress(&steps, "failed", at(5));
        let two = compute_progress(&steps, "failed", at(500));
        assert_eq!(one.percent, two.percent);
        assert_eq!(one.eta_at, None);
    }
}
