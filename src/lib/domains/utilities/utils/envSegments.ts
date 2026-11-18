export function segments(value: string): string[] {
  return value.split(';');
}

export function hasMultipleSegments(value: string): boolean {
  return segments(value).filter((s) => s.trim() !== '').length > 1;
}
