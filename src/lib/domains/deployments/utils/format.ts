export function fmtBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / Math.pow(1024, i);
  return `${value < 10 ? value.toFixed(1) : value.toFixed(0)} ${units[i]}`;
}

export function fmtPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

export function isContainerRunning(status: string): boolean {
  const s = status?.toLowerCase() || "";
  return s.includes("running") || s.includes("up");
}

export function containerStatusGroup(
  status: string,
): "running" | "stopped" | "other" {
  const s = status?.toLowerCase() || "";
  if (s.includes("running") || s.includes("up")) return "running";
  if (
    s.includes("exited") ||
    s.includes("stopped") ||
    s.includes("created")
  )
    return "stopped";
  return "other";
}

export function shortImageName(image: string): string {
  const withoutDigest = image.split("@")[0];
  const parts = withoutDigest.split("/");
  return parts[parts.length - 1] || image;
}
