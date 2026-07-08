const MARKDOWN_EXTENSIONS = /\.(md|mdx|markdown)$/i;

export function isMarkdownPath(path: string): boolean {
  return MARKDOWN_EXTENSIONS.test(path.trim());
}
