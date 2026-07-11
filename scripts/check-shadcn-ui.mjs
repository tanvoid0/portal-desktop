#!/usr/bin/env node
/**
 * Sanity check: no raw button/input/textarea/select/label in app Svelte files.
 * Mirrors eslint-rules/no-raw-ui-elements.mjs for CI or pre-commit use.
 */
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const SRC = join(ROOT, "src");

/** @param {string} dir @returns {string[]} */
function walkSvelte(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) out.push(...walkSvelte(p));
    else if (name.endsWith(".svelte")) out.push(p);
  }
  return out;
}

const patterns = [
  { re: /<button\b/g, tag: "button" },
  { re: /<textarea\b/g, tag: "textarea" },
  { re: /<select\b/g, tag: "select" },
  {
    re: /<input\b/g,
    tag: "input",
    allow: (s, i) => /\btype\s*=\s*["'](?:file|hidden)["']/i.test(s.slice(i, i + 120)),
  },
  {
    re: /<label\b/g,
    tag: "label",
    allow: (s, i) => /<Checkbox\b/.test(s.slice(i, i + 400)),
  },
];

let violations = 0;

for (const file of walkSvelte(SRC)) {
  const rel = relative(ROOT, file).replace(/\\/g, "/");
  if (rel.startsWith("src/lib/components/ui/")) continue;

  const source = readFileSync(file, "utf8");
  for (const { re, tag, allow } of patterns) {
    re.lastIndex = 0;
    let m;
    while ((m = re.exec(source)) !== null) {
      if (allow?.(source, m.index)) continue;
      const line = source.slice(0, m.index).split("\n").length;
      console.error(`${rel}:${line}: raw <${tag}> — use shadcn-svelte component`);
      violations++;
    }
  }
}

if (violations > 0) {
  console.error(`\n${violations} shadcn UI violation(s). See .cursor/rules/shadcn-ui.mdc`);
  process.exit(1);
}

console.log("shadcn UI check passed");
