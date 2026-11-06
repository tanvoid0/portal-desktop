//! What a project folder is built with, and how to run it.
//!
//! The project root and every module [`detect`] finds are classified the same
//! way; frameworks, languages and package managers are collected across all of
//! them, root first, then the most common module stack. Names match the rows of
//! the `frameworks` / `languages` / `package_managers` tables so they resolve to
//! ids. Commands come from the root only: a monorepo's modules each run on their
//! own, so a single root command would be a guess.

use super::detect::{self, DetectedModule};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stack {
    pub frameworks: Vec<String>,
    pub languages: Vec<String>,
    pub package_managers: Vec<String>,
    pub build: Option<String>,
    pub start: Option<String>,
    pub test: Option<String>,
}

const NODE_STACKS: &[&str] = &[
    "nestjs",
    "nextjs",
    "sveltekit",
    "svelte",
    "nuxt",
    "vue",
    "react",
    "angular",
    "tauri",
    "node",
];

const NODE_PMS: &[&str] = &["pnpm", "yarn", "bun", "npm"];

/// Framework found by a word in a dependency file. Platforms that sit beside a UI
/// framework (Electron, Express, ...) are listed here too, so a React + Electron
/// app shows both.
const DEP_FRAMEWORKS: &[(&[&str], &str, &str)] = &[
    (&["requirements.txt", "pyproject.toml", "Pipfile"], "django", "Django"),
    (&["requirements.txt", "pyproject.toml", "Pipfile"], "fastapi", "FastAPI"),
    (&["requirements.txt", "pyproject.toml", "Pipfile"], "flask", "Flask"),
    (&["pom.xml", "build.gradle", "build.gradle.kts"], "spring-boot", "Spring"),
    (&["pom.xml", "build.gradle", "build.gradle.kts"], "quarkus", "Quarkus"),
    (&["composer.json"], "laravel/framework", "Laravel"),
    (&["composer.json"], "symfony/framework-bundle", "Symfony"),
    (&["Gemfile"], "'rails'", "Rails"),
    (&["Gemfile"], "'sinatra'", "Sinatra"),
    (&["package.json"], "\"electron\"", "Electron"),
    (&["package.json"], "\"express\"", "Express"),
    (&["package.json"], "\"fastify\"", "Fastify"),
    (&["package.json"], "\"gatsby\"", "Gatsby"),
    (&["package.json"], "\"@remix-run/react\"", "Remix"),
    (&["package.json"], "\"@tauri-apps/api\"", "Tauri"),
];

pub fn detect(root: &Path) -> Stack {
    detect_with(root, &detect::scan(root))
}

/// [`detect`] with modules the caller already scanned.
pub fn detect_with(root: &Path, modules: &[DetectedModule]) -> Stack {
    let mut stack = Stack::default();
    let root_kind = detect::classify(root);

    // Root first, then module stacks by how often they occur.
    let mut dirs: Vec<(std::path::PathBuf, String)> = Vec::new();
    if let Some(kind) = &root_kind {
        dirs.push((root.to_path_buf(), kind.clone()));
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for m in modules {
        *counts.entry(m.stack.as_str()).or_default() += 1;
    }
    let mut sorted: Vec<&DetectedModule> = modules.iter().collect();
    sorted.sort_by_key(|m| std::cmp::Reverse(counts[m.stack.as_str()]));
    for m in sorted {
        dirs.push((root.join(&m.rel_path), m.stack.clone()));
    }

    // A root that is itself an app is what the project is; its modules (a
    // `src-tauri`, a vendored example) only add languages. A container root with
    // no framework of its own takes its modules' frameworks instead.
    let mut module_frameworks = Vec::new();
    for (dir, kind) in &dirs {
        let is_root = dir == root;
        let target = if is_root {
            &mut stack.frameworks
        } else {
            &mut module_frameworks
        };
        if let Some(name) = detect::framework_name(kind) {
            push(target, name);
        }
        // Dependency-level frameworks only at the root: a module's own deps
        // (tooling, an example app) are noise for the project.
        if is_root {
            for (files, needle, name) in DEP_FRAMEWORKS {
                if files.iter().any(|f| read(dir, f).contains(needle)) {
                    push(&mut stack.frameworks, name);
                }
            }
        }
        if let Some(lang) = language(kind, dir) {
            push(&mut stack.languages, lang);
        }
        for pm in package_managers(kind, dir) {
            push(&mut stack.package_managers, pm);
        }
    }
    // Workspace packages have no lockfile of their own; npm is the default only
    // when no Node package manager was found anywhere.
    let has_node = dirs.iter().any(|(_, k)| NODE_STACKS.contains(&k.as_str()));
    if has_node && !stack.package_managers.iter().any(|p| NODE_PMS.contains(&p.as_str())) {
        push(&mut stack.package_managers, "npm");
    }
    if root.join("src-tauri").is_dir() {
        push(&mut stack.frameworks, "Tauri");
    }
    if stack.frameworks.is_empty() {
        stack.frameworks = module_frameworks;
    }
    if stack.languages.is_empty() {
        if let Some(lang) = language_by_files(root) {
            push(&mut stack.languages, lang);
        }
    }
    if let Some(kind) = &root_kind {
        let (build, start, test) = commands(kind, root, &stack);
        stack.build = build;
        stack.start = start;
        stack.test = test;
    }
    stack
}

fn push(list: &mut Vec<String>, name: &str) {
    if !list.iter().any(|n| n == name) {
        list.push(name.to_string());
    }
}

fn read(dir: &Path, file: &str) -> String {
    fs::read_to_string(dir.join(file))
        .unwrap_or_default()
        .to_lowercase()
}

fn language(kind: &str, dir: &Path) -> Option<&'static str> {
    let has = |f: &str| dir.join(f).is_file();
    Some(match kind {
        "flutter" | "dart" => "Dart",
        // ponytail: build-script language, not source language; a Groovy-gradle
        // Android app written in Kotlin reads as Java. Count .kt files if that bites.
        "android" | "gradle" | "java" => {
            if has("build.gradle.kts") || has("settings.gradle.kts") {
                "Kotlin"
            } else {
                "Java"
            }
        }
        k if NODE_STACKS.contains(&k) => {
            if has("tsconfig.json") || read(dir, "package.json").contains("\"typescript\"") {
                "TypeScript"
            } else {
                "JavaScript"
            }
        }
        "rust" => "Rust",
        "go" => "Go",
        "python" => "Python",
        "php" => "PHP",
        "ruby" => "Ruby",
        "dotnet" => "C#",
        _ => return None,
    })
}

fn package_managers(kind: &str, dir: &Path) -> Vec<&'static str> {
    let has = |f: &str| dir.join(f).is_file();
    let mut out = Vec::new();
    match kind {
        k if NODE_STACKS.contains(&k) => {
            for (lock, pm) in [
                ("pnpm-lock.yaml", "pnpm"),
                ("yarn.lock", "yarn"),
                ("bun.lockb", "bun"),
                ("bun.lock", "bun"),
                ("package-lock.json", "npm"),
            ] {
                if has(lock) && !out.contains(&pm) {
                    out.push(pm);
                }
            }
        }
        "flutter" | "dart" => out.push("pub"),
        "rust" => out.push("cargo"),
        "go" => out.push("go mod"),
        "python" => out.push(if has("poetry.lock") {
            "poetry"
        } else if has("Pipfile") {
            "pipenv"
        } else {
            "pip"
        }),
        "java" => out.push("maven"),
        "android" | "gradle" => out.push("gradle"),
        "php" => out.push("composer"),
        "ruby" => out.push("bundler"),
        "dotnet" => out.push("nuget"),
        _ => {}
    }
    out
}

/// Build, start and test commands for the root, `None` where there is no
/// sensible default. Node commands go through the package manager so local
/// binaries (`vite`, `next`) resolve.
fn commands(
    kind: &str,
    root: &Path,
    stack: &Stack,
) -> (Option<String>, Option<String>, Option<String>) {
    let s = |v: &str| Some(v.to_string());
    let gradlew = if cfg!(windows) { "gradlew" } else { "./gradlew" };
    let spring = stack.frameworks.iter().any(|f| f == "Spring");
    match kind {
        k if NODE_STACKS.contains(&k) => {
            let pm = stack
                .package_managers
                .iter()
                .find(|p| NODE_PMS.contains(&p.as_str()))
                .map_or("npm", String::as_str);
            let pkg: Value = serde_json::from_str(
                &fs::read_to_string(root.join("package.json")).unwrap_or_default(),
            )
            .unwrap_or(Value::Null);
            let script = |names: &[&str]| {
                names.iter().find_map(|n| {
                    let body = pkg.get("scripts")?.get(*n)?.as_str()?;
                    // npm init's placeholder test script only prints an error.
                    (!body.contains("no test specified")).then(|| format!("{pm} run {n}"))
                })
            };
            // `dev` first: `next start` and friends need a production build first.
            (script(&["build"]), script(&["dev", "start"]), script(&["test"]))
        }
        "flutter" => (None, s("flutter run"), s("flutter test")),
        "dart" => (None, s("dart run"), s("dart test")),
        "rust" => (s("cargo build"), s("cargo run"), s("cargo test")),
        "go" => (s("go build ./..."), s("go run ."), s("go test ./...")),
        "dotnet" => (s("dotnet build"), s("dotnet run"), s("dotnet test")),
        "java" => (
            s("mvn package"),
            spring.then(|| "mvn spring-boot:run".to_string()),
            s("mvn test"),
        ),
        "gradle" => (
            Some(format!("{gradlew} build")),
            spring.then(|| format!("{gradlew} bootRun")),
            Some(format!("{gradlew} test")),
        ),
        "android" => (
            Some(format!("{gradlew} assembleDebug")),
            None,
            Some(format!("{gradlew} test")),
        ),
        "python" => {
            let start = ["manage.py", "main.py", "app.py"]
                .into_iter()
                .find(|f| root.join(f).is_file())
                .map(|f| match f {
                    "manage.py" => "python manage.py runserver".to_string(),
                    f => format!("python {f}"),
                });
            let test = (root.join("tests").is_dir() || root.join("test").is_dir())
                .then(|| "pytest".to_string());
            (None, start, test)
        }
        _ => (None, None, None),
    }
}

/// Most common source language by file extension, for folders with no build
/// file (IDE-only Java projects, scripts, static sites). Bounded so a huge
/// folder cannot stall a scan.
fn language_by_files(root: &Path) -> Option<&'static str> {
    const MAX_DEPTH: usize = 4;
    const MAX_FILES: usize = 3000;
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let mut seen = 0;
    let mut stack = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            seen += 1;
            if seen > MAX_FILES {
                break;
            }
            let path = e.path();
            let name = e.file_name();
            let name = name.to_string_lossy();
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                if depth + 1 < MAX_DEPTH
                    && !name.starts_with('.')
                    && !detect::SKIP_DIRS.contains(&name.as_ref())
                {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            let lang = match path.extension().and_then(|x| x.to_str()) {
                Some("java") => "Java",
                Some("kt" | "kts") => "Kotlin",
                Some("py" | "ipynb") => "Python",
                Some("js" | "jsx" | "mjs" | "cjs") => "JavaScript",
                Some("ts" | "tsx") => "TypeScript",
                Some("dart") => "Dart",
                Some("go") => "Go",
                Some("rs") => "Rust",
                Some("cs") => "C#",
                Some("cpp" | "cc" | "cxx" | "hpp") => "C++",
                Some("c") => "C",
                Some("php") => "PHP",
                Some("rb") => "Ruby",
                Some("swift") => "Swift",
                Some("html" | "htm") => "HTML",
                _ => continue,
            };
            *counts.entry(lang).or_default() += 1;
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(l, _)| l)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("portal-stack-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn tauri_sveltekit_app() {
        let dir = tmp("tauri");
        touch(
            &dir,
            "package.json",
            r#"{"scripts":{"dev":"vite dev","build":"vite build","test":"echo \"Error: no test specified\" && exit 1"},
               "devDependencies":{"@sveltejs/kit":"2","@tauri-apps/api":"2","typescript":"5"}}"#,
        );
        touch(&dir, "pnpm-lock.yaml", "");
        touch(&dir, "src-tauri/Cargo.toml", "[package]");
        // A vendored app inside the project must not become one of its frameworks.
        touch(&dir, "inspirations/other/package.json", r#"{"dependencies":{"vue":"3"}}"#);
        let got = detect(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.frameworks, ["Svelte", "Tauri"]);
        assert_eq!(got.languages, ["TypeScript", "JavaScript", "Rust"]);
        assert_eq!(got.package_managers, ["pnpm", "cargo"]);
        assert_eq!(got.build.as_deref(), Some("pnpm run build"));
        assert_eq!(got.start.as_deref(), Some("pnpm run dev"));
        assert_eq!(got.test, None);
    }

    #[test]
    fn spring_maven_and_django() {
        let dir = tmp("spring");
        touch(&dir, "pom.xml", "<artifactId>spring-boot-starter-web</artifactId>");
        let got = detect(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.frameworks, ["Spring"]);
        assert_eq!(got.languages, ["Java"]);
        assert_eq!(got.package_managers, ["maven"]);
        assert_eq!(got.start.as_deref(), Some("mvn spring-boot:run"));

        let dir = tmp("django");
        touch(&dir, "requirements.txt", "Django==5.0\n");
        touch(&dir, "manage.py", "");
        let got = detect(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.frameworks, ["Django"]);
        assert_eq!(got.languages, ["Python"]);
        assert_eq!(got.start.as_deref(), Some("python manage.py runserver"));
    }

    #[test]
    fn monorepo_of_flutter_apps_and_markerless_folder() {
        let dir = tmp("mono");
        let flutter = "dependencies:\n  flutter:\n    sdk: flutter\n";
        touch(&dir, "apps/a/pubspec.yaml", flutter);
        touch(&dir, "apps/b/pubspec.yaml", flutter);
        touch(&dir, "server/go.mod", "module x");
        let got = detect(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.frameworks, ["Flutter"]);
        assert_eq!(got.languages, ["Dart", "Go"]);
        assert_eq!(got.start, None, "no root command for a monorepo");

        let dir = tmp("plain");
        touch(&dir, "src/Main.java", "");
        touch(&dir, "src/Util.java", "");
        touch(&dir, "README.md", "");
        let got = detect(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(got.languages, ["Java"]);
        assert!(got.frameworks.is_empty());
    }
}
