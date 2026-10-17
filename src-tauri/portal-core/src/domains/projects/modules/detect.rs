//! Finds the modules (apps, services, packages) inside a project folder.
//!
//! A module is a directory with a build marker (`pubspec.yaml`, `package.json`,
//! `Cargo.toml`, ...). The project root itself is never a module: the root is the
//! project, and its own stack is what the project already records. The walk stops
//! at a module, so `apps/gym/ui/pubspec.yaml` belongs to `apps/gym` rather than
//! being listed beside it. Depth is capped, and so is the count, so a huge
//! folder cannot stall a scan.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

const MAX_DEPTH: usize = 3;
const MAX_MODULES: usize = 200;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "build",
    "dist",
    "out",
    "vendor",
    "venv",
    "__pycache__",
    "Pods",
    "coverage",
    "bin",
    "obj",
    // Not part of the shipped project: old code and demo apps for a package.
    "_archive",
    "archive",
    "archived",
    "deprecated",
    "example",
    "examples",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedModule {
    /// Relative to the project root, forward slashes.
    pub rel_path: String,
    pub name: String,
    /// `flutter`, `dart`, `nestjs`, `android`, `rust`, `node`, ... a label, not an enum.
    pub stack: String,
    /// `app`, `service` or `package`.
    pub role: String,
}

pub fn scan(root: &Path) -> Vec<DetectedModule> {
    let mut found = Vec::new();
    walk(root, root, 0, &mut found);
    found.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    found
}

fn walk(root: &Path, dir: &Path, depth: usize, found: &mut Vec<DetectedModule>) {
    if depth >= MAX_DEPTH || found.len() >= MAX_MODULES {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut dirs: Vec<_> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for path in dirs {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with('.') || SKIP_DIRS.contains(&name) {
            continue;
        }
        match classify(&path) {
            Some(stack) => {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let role = role_for(&rel, &stack);
                found.push(DetectedModule {
                    rel_path: rel,
                    name: name.to_string(),
                    stack,
                    role,
                });
            }
            None => walk(root, &path, depth + 1, found),
        }
    }
}

/// The stack of the directory's own marker, or `None` if it is not a module.
pub fn classify(dir: &Path) -> Option<String> {
    let has = |f: &str| dir.join(f).is_file();
    if has("pubspec.yaml") {
        let spec = fs::read_to_string(dir.join("pubspec.yaml")).unwrap_or_default();
        let flutter = spec
            .lines()
            .any(|l| l.trim().replace(' ', "") == "sdk:flutter");
        return Some(if flutter { "flutter" } else { "dart" }.into());
    }
    if has("nest-cli.json") {
        return Some("nestjs".into());
    }
    if has("package.json") {
        return Some(node_stack(
            &fs::read_to_string(dir.join("package.json")).unwrap_or_default(),
        ));
    }
    if has("build.gradle")
        || has("build.gradle.kts")
        || has("settings.gradle")
        || has("settings.gradle.kts")
    {
        let android = dir.join("src/main/AndroidManifest.xml").is_file()
            || dir.join("app/src/main/AndroidManifest.xml").is_file();
        return Some(if android { "android" } else { "gradle" }.into());
    }
    for (file, stack) in [
        ("Cargo.toml", "rust"),
        ("go.mod", "go"),
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("pom.xml", "java"),
        ("composer.json", "php"),
        ("Gemfile", "ruby"),
    ] {
        if has(file) {
            return Some(stack.into());
        }
    }
    let dotnet = fs::read_dir(dir).ok()?.flatten().any(|e| {
        matches!(
            e.path().extension().and_then(|x| x.to_str()),
            Some("csproj" | "sln")
        )
    });
    dotnet.then(|| "dotnet".into())
}

fn node_stack(package_json: &str) -> String {
    let Ok(pkg) = serde_json::from_str::<Value>(package_json) else {
        return "node".into();
    };
    let has_dep = |name: &str| {
        ["dependencies", "devDependencies"]
            .iter()
            .any(|k| pkg.get(k).and_then(|d| d.get(name)).is_some())
    };
    // Most specific first: a Nest or SvelteKit app also depends on plainer packages.
    for (dep, stack) in [
        ("@nestjs/core", "nestjs"),
        ("next", "nextjs"),
        ("@sveltejs/kit", "sveltekit"),
        ("nuxt", "nuxt"),
        ("@angular/core", "angular"),
        ("vue", "vue"),
        ("svelte", "svelte"),
        ("react", "react"),
        ("@tauri-apps/api", "tauri"),
    ] {
        if has_dep(dep) {
            return stack.into();
        }
    }
    "node".into()
}

fn role_for(rel_path: &str, stack: &str) -> String {
    let first = rel_path.split('/').next().unwrap_or_default();
    let role = if matches!(first, "packages" | "libs" | "crates" | "shared") {
        "package"
    } else if matches!(
        first,
        "services" | "server" | "servers" | "api" | "backend"
    ) || matches!(stack, "nestjs" | "go" | "java" | "dotnet")
    {
        "service"
    } else {
        "app"
    };
    role.into()
}

/// The framework name this stack is known by in the `frameworks` table, if it is a framework.
pub fn framework_name(stack: &str) -> Option<&'static str> {
    Some(match stack {
        "flutter" => "Flutter",
        "nestjs" => "NestJS",
        "android" => "Android",
        "nextjs" => "Next.js",
        "sveltekit" | "svelte" => "Svelte",
        "nuxt" => "Nuxt",
        "vue" => "Vue.js",
        "react" => "React",
        "angular" => "Angular",
        "tauri" => "Tauri",
        _ => return None,
    })
}

/// The language this stack is written in, by its `languages` table name.
pub fn language_name(stack: &str) -> Option<&'static str> {
    Some(match stack {
        "flutter" | "dart" => "Dart",
        "android" | "gradle" => "Kotlin",
        "nestjs" | "nextjs" | "sveltekit" | "svelte" | "nuxt" | "vue" | "react" | "angular" | "node" | "tauri" => "TypeScript",
        "rust" => "Rust",
        "go" => "Go",
        "python" => "Python",
        "java" => "Java",
        "php" => "PHP",
        "ruby" => "Ruby",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    #[test]
    fn finds_mixed_stacks_and_stops_at_a_module() {
        let dir = std::env::temp_dir().join(format!("portal-modules-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        touch(&dir, "package.json", r#"{"scripts":{}}"#); // root: never a module
        touch(
            &dir,
            "apps/gym/pubspec.yaml",
            "dependencies:\n  flutter:\n    sdk: flutter\n",
        );
        touch(&dir, "apps/gym/ui/pubspec.yaml", "name: ui\n"); // inside a module: ignored
        touch(&dir, "apps/launcher/build.gradle.kts", "");
        touch(&dir, "apps/launcher/src/main/AndroidManifest.xml", "");
        touch(
            &dir,
            "server/api/package.json",
            r#"{"dependencies":{"@nestjs/core":"1"}}"#,
        );
        touch(&dir, "packages/core/pubspec.yaml", "name: core\n");
        touch(&dir, "node_modules/x/package.json", "{}"); // skipped
        touch(&dir, "_archive/old/pubspec.yaml", "name: old\n"); // skipped
        touch(&dir, "packages/core/example/pubspec.yaml", "name: ex\n"); // inside a module anyway

        let got: Vec<_> = scan(&dir)
            .into_iter()
            .map(|m| (m.rel_path, m.stack, m.role))
            .collect();
        let _ = fs::remove_dir_all(&dir);
        let want: Vec<(String, String, String)> = [
            ("apps/gym", "flutter", "app"),
            ("apps/launcher", "android", "app"),
            ("packages/core", "dart", "package"),
            ("server/api", "nestjs", "service"),
        ]
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
        assert_eq!(got, want);
    }
}
