/**
 * Terminal Output Parser
 * Parses terminal output for URLs, file paths, and error patterns
 */

export interface ParsedOutput {
  text: string;
  links: Array<{
    start: number;
    end: number;
    text: string;
    url: string;
    type: 'url' | 'file' | 'path';
  }>;
  errors: Array<{
    start: number;
    end: number;
    text: string;
    severity: 'error' | 'warning' | 'info';
  }>;
}

export function parseTerminalOutput(output: string): ParsedOutput {
  const links: ParsedOutput['links'] = [];
  const errors: ParsedOutput['errors'] = [];
  
  // URL patterns
  const urlPatterns = [
    /https?:\/\/[^\s]+/g,
    /ftp:\/\/[^\s]+/g,
    /file:\/\/[^\s]+/g,
    /ssh:\/\/[^\s]+/g,
    /git@[^\s]+/g
  ];
  
  // File path patterns
  const filePatterns = [
    /\/[^\s]+\.[a-zA-Z0-9]+/g,  // Absolute paths with extensions
    /\.\/[^\s]+/g,             // Relative paths
    /\.\.\/[^\s]+/g,           // Parent directory paths
    /[a-zA-Z0-9_-]+\/[^\s]+\.[a-zA-Z0-9]+/g  // Relative paths with extensions
  ];
  
  // Error patterns
  const errorPatterns = [
    // Common error formats
    { pattern: /error:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /Error:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /ERROR:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /failed:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /Failed:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /FAILED:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /fatal:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /Fatal:\s*(.+)/gi, severity: 'error' as const },
    { pattern: /FATAL:\s*(.+)/gi, severity: 'error' as const },
    
    // Warning patterns
    { pattern: /warning:\s*(.+)/gi, severity: 'warning' as const },
    { pattern: /Warning:\s*(.+)/gi, severity: 'warning' as const },
    { pattern: /WARNING:\s*(.+)/gi, severity: 'warning' as const },
    { pattern: /warn:\s*(.+)/gi, severity: 'warning' as const },
    { pattern: /Warn:\s*(.+)/gi, severity: 'warning' as const },
    { pattern: /WARN:\s*(.+)/gi, severity: 'warning' as const },
    
    // Info patterns
    { pattern: /info:\s*(.+)/gi, severity: 'info' as const },
    { pattern: /Info:\s*(.+)/gi, severity: 'info' as const },
    { pattern: /INFO:\s*(.+)/gi, severity: 'info' as const },
    
    // Tool-specific patterns
    { pattern: /npm ERR!/gi, severity: 'error' as const },
    { pattern: /npm WARN/gi, severity: 'warning' as const },
    { pattern: /cargo error:/gi, severity: 'error' as const },
    { pattern: /cargo warning:/gi, severity: 'warning' as const },
    { pattern: /python error:/gi, severity: 'error' as const },
    { pattern: /python warning:/gi, severity: 'warning' as const },
    { pattern: /pytest FAILED/gi, severity: 'error' as const },
    { pattern: /pytest WARNING/gi, severity: 'warning' as const },
    { pattern: /jest FAIL/gi, severity: 'error' as const },
    { pattern: /jest WARN/gi, severity: 'warning' as const }
  ];
  
  // Parse URLs
  urlPatterns.forEach(pattern => {
    let match;
    while ((match = pattern.exec(output)) !== null) {
      links.push({
        start: match.index,
        end: match.index + match[0].length,
        text: match[0],
        url: match[0],
        type: 'url'
      });
    }
  });
  
  // Parse file paths
  filePatterns.forEach(pattern => {
    let match;
    while ((match = pattern.exec(output)) !== null) {
      const text = match[0];
      // Skip if it's already a URL
      if (!text.startsWith('http') && !text.startsWith('ftp') && !text.startsWith('file') && !text.startsWith('ssh')) {
        links.push({
          start: match.index,
          end: match.index + text.length,
          text,
          url: text,
          type: 'file'
        });
      }
    }
  });
  
  // Parse errors
  errorPatterns.forEach(({ pattern, severity }) => {
    let match;
    while ((match = pattern.exec(output)) !== null) {
      errors.push({
        start: match.index,
        end: match.index + match[0].length,
        text: match[0],
        severity
      });
    }
  });
  
  // Sort by position to avoid conflicts
  links.sort((a, b) => a.start - b.start);
  errors.sort((a, b) => a.start - b.start);
  
  return {
    text: output,
    links,
    errors
  };
}

export function createClickableOutput(parsed: ParsedOutput): string {
  let result = parsed.text;
  let offset = 0;
  
  // Process links and errors together, sorted by position
  const allItems = [
    ...parsed.links.map(link => ({ ...link, type: 'link' as const })),
    ...parsed.errors.map(error => ({ ...error, type: 'error' as const }))
  ].sort((a, b) => a.start - b.start);
  
  allItems.forEach(item => {
    const start = item.start + offset;
    const end = item.end + offset;
    
    if (item.type === 'link') {
      const link = item as { type: 'link'; start: number; end: number; text: string; url: string };
      const replacement = `<a href="${link.url}" class="terminal-link" data-type="${link.type}">${link.text}</a>`;
      result = result.slice(0, start) + replacement + result.slice(end);
      offset += replacement.length - (end - start);
    } else if (item.type === 'error') {
      const error = item as ParsedOutput['errors'][0];
      const severityClass = `terminal-${error.severity}`;
      const replacement = `<span class="terminal-error ${severityClass}">${error.text}</span>`;
      result = result.slice(0, start) + replacement + result.slice(end);
      offset += replacement.length - (end - start);
    }
  });
  
  return result;
}

export function extractErrorSummary(parsed: ParsedOutput): {
  errorCount: number;
  warningCount: number;
  infoCount: number;
  errors: string[];
} {
  const errorCount = parsed.errors.filter(e => e.severity === 'error').length;
  const warningCount = parsed.errors.filter(e => e.severity === 'warning').length;
  const infoCount = parsed.errors.filter(e => e.severity === 'info').length;
  
  const errors = parsed.errors
    .filter(e => e.severity === 'error')
    .map(e => e.text)
    .slice(0, 5); // Limit to first 5 errors
  
  return {
    errorCount,
    warningCount,
    infoCount,
    errors
  };
}
