//! Placeholders in a command line that is about to run through a shell.
//!
//! Pipeline steps, Actions and script blocks are templates: `${NAME}` (or
//! `$NAME`) stands for a value such as the project's folder name, its path or
//! something the user typed. A value is data, never shell syntax, so [`render`]
//! quotes each one for the shell [`shell_command`] spawns, and for whatever
//! quote the template has open at that point (`"${PROJECT_PATH}"` and a bare
//! `${PROJECT_PATH}` both work). One left-to-right pass: substituted text is
//! never scanned again, so a value that reads `${OTHER}` stays literal.
//!
//! **sh.** Outside quotes a value becomes `'...'`, each `'` written `'\''`;
//! inside the template's `'...'` only that `'` rewrite is needed; inside its
//! `"..."` the four characters `\ $ ` "` get a backslash.
//!
//! **cmd.exe** has no quoting that switches everything off, so each hazard is
//! handled on its own. The value sits inside `"..."` (its own pair, or the
//! template's open one), where `& | < > ( ) ^` are literal. An embedded `"` is
//! doubled: cmd's quote state flips twice and stays inside, and the program's
//! C runtime reads `""` as one `"` (backslashes before it are doubled, as the
//! runtime wants). `%` is written `%%cd:~,%`, which cmd's `%VAR%` pass reduces
//! to a lone `%`, so a value's own `%PATH%` never expands. `!` is literal since
//! the shell runs with `/V:OFF`. CR and LF cannot be carried on a cmd line and
//! become spaces. The encoding also survives the second cmd parse that a
//! `.cmd` shim (npm, pnpm) does on its arguments: `%*` is not rescanned.
//!
//! A value that needs no quoting (letters, digits, `+-./:@_`, and `\` on
//! Windows) goes in bare, so `${PACKAGE_MANAGER} install` still reads
//! `pnpm install`. The flip side: a value is always one word, never a list of
//! arguments.

use std::collections::HashMap;
use tokio::process::Command;

#[derive(Clone, Copy, Debug)]
enum Shell {
    Sh,
    Cmd,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    None,
    Single,
    Double,
}

const HOST: Shell = if cfg!(windows) { Shell::Cmd } else { Shell::Sh };

/// Fill `template`'s placeholders, quoted for [`shell_command`]. Unknown names
/// are left as written.
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    render_for(HOST, template, vars)
}

/// Fill placeholders with the raw values, in one pass, for text no shell
/// parses (an environment variable's value).
pub fn expand_raw(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(c) = rest.chars().next() {
        let len = match lookup(rest, vars) {
            Some((value, len)) => {
                out.push_str(value);
                len
            }
            None => {
                out.push(c);
                c.len_utf8()
            }
        };
        rest = &rest[len..];
    }
    out
}

/// The shell [`render`] quotes for: `cmd /S /V:OFF /C "<line>"` on Windows,
/// `sh -c <line>` elsewhere. On Windows the line is passed verbatim: std's argv
/// quoting turns each `"` into `\"`, which cmd does not read as an escape, so
/// every quoted argument used to arrive mangled. `/S` makes cmd strip exactly
/// the outer pair added here.
pub fn shell_command(line: &str) -> Command {
    #[cfg(windows)]
    {
        use crate::process_ext::NoWindowExt;
        let mut c = Command::new("cmd");
        c.no_window();
        c.args(["/S", "/V:OFF", "/C"])
            .raw_arg(format!("\"{line}\""));
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new("sh");
        c.args(["-c", line]);
        c
    }
}

fn render_for(shell: Shell, template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut quote = Quote::None;
    let mut rest = template;
    while let Some(c) = rest.chars().next() {
        if let Some((value, len)) = lookup(rest, vars) {
            rest = &rest[len..];
            match shell {
                Shell::Sh => quote_sh(&mut out, value, quote),
                Shell::Cmd => quote_cmd(&mut out, value, quote, rest.starts_with('"')),
            }
            continue;
        }
        // An escape takes the next char with it: `\${X}` / `^${X}` stay
        // literal, and an escaped quote does not open or close anything.
        let escape = match shell {
            Shell::Sh => c == '\\' && quote != Quote::Single,
            Shell::Cmd => c == '^' && quote == Quote::None,
        };
        let mut len = c.len_utf8();
        if escape {
            len += rest[len..].chars().next().map_or(0, char::len_utf8);
        }
        quote = match (shell, quote, c) {
            _ if escape => quote,
            (Shell::Sh, Quote::None, '\'') => Quote::Single,
            (Shell::Sh, Quote::Single, '\'') => Quote::None,
            (_, Quote::None, '"') => Quote::Double,
            (_, Quote::Double, '"') => Quote::None,
            _ => quote,
        };
        out.push_str(&rest[..len]);
        rest = &rest[len..];
    }
    out
}

/// `${NAME}` or `$NAME` at the start of `s`, when NAME is a known variable:
/// its value and the placeholder's byte length.
fn lookup<'v>(s: &str, vars: &'v HashMap<String, String>) -> Option<(&'v str, usize)> {
    let body = s.strip_prefix('$')?;
    let (name, len) = match body.strip_prefix('{') {
        Some(inner) => {
            let end = inner.find('}')?;
            (&inner[..end], end + 3)
        }
        None => {
            let end = body
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(body.len());
            (&body[..end], end + 1)
        }
    };
    vars.get(name).map(|v| (v.as_str(), len))
}

fn quote_sh(out: &mut String, value: &str, quote: Quote) {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "+-./:@_".contains(c))
    {
        out.push_str(value);
        return;
    }
    match quote {
        Quote::None => {
            out.push('\'');
            out.push_str(&value.replace('\'', r"'\''"));
            out.push('\'');
        }
        Quote::Single => out.push_str(&value.replace('\'', r"'\''")),
        Quote::Double => {
            for c in value.chars() {
                if matches!(c, '\\' | '$' | '`' | '"') {
                    out.push('\\');
                }
                out.push(c);
            }
        }
    }
}

/// `quote_follows`: the template's next char is `"`, so trailing backslashes
/// must be doubled for the C runtime not to read `\"` as a literal quote.
fn quote_cmd(out: &mut String, value: &str, quote: Quote, quote_follows: bool) {
    if !value.ends_with('\\')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || r"+-./:@_\".contains(c))
    {
        out.push_str(value);
        return;
    }
    let own_quotes = quote == Quote::None;
    if own_quotes {
        out.push('"');
    }
    let mut backslashes = 0;
    for c in value.chars() {
        match c {
            '\\' => {
                backslashes += 1;
                out.push(c);
                continue;
            }
            '"' => {
                out.extend(std::iter::repeat_n('\\', backslashes));
                out.push_str("\"\"");
            }
            '%' => out.push_str("%%cd:~,%"),
            '\r' | '\n' => out.push(' '),
            _ => out.push(c),
        }
        backslashes = 0;
    }
    if own_quotes || quote_follows {
        out.extend(std::iter::repeat_n('\\', backslashes));
    }
    if own_quotes {
        out.push('"');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// The values the shell-injection fix is pinned on.
    const HOSTILE: [&str; 5] = ["a & calc", "$(id)", "%PATH%", "a\"b", "${OTHER}"];

    #[test]
    fn sh_quotes_each_hostile_value() {
        let expect = ["'a & calc'", "'$(id)'", "'%PATH%'", "'a\"b'", "'${OTHER}'"];
        for (v, want) in HOSTILE.iter().zip(expect) {
            let vs = vars(&[("V", v), ("OTHER", "x")]);
            assert_eq!(
                render_for(Shell::Sh, "echo ${V}", &vs),
                format!("echo {want}")
            );
        }
    }

    #[test]
    fn cmd_quotes_each_hostile_value() {
        let expect = [
            r#""a & calc""#,
            r#""$(id)""#,
            r#""%%cd:~,%PATH%%cd:~,%""#,
            r#""a""b""#,
            r#""${OTHER}""#,
        ];
        for (v, want) in HOSTILE.iter().zip(expect) {
            let vs = vars(&[("V", v), ("OTHER", "x")]);
            assert_eq!(
                render_for(Shell::Cmd, "echo ${V}", &vs),
                format!("echo {want}")
            );
        }
    }

    #[test]
    fn substituted_text_is_not_expanded_again() {
        // Whatever order the map iterates in, one pass: OTHER's text in A stays literal.
        let vs = vars(&[("A", "${B}"), ("B", "${A}")]);
        assert_eq!(expand_raw("${A} ${B}", &vs), "${B} ${A}");
        assert_eq!(render_for(Shell::Sh, "${A}", &vs), "'${B}'");
    }

    #[test]
    fn honours_the_quote_the_template_has_open() {
        let vs = vars(&[("P", "/a b/it's $x")]);
        assert_eq!(
            render_for(Shell::Sh, "cd ${P}", &vs),
            r"cd '/a b/it'\''s $x'"
        );
        assert_eq!(
            render_for(Shell::Sh, "cd \"${P}\"", &vs),
            r#"cd "/a b/it's \$x""#
        );
        assert_eq!(
            render_for(Shell::Sh, "cd '${P}'", &vs),
            r"cd '/a b/it'\''s $x'"
        );

        let vs = vars(&[("P", r"C:\a b\x&y")]);
        assert_eq!(render_for(Shell::Cmd, "cd ${P}", &vs), r#"cd "C:\a b\x&y""#);
        assert_eq!(
            render_for(Shell::Cmd, "cd \"${P}\"", &vs),
            r#"cd "C:\a b\x&y""#
        );
    }

    #[test]
    fn plain_values_go_in_bare() {
        let vs = vars(&[("PM", "pnpm"), ("P", r"C:\Users\me\proj"), ("E", "")]);
        assert_eq!(
            render_for(Shell::Cmd, "${PM} install ${P}${E}", &vs),
            r"pnpm install C:\Users\me\proj"
        );
        assert_eq!(
            render_for(Shell::Sh, "$PM run build${E}", &vs),
            "pnpm run build"
        );
    }

    #[test]
    fn escaped_and_unknown_placeholders_stay_literal() {
        let vs = vars(&[("X", "a b")]);
        assert_eq!(
            render_for(Shell::Sh, r"echo \${X} ${Y} $XY", &vs),
            r"echo \${X} ${Y} $XY"
        );
        assert_eq!(render_for(Shell::Cmd, "echo ^${X}", &vs), "echo ^${X}");
        // An escaped quote does not change the context: X is still outside quotes.
        assert_eq!(
            render_for(Shell::Sh, r#"echo \" ${X}"#, &vs),
            r#"echo \" 'a b'"#
        );
    }

    #[test]
    fn cmd_backslashes_before_a_quote_are_doubled() {
        let vs = vars(&[("D", r"C:\dir\"), ("Q", r#"a\"b"#)]);
        assert_eq!(render_for(Shell::Cmd, "x ${D}", &vs), r#"x "C:\dir\\""#);
        assert_eq!(render_for(Shell::Cmd, "x \"${D}\"", &vs), r#"x "C:\dir\\""#);
        assert_eq!(render_for(Shell::Cmd, "x ${Q}", &vs), r#"x "a\\""b""#);
        assert_eq!(
            render_for(Shell::Cmd, "x ${V}", &vars(&[("V", "a\r\nb!^")])),
            r#"x "a  b!^""#
        );
    }

    /// Through the real host shell: the program gets the value as one
    /// argument and nothing else runs.
    #[tokio::test]
    async fn host_shell_runs_nothing_from_a_value() {
        let dir = std::env::temp_dir().join(format!("shell_template_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        // A `.cmd` shim like pnpm.cmd: cmd parses its `%*` a second time.
        let shim = dir.join("shim.cmd");
        std::fs::write(&shim, "@echo %*\r\n").unwrap();
        // (template, the part of it before the value's argument)
        let templates: &[(&str, &str)] = if cfg!(windows) {
            &[
                ("echo ${V}", "echo "),
                ("echo \"${V}\"", "echo "),
                ("${SHIM} ${V}", "${SHIM} "),
            ]
        } else {
            &[
                ("printf '%s\\n' ${V}", ""),
                ("printf '%s\\n' \"${V}\"", ""),
                ("printf '%s\\n' '${V}'", ""),
            ]
        };
        let extra = ["it's", "a  b", r"trail\", "!PATH! ^x ;id", "a\"&\"calc"];
        for v in HOSTILE.iter().chain(&extra) {
            let vs = vars(&[("V", v), ("OTHER", "x"), ("SHIM", shim.to_str().unwrap())]);
            for (template, head) in templates {
                let line = render(template, &vs);
                let out = shell_command(&line).output().await.unwrap();
                let got = String::from_utf8_lossy(&out.stdout);
                let got = got.trim_end_matches(['\r', '\n']);
                // cmd's echo prints its argument text as written, quotes and
                // all, once cmd has resolved the `%` encoding.
                let want = if cfg!(windows) {
                    line[render(head, &vs).len()..].replace("%%cd:~,%", "%")
                } else {
                    v.to_string()
                };
                assert_eq!(got, want, "{template} with {v:?} rendered as {line}");
            }
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
