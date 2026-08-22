//! Duplicate clustering and risk scoring over a discovered set.

use std::collections::HashMap;

use super::scan::{
    ActivityStatus, DiscoveredProject, DuplicateCluster, ProjectGitSummary, RiskLevel,
};

/// Strips the suffixes people use when copying a project, so `care-flex` and
/// `care-flex-main` land in the same bucket.
fn name_stem(name: &str) -> String {
    let lower = name.to_lowercase();
    let suffixes = [
        "old", "backup", "bak", "copy", "orig", "original", "new", "final", "temp", "tmp",
        "example", "demo", "clone", "main", "master", "fresh",
    ];
    let mut base = lower
        .as_str()
        .trim_end_matches(|c: char| c.is_ascii_digit());
    loop {
        let trimmed = base.trim_end_matches(['-', '_', ' ', '.']);
        let mut matched = false;
        for s in suffixes {
            if let Some(rest) = trimmed.strip_suffix(s) {
                // Only strip when it is a real suffix, not the whole name.
                if !rest.is_empty() && rest.ends_with(['-', '_', ' ', '.']) {
                    base = rest;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            base = trimmed;
            break;
        }
    }
    base.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

fn remote_key(remote: &str) -> String {
    remote
        .to_lowercase()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string()
}

/// Groups projects that are probably the same thing: identical git remote, or a
/// near-identical name after copy-suffixes are removed.
pub fn cluster_duplicates(projects: &[DiscoveredProject]) -> Vec<DuplicateCluster> {
    let mut by_remote: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_name: HashMap<String, Vec<String>> = HashMap::new();

    for p in projects {
        if let Some(r) = p.git.remote.as_deref() {
            if !r.is_empty() {
                by_remote
                    .entry(remote_key(r))
                    .or_default()
                    .push(p.path.clone());
            }
        }
        let stem = name_stem(&p.name);
        if !stem.is_empty() {
            by_name.entry(stem).or_default().push(p.path.clone());
        }
    }

    let mut clusters: Vec<DuplicateCluster> = Vec::new();
    let mut remote_keys: Vec<_> = by_remote.into_iter().filter(|(_, v)| v.len() > 1).collect();
    remote_keys.sort_by(|a, b| a.0.cmp(&b.0));
    for (k, members) in remote_keys {
        clusters.push(DuplicateCluster {
            kind: "same-remote".into(),
            key: k.rsplit('/').next().unwrap_or(&k).to_string(),
            members,
        });
    }
    let mut name_keys: Vec<_> = by_name.into_iter().filter(|(_, v)| v.len() > 1).collect();
    name_keys.sort_by(|a, b| a.0.cmp(&b.0));
    for (k, members) in name_keys {
        clusters.push(DuplicateCluster {
            kind: "similar-name".into(),
            key: k,
            members,
        });
    }
    clusters
}

/// Writes cluster membership back onto each project.
pub fn apply_clusters(
    mut projects: Vec<DiscoveredProject>,
    clusters: &[DuplicateCluster],
) -> Vec<DiscoveredProject> {
    let mut index: HashMap<&str, Vec<String>> = HashMap::new();
    for c in clusters {
        for m in &c.members {
            index.entry(m.as_str()).or_default().push(c.key.clone());
        }
    }
    for p in &mut projects {
        if let Some(keys) = index.get(p.path.as_str()) {
            let mut k = keys.clone();
            k.sort();
            k.dedup();
            p.duplicate_of = k;
        }
    }
    projects
}

/// Severity is about *where the work lives*, not how messy it is.
///
/// - `high` — the work exists only on this disk: unpushed commits, or history
///   with no remote at all. A disk failure loses it.
/// - `med`  — recoverable but easy to forget: uncommitted files, stashes, or
///   active work with no version control.
/// - `low`  — a hint worth a look, nothing urgent.
pub fn score_risk(
    git: &ProjectGitSummary,
    module_count: usize,
    status: ActivityStatus,
) -> (Vec<String>, RiskLevel) {
    let mut risk = Vec::new();
    let local_only = git.repos > 0 && git.no_remote == git.repos && git.commits > 0;

    if git.ahead > 0 {
        risk.push(format!("{} unpushed", git.ahead));
    }
    if local_only {
        risk.push("no remote".to_string());
    }
    if git.dirty > 0 {
        risk.push(format!("{} uncommitted", git.dirty));
    }
    if git.stash > 0 {
        risk.push(format!("{} stash", git.stash));
    }
    if git.repos == 0 && module_count > 0 {
        risk.push("not in git".to_string());
    }

    let level = if git.ahead > 0 || local_only {
        RiskLevel::High
    } else if git.dirty > 0
        || git.stash > 0
        || (git.repos == 0 && matches!(status, ActivityStatus::Active | ActivityStatus::Recent))
    {
        RiskLevel::Med
    } else if !risk.is_empty() {
        RiskLevel::Low
    } else {
        RiskLevel::None
    };

    (risk, level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(repos: i64, no_remote: i64, commits: i64, ahead: i64) -> ProjectGitSummary {
        ProjectGitSummary {
            repos,
            no_remote,
            commits,
            ahead,
            ..Default::default()
        }
    }

    #[test]
    fn unpushed_work_is_high_risk() {
        let (risk, level) = score_risk(&summary(1, 0, 10, 5), 1, ActivityStatus::Active);
        assert_eq!(level, RiskLevel::High);
        assert!(risk.iter().any(|r| r.contains("unpushed")));
    }

    #[test]
    fn history_with_no_remote_is_high_risk() {
        let (risk, level) = score_risk(&summary(1, 1, 44, -1), 1, ActivityStatus::Stale);
        assert_eq!(level, RiskLevel::High);
        assert!(risk.iter().any(|r| r == "no remote"));
    }

    #[test]
    fn a_pushed_clean_repo_carries_no_risk() {
        let (risk, level) = score_risk(&summary(1, 0, 10, 0), 1, ActivityStatus::Active);
        assert_eq!(level, RiskLevel::None);
        assert!(risk.is_empty());
    }

    #[test]
    fn empty_repo_without_remote_is_not_high_risk() {
        // No commits means there is nothing to lose yet.
        let (_, level) = score_risk(&summary(1, 1, 0, -1), 1, ActivityStatus::Stale);
        assert_ne!(level, RiskLevel::High);
    }

    #[test]
    fn copy_suffixes_collapse_to_one_stem() {
        assert_eq!(name_stem("care-flex"), name_stem("care-flex-main"));
        assert_eq!(name_stem("portal_desktop"), name_stem("portal_desktop_old"));
        assert_eq!(name_stem("daery"), name_stem("daery2"));
    }

    #[test]
    fn distinct_names_do_not_collapse() {
        assert_ne!(name_stem("portal_web"), name_stem("portal_desktop"));
    }

    #[test]
    fn same_remote_clusters_across_categories() {
        let mk = |path: &str, remote: Option<&str>, name: &str| DiscoveredProject {
            path: path.into(),
            abs: String::new(),
            name: name.into(),
            area: String::new(),
            bucket: String::new(),
            module_count: 1,
            modules: vec![],
            activity: 0,
            age_days: None,
            status: ActivityStatus::Unknown,
            git: ProjectGitSummary {
                remote: remote.map(|s| s.to_string()),
                ..Default::default()
            },
            langs: vec![],
            regenerable: vec![],
            regenerable_bytes: -1,
            description: None,
            risk: vec![],
            risk_level: RiskLevel::None,
            duplicate_of: vec![],
        };
        let projects = vec![
            mk(
                "p/portfolio/portfolio",
                Some("https://github.com/x/tanvoid0.git"),
                "portfolio",
            ),
            mk(
                "p/javascript/tanvoid0",
                Some("https://github.com/x/tanvoid0"),
                "tanvoid0",
            ),
            mk("p/other/thing", None, "thing"),
        ];
        let clusters = cluster_duplicates(&projects);
        assert!(
            clusters
                .iter()
                .any(|c| c.kind == "same-remote" && c.members.len() == 2),
            "trailing .git must not split a cluster"
        );
    }
}
