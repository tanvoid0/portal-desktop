//! Single-app build/run detection for local Docker deploys.
//!
//! Given a project directory, infer the stack, the port the app listens on
//! inside its container, and whether a Dockerfile / compose file already exists.
//! A `portal.yaml` committed to the project overrides any inferred value.
//!
//! This is the deploy-side cousin of `projects::discovery::scan`, which answers
//! a different question (catalogue many projects). Detection here is about one
//! project and needs a port + build hints the scan never computes.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Everything the deploy pipeline needs to know about one project.
#[derive(Debug, Clone)]
pub struct DetectedApp {
    /// Canonical stack id: node | python | go | rust | java | php | ruby | docker.
    pub stack: String,
    /// Port the app listens on *inside* the container. Never guessed as a host
    /// port — that is resolved separately against what is actually free.
    pub container_port: u16,
    /// Path to an existing Dockerfile. When set, it is used verbatim and no
    /// Dockerfile is generated.
    pub dockerfile: Option<String>,
    /// Hostname to route to this app. From `portal.yaml`, else `<name>.localhost`.
    pub host: Option<String>,
}

/// Subset of `portal.yaml` we read today. Unknown keys are ignored, so the file
/// can carry fields a later phase will consume without breaking this parse.
#[derive(Debug, Default, Deserialize)]
struct PortalConfig {
    name: Option<String>,
    stack: Option<String>,
    host: Option<String>,
    run: Option<PortalRun>,
}

#[derive(Debug, Default, Deserialize)]
struct PortalRun {
    port: Option<u16>,
}

/// Default listen port per stack. These are the ports the generated Dockerfiles
/// `EXPOSE`, so the two must stay in lock-step (see `generate_dockerfile`).
fn default_port(stack: &str) -> u16 {
    match stack {
        "node" => 3000,
        "nextjs" => 3000,
        "python" => 8000,
        "go" => 8080,
        "rust" => 8080,
        "java" => 8080,
        "php" => 80,
        "ruby" => 3000,
        _ => 8080,
    }
}

/// First manifest found → canonical stack. Order matters: a repo with both a
/// Dockerfile and a `package.json` is still "node" for generation purposes, but
/// a lone Dockerfile (no known manifest) is "docker".
fn stack_from_manifests(dir: &Path) -> Option<String> {
    let has = |f: &str| dir.join(f).exists();
    if has("package.json") {
        Some("node".into())
    } else if has("requirements.txt") || has("pyproject.toml") || has("setup.py") {
        Some("python".into())
    } else if has("go.mod") {
        Some("go".into())
    } else if has("Cargo.toml") {
        Some("rust".into())
    } else if has("pom.xml") || has("build.gradle") || has("build.gradle.kts") {
        Some("java".into())
    } else if has("composer.json") {
        Some("php".into())
    } else if has("Gemfile") {
        Some("ruby".into())
    } else {
        None
    }
}

/// Parse the first `EXPOSE <port>` out of a Dockerfile. The app's real listen
/// port beats any stack default when the author already declared it.
fn expose_port(dockerfile: &Path) -> Option<u16> {
    let text = std::fs::read_to_string(dockerfile).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("EXPOSE ").or_else(|| line.strip_prefix("expose ")) {
            // `EXPOSE 8080/tcp 9090` — take the first token's numeric part.
            if let Some(tok) = rest.split_whitespace().next() {
                let num = tok.split('/').next().unwrap_or(tok);
                if let Ok(p) = num.parse::<u16>() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// Does this Node project depend on Next.js? Next needs a `next build` before
/// `next start`, so it gets a build-aware Dockerfile rather than the generic
/// `npm start` template. Checked by reading package.json's dependency maps.
fn is_nextjs(dir: &Path) -> bool {
    let text = match std::fs::read_to_string(dir.join("package.json")) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    ["dependencies", "devDependencies"].iter().any(|k| {
        json.get(k)
            .and_then(|m| m.as_object())
            .is_some_and(|m| m.contains_key("next"))
    })
}

fn load_portal_config(dir: &Path) -> PortalConfig {
    for name in ["portal.yaml", "portal.yml"] {
        let f = dir.join(name);
        if f.exists() {
            if let Ok(text) = std::fs::read_to_string(&f) {
                if let Ok(cfg) = serde_yaml::from_str::<PortalConfig>(&text) {
                    return cfg;
                }
            }
        }
    }
    PortalConfig::default()
}

/// Lowercase, collapse to `[a-z0-9-]`, so a deployment name becomes a valid
/// hostname label and Docker/Traefik object name.
pub fn sanitize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "app".into()
    } else {
        trimmed
    }
}

/// Detect how to build and run the project at `project_path`.
///
/// `fallback_stack` is the stack the caller already had (e.g. the UI's project
/// type); it is used only when no manifest and no `portal.yaml` resolve one.
pub fn detect(project_path: &str, fallback_stack: &str) -> DetectedApp {
    let dir = Path::new(project_path);
    let cfg = load_portal_config(dir);

    let dockerfile_path = dir.join("Dockerfile");
    let dockerfile = if dockerfile_path.exists() {
        Some(dockerfile_path.to_string_lossy().to_string())
    } else {
        None
    };

    // Stack: portal.yaml → manifests → a lone Dockerfile → caller fallback.
    let explicit_stack = cfg.stack.is_some();
    let mut stack = cfg
        .stack
        .map(|s| s.to_lowercase())
        .or_else(|| stack_from_manifests(dir))
        .or_else(|| dockerfile.as_ref().map(|_| "docker".to_string()))
        .unwrap_or_else(|| fallback_stack.to_lowercase());

    // Framework refinement: a Node project that depends on Next.js needs a
    // build step, so narrow "node" → "nextjs". Skipped when the user pinned the
    // stack in portal.yaml.
    if !explicit_stack && stack == "node" && is_nextjs(dir) {
        stack = "nextjs".to_string();
    }

    // Port: portal.yaml run.port → Dockerfile EXPOSE → stack default.
    let container_port = cfg
        .run
        .and_then(|r| r.port)
        .or_else(|| dockerfile.as_deref().and_then(|d| expose_port(Path::new(d))))
        .unwrap_or_else(|| default_port(&stack));

    // Host: portal.yaml host → <name or dir>.localhost. The caller may override
    // the base name; here we only honour an explicit portal.yaml host.
    let host = cfg.host.or_else(|| {
        cfg.name
            .as_deref()
            .map(|n| format!("{}.localhost", sanitize_name(n)))
    });

    DetectedApp {
        stack,
        container_port,
        dockerfile,
        host,
    }
}

/// One `KEY=value` pair read from a project `.env*` file.
#[derive(Debug, Clone, Serialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

/// A `.env*` file found in the project, with its parsed contents. The UI lists
/// these so the user can pick one to seed the deployment's env, instead of
/// hand-typing variables that already live in the repo.
#[derive(Debug, Clone, Serialize)]
pub struct EnvFile {
    pub name: String,
    pub vars: Vec<EnvVar>,
}

/// Parse a dotenv file into ordered pairs. Deliberately minimal: `KEY=value`,
/// one per line, `#` comments and blanks skipped, an optional `export ` prefix
/// dropped, and a single layer of surrounding quotes stripped. No variable
/// expansion — the values are shown to the user and injected verbatim.
fn parse_env_file(path: &Path) -> Vec<EnvVar> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut vars = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = value.trim();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = &value[1..value.len() - 1];
        }
        vars.push(EnvVar {
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    vars
}

/// Every `.env*` file in the project directory (`.env`, `.env.local`,
/// `.env.production`, …), each with its parsed vars, sorted by name. Used by the
/// deploy UI to offer "load from .env" instead of manual entry. Returns empty if
/// the directory cannot be read.
pub fn list_env_files(project_path: &str) -> Vec<EnvFile> {
    let dir = Path::new(project_path);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n == ".env" || n.starts_with(".env."))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let vars = parse_env_file(&dir.join(&name));
            EnvFile { name, vars }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("detect-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn manifest_drives_stack_and_default_port() {
        let d = tmp("node");
        fs::write(d.join("package.json"), "{}").unwrap();
        let got = detect(d.to_str().unwrap(), "unknown");
        assert_eq!(got.stack, "node");
        assert_eq!(got.container_port, 3000);
        assert!(got.dockerfile.is_none());
    }

    #[test]
    fn dockerfile_expose_beats_stack_default() {
        let d = tmp("expose");
        fs::write(d.join("go.mod"), "module x").unwrap();
        fs::write(d.join("Dockerfile"), "FROM golang\nEXPOSE 9090/tcp\n").unwrap();
        let got = detect(d.to_str().unwrap(), "node");
        assert_eq!(got.stack, "go");
        assert_eq!(got.container_port, 9090, "EXPOSE must win over the go default");
        assert!(got.dockerfile.is_some());
    }

    #[test]
    fn portal_yaml_overrides_everything() {
        let d = tmp("yaml");
        fs::write(d.join("package.json"), "{}").unwrap();
        fs::write(d.join("Dockerfile"), "FROM node\nEXPOSE 3000\n").unwrap();
        fs::write(
            d.join("portal.yaml"),
            "stack: python\nhost: custom.localhost\nrun:\n  port: 7777\n",
        )
        .unwrap();
        let got = detect(d.to_str().unwrap(), "node");
        assert_eq!(got.stack, "python");
        assert_eq!(got.container_port, 7777, "portal.yaml beats EXPOSE");
        assert_eq!(got.host.as_deref(), Some("custom.localhost"));
    }

    #[test]
    fn next_dependency_narrows_node_to_nextjs() {
        let d = tmp("next");
        fs::write(
            d.join("package.json"),
            r#"{"dependencies":{"next":"^16.0.0","react":"^18"}}"#,
        )
        .unwrap();
        let got = detect(d.to_str().unwrap(), "node");
        assert_eq!(got.stack, "nextjs");
        assert_eq!(got.container_port, 3000);

        // A plain Node project stays "node".
        let d2 = tmp("plain-node");
        fs::write(d2.join("package.json"), r#"{"dependencies":{"express":"^4"}}"#).unwrap();
        assert_eq!(detect(d2.to_str().unwrap(), "node").stack, "node");
    }

    #[test]
    fn lists_and_parses_env_files() {
        let d = tmp("env");
        fs::write(
            d.join(".env.local"),
            "# comment\nexport FOO=bar\nQUOTED=\"hi there\"\nSINGLE='x=y'\nBLANK=\n\nnot_a_pair\n",
        )
        .unwrap();
        fs::write(d.join(".env"), "A=1\n").unwrap();
        fs::write(d.join("package.json"), "{}").unwrap(); // must be ignored

        let files = list_env_files(d.to_str().unwrap());
        let names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec![".env", ".env.local"], "sorted, only .env*");

        let local = &files[1];
        let get = |k: &str| local.vars.iter().find(|v| v.key == k).map(|v| v.value.as_str());
        assert_eq!(get("FOO"), Some("bar"), "export prefix dropped");
        assert_eq!(get("QUOTED"), Some("hi there"), "double quotes stripped");
        assert_eq!(get("SINGLE"), Some("x=y"), "value keeps inner =");
        assert_eq!(get("BLANK"), Some(""), "empty value kept");
        assert!(local.vars.iter().all(|v| v.key != "not_a_pair"), "non-pairs skipped");
    }

    #[test]
    fn sanitize_makes_valid_host_labels() {
        assert_eq!(sanitize_name("My App!"), "my-app");
        assert_eq!(sanitize_name("  weird__name  "), "weird-name");
        assert_eq!(sanitize_name("***"), "app");
    }
}
