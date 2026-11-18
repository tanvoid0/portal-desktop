export function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

export function formatDate(dateStr: string | null): string {
  if (!dateStr) return 'Never';
  try {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(new Date(dateStr));
  } catch {
    return dateStr;
  }
}

export function formatPatternData(data: Record<string, unknown>): string {
  try {
    return JSON.stringify(data, null, 2);
  } catch {
    return String(data);
  }
}

export function getPatternTypeLabel(type: string): string {
  return type.charAt(0).toUpperCase() + type.slice(1);
}

export function getEventTypeLabel(type: string): string {
  return type
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');
}
