import { marked } from "marked";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import css from "highlight.js/lib/languages/css";
import go from "highlight.js/lib/languages/go";
import java from "highlight.js/lib/languages/java";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

const LANG_ALIASES: Record<string, string> = {
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  jsx: "javascript",
  py: "python",
  rb: "ruby",
  sh: "bash",
  shell: "bash",
  zsh: "bash",
  yml: "yaml",
  md: "markdown",
  rs: "rust",
  "c++": "cpp",
  h: "c",
  hpp: "cpp",
  plaintext: "text",
  txt: "text",
};

for (const [name, lang] of [
  ["bash", bash],
  ["css", css],
  ["go", go],
  ["java", java],
  ["javascript", javascript],
  ["json", json],
  ["markdown", markdown],
  ["python", python],
  ["rust", rust],
  ["sql", sql],
  ["typescript", typescript],
  ["xml", xml],
  ["yaml", yaml],
] as const) {
  hljs.registerLanguage(name, lang);
}

function resolveLanguage(lang?: string | null): string | null {
  if (!lang) return null;
  const normalized = LANG_ALIASES[lang.toLowerCase()] ?? lang.toLowerCase();
  return hljs.getLanguage(normalized) ? normalized : null;
}

function highlightCode(text: string, lang?: string | null): string {
  const resolved = resolveLanguage(lang);
  if (resolved) {
    return hljs.highlight(text, { language: resolved }).value;
  }
  const auto = hljs.highlightAuto(text, [
    "typescript",
    "javascript",
    "python",
    "rust",
    "bash",
    "json",
    "yaml",
    "sql",
    "go",
    "java",
    "css",
    "xml",
    "markdown",
  ]);
  return auto.value;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const MARKDOWN_FENCE_LANGS = new Set(["markdown", "md"]);

let configured = false;
let renderingMarkdownPreview = false;

function renderCodeBlock(text: string, lang?: string | null): string {
  const displayLang = (lang || "text").toLowerCase();

  if (
    !renderingMarkdownPreview &&
    MARKDOWN_FENCE_LANGS.has(displayLang)
  ) {
    renderingMarkdownPreview = true;
    try {
      const body = marked.parse(text) as string;
      const escaped = escapeHtml(text);
      return `<div class="chat-md-preview">
  <div class="chat-md-preview-header">
    <span class="chat-md-preview-label">Markdown preview</span>
    <button type="button" class="chat-md-preview-toggle" data-mode="source" aria-label="Show markdown source">
      Show source
    </button>
  </div>
  <div class="chat-md-preview-body">${body}</div>
  <pre class="chat-md-preview-source" hidden><code>${escaped}</code></pre>
</div>`;
    } finally {
      renderingMarkdownPreview = false;
    }
  }

  const highlighted = highlightCode(text, lang);
  return `<div class="chat-code-block" data-lang="${escapeHtml(displayLang)}">
  <div class="chat-code-header">
    <span class="chat-code-lang">${escapeHtml(displayLang)}</span>
    <button type="button" class="chat-code-copy" aria-label="Copy code">
      <span class="chat-code-copy-label">Copy</span>
    </button>
  </div>
  <pre class="chat-code-pre"><code class="hljs">${highlighted}</code></pre>
</div>`;
}

function configureMarked() {
  if (configured) return;
  configured = true;

  marked.use({
    breaks: true,
    gfm: true,
    renderer: {
      code({ text, lang }) {
        return renderCodeBlock(text, lang);
      },
      link({ href, title, text }) {
        const titleAttr = title ? ` title="${escapeHtml(title)}"` : "";
        return `<a href="${escapeHtml(href)}"${titleAttr} target="_blank" rel="noopener noreferrer" class="chat-md-link">${text}</a>`;
      },
      blockquote({ text }) {
        return `<blockquote class="chat-blockquote">${text}</blockquote>`;
      },
    },
  });
}

/** Close an unclosed fenced code block so streaming markdown renders cleanly. */
export function prepareStreamingMarkdown(content: string): string {
  const fenceCount = (content.match(/^```/gm) || []).length;
  if (fenceCount % 2 === 1) {
    return `${content}\n\`\`\``;
  }
  return content;
}

export function renderMarkdown(
  content: string,
  options: { streaming?: boolean } = {},
): string {
  configureMarked();
  const source = options.streaming
    ? prepareStreamingMarkdown(content)
    : content;
  return marked.parse(source) as string;
}
