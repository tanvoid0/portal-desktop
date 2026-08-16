//! Line-level diff + hunk rebuild, backing the Cursor-style change review.
//!
//! A file edit is applied to disk immediately, but we also snapshot the
//! `before` content and compute the change as a list of [`Hunk`]s (contiguous
//! blocks of removed/added lines). The UI can then accept/reject each hunk;
//! [`rebuild`] deterministically reconstructs the file from `before` + each
//! hunk's `accepted` flag, so rejecting a hunk restores exactly its old lines.
//!
//! Lines are split with `split_inclusive('\n')` so newlines are preserved and
//! an all-accepted rebuild reproduces the `after` content byte-for-byte.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hunk {
    /// Position in this hunk's ordering (stable id for accept/reject calls).
    pub index: usize,
    /// Index into `before`'s lines where this hunk's removed lines start.
    pub before_start: usize,
    /// Lines removed from `before` (may be empty for a pure insertion).
    pub before_lines: Vec<String>,
    /// Lines added in `after` (may be empty for a pure deletion).
    pub after_lines: Vec<String>,
    /// Whether the added lines are kept (true) or reverted to `before` (false).
    pub accepted: bool,
}

enum Op {
    Common,
    Del,
    Add,
}

/// Compute the change blocks between `before` and `after`.
pub fn compute_hunks(before: &str, after: &str) -> Vec<Hunk> {
    let a: Vec<&str> = before.split_inclusive('\n').collect();
    let b: Vec<&str> = after.split_inclusive('\n').collect();
    let n = a.len();
    let m = b.len();

    // LCS length table.
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    // Backtrack into a flat op sequence.
    let mut ops: Vec<Op> = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push(Op::Common);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(Op::Del);
            i += 1;
        } else {
            ops.push(Op::Add);
            j += 1;
        }
    }
    while i < n {
        ops.push(Op::Del);
        i += 1;
    }
    while j < m {
        ops.push(Op::Add);
        j += 1;
    }

    // Group consecutive Del/Add ops into hunks.
    let mut hunks: Vec<Hunk> = Vec::new();
    let (mut bi, mut bj) = (0usize, 0usize); // pointers into a / b
    let mut cur: Option<Hunk> = None;
    for op in ops {
        match op {
            Op::Common => {
                if let Some(h) = cur.take() {
                    hunks.push(h);
                }
                bi += 1;
                bj += 1;
            }
            Op::Del => {
                let h = cur.get_or_insert_with(|| Hunk {
                    index: 0,
                    before_start: bi,
                    before_lines: Vec::new(),
                    after_lines: Vec::new(),
                    accepted: true,
                });
                h.before_lines.push(a[bi].to_string());
                bi += 1;
            }
            Op::Add => {
                let h = cur.get_or_insert_with(|| Hunk {
                    index: 0,
                    before_start: bi,
                    before_lines: Vec::new(),
                    after_lines: Vec::new(),
                    accepted: true,
                });
                h.after_lines.push(b[bj].to_string());
                bj += 1;
            }
        }
    }
    if let Some(h) = cur.take() {
        hunks.push(h);
    }
    for (idx, h) in hunks.iter_mut().enumerate() {
        h.index = idx;
    }
    hunks
}

/// Reconstruct file content from `before` applying each hunk's decision.
pub fn rebuild(before: &str, hunks: &[Hunk]) -> String {
    let a: Vec<&str> = before.split_inclusive('\n').collect();
    let mut out = String::new();
    let mut b = 0usize;
    for h in hunks {
        while b < h.before_start && b < a.len() {
            out.push_str(a[b]);
            b += 1;
        }
        if h.accepted {
            for l in &h.after_lines {
                out.push_str(l);
            }
        } else {
            for l in &h.before_lines {
                out.push_str(l);
            }
        }
        b += h.before_lines.len();
    }
    while b < a.len() {
        out.push_str(a[b]);
        b += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_accepted_reproduces_after() {
        let before = "a\nb\nc\n";
        let after = "a\nB\nc\nd\n";
        let hunks = compute_hunks(before, after);
        assert_eq!(rebuild(before, &hunks), after);
    }

    #[test]
    fn all_rejected_reproduces_before() {
        let before = "a\nb\nc\n";
        let after = "a\nB\nc\nd\n";
        let mut hunks = compute_hunks(before, after);
        for h in &mut hunks {
            h.accepted = false;
        }
        assert_eq!(rebuild(before, &hunks), before);
    }

    #[test]
    fn reject_one_hunk_keeps_others() {
        let before = "keep\nold1\nmid\nold2\n";
        let after = "keep\nnew1\nmid\nnew2\n";
        let mut hunks = compute_hunks(before, after);
        assert_eq!(hunks.len(), 2);
        hunks[0].accepted = false; // revert first change only
        let out = rebuild(before, &hunks);
        assert_eq!(out, "keep\nold1\nmid\nnew2\n");
    }

    #[test]
    fn pure_insertion() {
        let before = "a\nc\n";
        let after = "a\nb\nc\n";
        let hunks = compute_hunks(before, after);
        assert_eq!(rebuild(before, &hunks), after);
        let mut rej = hunks.clone();
        rej[0].accepted = false;
        assert_eq!(rebuild(before, &rej), before);
    }
}
