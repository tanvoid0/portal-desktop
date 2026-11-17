/**
 * Markdown → one line of plain text, for card/row previews that sit inside a
 * click or drag target (no links or copy buttons there). The detail view
 * renders the real markdown.
 */
export function stripMarkdown(md: string): string {
  return md
    .replace(/^\s*```.*$/gm, '') // fence lines, keep the code
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1') // images → alt
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1') // links → text
    .replace(/^\s{0,3}#{1,6}\s+/gm, '') // headings
    .replace(/^\s*>\s?/gm, '') // blockquotes
    .replace(/^\s*([-*+]|\d+\.)\s+/gm, '') // list markers
    .replace(/^\s*([-*_]\s*){3,}$/gm, '') // hr
    .replace(/(\*\*|__|~~|`)/g, '') // bold, strike, code
    .replace(/\*([^*\n]+)\*/g, '$1') // italics (single _ left alone: snake_case)
    .replace(/\s+/g, ' ')
    .trim();
}
